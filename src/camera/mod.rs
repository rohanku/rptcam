pub mod lens;

use crate::camera::lens::{Lens, LensSystem};
use crate::lens::IMAGING_MEDIUM_N_D;
use rand::{rngs::StdRng, Rng};
use rand_distr::{UnitDisc, UnitSphere};

use crate::shape::Ray;

pub trait Camera {
    /// Perspective camera looking at a point, with a given field of view
    fn look_at(&mut self, eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64);

    /// Focus the camera on a position
    fn focus(&mut self, focal_point: glm::DVec3, aperture: f64);

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

impl Camera for ThinLensCamera {
    fn look_at(&mut self, eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64) {
        self.eye = eye;
        self.direction = (center - eye).normalize();
        self.up = (up - up.dot(&self.direction) * self.direction).normalize();
        self.fov = fov;
    }

    fn focus(&mut self, focal_point: glm::DVec3, aperture: f64) {
        self.focal_distance = (focal_point - self.eye).dot(&self.direction);
        self.aperture = aperture;
    }

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

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,

    /// Aperture radius for depth-of-field effects
    pub aperture: f64,

    /// Width of image sensor (mm).
    pub sensor_width: f64,

    /// Height of image sensor (mm).
    pub sensor_height: f64,

    /// Lens.
    pub lens: L,

    /// Current lens system.
    pub lens_system: LensSystem,
}

impl<L: Lens> Camera for PhysicalCamera<L> {
    fn look_at(&mut self, eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64) {
        self.eye = eye;
        self.direction = (center - eye).normalize();
        self.up = (up - up.dot(&self.direction) * self.direction).normalize();
        self.fov = fov;
    }

    fn focus(&mut self, focal_point: glm::DVec3, aperture: f64) {
        self.lens_system = self
            .lens
            .lens_system((focal_point - self.eye).dot(&self.direction).abs());
        self.aperture = aperture;
    }

    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray {
        let right = glm::cross(&self.direction, &self.up).normalize();

        let mut p =
            self.eye + self.sensor_width * x / 2. * right + self.sensor_height * x / 2. * self.up;

        let new_p = if let Some(surface) = self.lens_system.surfaces.last() {
            let [x, y]: [f64; 2] = rng.sample(UnitDisc);
            self.eye + self.direction * surface.thickness + x * right * surface.aperture / 2.
        } else {
            let [x, y, z]: [f64; 3] = rng.sample(UnitSphere);
            return Ray {
                origin: p,
                dir: glm::vec3(x, y, z),
            };
        };

        let mut new_dir = new_p - p;
        let mut axial_loc = 0;
        p = new_p;

        for i in (0..self.lens_system.surfaces.len()).rev() {
            let surface = &self.lens_system.surfaces[i];
            let next_index = if i == 0 {
                IMAGING_MEDIUM_N_D
            } else {
                self.lens_system.surfaces[i - 1]
                    .n_d
                    .unwrap_or(IMAGING_MEDIUM_N_D)
            };
        }

        Ray {
            origin: p,
            dir: new_dir.normalize(),
        }
    }
}
