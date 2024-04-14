pub mod lens;

use crate::camera::lens::{Lens, LensSystem};
use crate::lens::IMAGING_MEDIUM_N_D;
use rand::{rngs::StdRng, Rng};
use rand_distr::{UnitDisc, UnitSphere};

use crate::shape::Ray;

pub trait Camera: Send + Sync {
    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray;
}

/// A simple thin-lens perspective camera
#[derive(Copy, Clone, Debug)]
pub struct ThinLensCamera {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing (normalized).
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction` (normalized).
    pub up: glm::DVec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,

    /// Aperture radius for depth-of-field effects
    pub aperture: f64,

    /// Focal distance, if aperture radius is nonzero
    pub focal_distance: f64,
}

impl Default for ThinLensCamera {
    fn default() -> Self {
        Self {
            eye: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f64::consts::FRAC_PI_6,
            aperture: 0.0,
            focal_distance: 0.0,
        }
    }
}

impl ThinLensCamera {
    /// Perspective camera looking at a point, with a given field of view
    pub fn look_at(eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64) -> Self {
        let direction = (center - eye).normalize();
        let up = (up - up.dot(&direction) * direction).normalize();
        Self {
            eye,
            direction,
            up,
            fov,
            aperture: 0.0,
            focal_distance: 0.0,
        }
    }

    /// Focus the camera on a position, with simulated depth-of-field
    pub fn focus(mut self, focal_point: glm::DVec3, aperture: f64) -> Self {
        self.focal_distance = (focal_point - self.eye).dot(&self.direction);
        self.aperture = aperture;
        self
    }
}

impl Camera for ThinLensCamera {
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray {
        // cot(f / 2) = depth / radius
        let d = (self.fov / 2.0).tan().recip();
        let right = glm::cross(&self.direction, &self.up).normalize();
        let mut origin = self.eye;
        let mut new_dir = d * self.direction + x * right + y * self.up;
        if self.aperture > 0.0 {
            // Depth of field
            let focal_point = origin + new_dir.normalize() * self.focal_distance;
            let [x, y]: [f64; 2] = rng.sample(UnitDisc);
            origin += (x * right + y * self.up) * self.aperture;
            new_dir = focal_point - origin;
        }
        Ray {
            origin,
            dir: new_dir.normalize(),
        }
    }
}

pub struct PhysicalCamera<L> {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::DVec3,

    /// Width of image sensor (mm).
    pub sensor_width: f64,

    /// Height of image sensor (mm).
    pub sensor_height: f64,

    /// Lens.
    pub lens: L,

    /// Current lens system.
    pub lens_system: LensSystem,
}

impl<L: Lens + Default> Default for PhysicalCamera<L> {
    fn default() -> Self {
        let lens = L::default();
        let lens_system = lens.lens_system(100.);
        Self {
            eye: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            sensor_width: 4.,
            sensor_height: 3.,
            lens,
            lens_system,
        }
    }
}

impl<L: Lens> PhysicalCamera<L> {
    fn look_at(&mut self, eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3) {
        self.eye = eye;
        self.direction = (center - eye).normalize();
        self.up = (up - up.dot(&self.direction) * self.direction).normalize();
    }

    fn focus(&mut self, focal_point: glm::DVec3) {
        self.lens_system = self
            .lens
            .lens_system((focal_point - self.eye).dot(&self.direction).abs());
    }
}

impl<L: Lens> Camera for PhysicalCamera<L> {
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray {
        let right = glm::cross(&self.direction, &self.up).normalize();

        let mut p =
            self.eye + self.sensor_width * x / 2. * right + self.sensor_height * y / 2. * self.up;

        loop {
            let new_p = if let Some(surface) = self.lens_system.surfaces.last() {
                let [x, y]: [f64; 2] = rng.sample(UnitDisc);
                let z = (surface.radius * surface.radius - x * x - y * y).sqrt();
                self.eye
                    + self.direction
                        * (surface.thickness
                            - surface.radius * (surface.radius.abs() - z) / surface.radius.abs())
                    + x * right * surface.aperture / 2.
                    + y * self.up * surface.aperture / 2.
            } else {
                let [x, y, z]: [f64; 3] = rng.sample(UnitSphere);
                return Ray {
                    origin: p,
                    dir: glm::vec3(x, y, z),
                };
            };

            let mut dir = (new_p - p).normalize();
            let mut axial_loc = 0.;
            let mut valid = true;

            for i in (0..self.lens_system.surfaces.len()).rev() {
                let surface = &self.lens_system.surfaces[i];
                axial_loc += surface.thickness;
                let next_n_d = if i == 0 {
                    IMAGING_MEDIUM_N_D
                } else {
                    self.lens_system.surfaces[i - 1]
                        .n_d
                        .unwrap_or(IMAGING_MEDIUM_N_D)
                };

                // Find intersection with lens.
                let lens_center = (axial_loc - surface.radius) * self.direction + self.eye;
                let a = dir.dot(&dir);
                let v = p - lens_center;
                let b = 2. * v.dot(&dir);
                let c = v.dot(&v) - surface.radius * surface.radius;
                let discriminant = b * b - 4. * a * c;
                if discriminant < 0. {
                    println!("discriminant less than 0");
                    valid = false;
                    break;
                }
                let t = (-b - (b * b - 4. * a * c).sqrt()) / 2. / a;
                let intersect = p + dir * t;
                let intersect2camera = intersect - self.eye;
                let axial_radius_squared = (intersect2camera
                    - (intersect2camera).dot(&self.direction) * self.direction)
                    .norm_squared();
                if axial_radius_squared > surface.aperture * surface.aperture / 4. {
                    println!("outside of aperture");
                    valid = false;
                    break;
                }

                // Calculate refracted ray.
                let normal = (intersect - lens_center).normalize();
                let sin_theta1 = normal.cross(&dir).norm();
                let sin_theta2 = surface.n_d.unwrap_or(IMAGING_MEDIUM_N_D) / next_n_d * sin_theta1;
                let dir_norm = normal.dot(&dir) * normal;
                let dir_perp = dir - dir_norm;
                let new_dir_perp = sin_theta2 / sin_theta1 * dir_perp;
                dir = (dir_norm + new_dir_perp).normalize();

                // Update ray origin to next surface plane.
                p = intersect;
            }

            if valid {
                break Ray { origin: p, dir };
            }
        }
    }
}
