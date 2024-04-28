pub mod lens;

use crate::camera::lens::{Lens, LensSystem};
use crate::lens::IMAGING_MEDIUM_N_D;
use crate::Color;
use glm::vec3;
use rand::distributions::Uniform;
use rand::{rngs::StdRng, Rng};
use rand_distr::{UnitDisc, UnitSphere};

use crate::shape::Ray;

/// A camera that can cast rays into the scene
pub trait Camera: Send + Sync {
    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> (Ray, Color, f64);
}

/// A simple thin-lens perspective camera
#[derive(Clone, Debug)]
pub struct ThinLensCamera {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing (normalized).
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction` (normalized).
    pub up: glm::DVec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,

    /// The camera aperture size and shape
    pub aperture: Option<Aperture>,
}

/// A simple aperture of various shape
#[derive(Clone, Debug)]
pub struct Aperture {
    /// Aperture radius for depth-of-field effects
    pub scale: f64,

    /// Focal distance
    pub focal_distance: f64,

    /// The shape of the aperture
    pub shape: ApertureShape,
}

/// Various shape options for aperture
#[derive(Clone, Debug)]
pub enum ApertureShape {
    /// A circular aperture.
    ///
    /// Represents a circle centered at (0, 0) with radius 1.
    Circle,
    /// A square aperture.
    ///
    /// Equivalent to a 4-point polygon aperture with points at (+/- 1, +/- 1).
    Square,
    /// An aperture with an arbitrary polygon shape.
    ///
    /// The points of the polygon must lie within a [-1, 1] box.
    Poly(Polygon),
}

/// Polygon composed of points
#[derive(Clone, Debug)]
pub struct Polygon {
    pts: Vec<[f64; 2]>,
}

impl Default for ThinLensCamera {
    fn default() -> Self {
        Self {
            eye: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f64::consts::FRAC_PI_6,
            aperture: None,
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
            aperture: None,
        }
    }

    /// Focus the camera on a position, with simulated depth-of-field
    pub fn focus(mut self, focal_point: glm::DVec3, aperture: Option<Aperture>) -> Self {
        self.aperture = aperture.map(|mut aperture| {
            let focal_distance = (focal_point - self.eye).dot(&self.direction);
            aperture.focal_distance = focal_distance;
            aperture
        });
        self
    }
}

impl Camera for ThinLensCamera {
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> (Ray, Color, f64) {
        // cot(f / 2) = depth / radius
        let d = (self.fov / 2.0).tan().recip();
        let right = glm::cross(&self.direction, &self.up).normalize();
        let mut origin = self.eye;
        let mut new_dir = d * self.direction + x * right + y * self.up;
        if let Some(ref aperture) = self.aperture {
            // Depth of field
            let focal_point = origin + new_dir.normalize() * aperture.focal_distance;
            let [x, y]: [f64; 2] = aperture.shape.sample(rng);
            origin += (x * right + y * self.up) * aperture.scale;
            new_dir = focal_point - origin;
        }
        (
            Ray {
                origin,
                dir: new_dir.normalize(),
            },
            vec3(1., 1., 1.),
            1.,
        )
    }
}

/// A physical camera
pub struct PhysicalCamera<L> {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::DVec3,

    /// Width of image sensor.
    pub sensor_width: f64,

    /// Height of image sensor.
    pub sensor_height: f64,

    /// Lens.
    pub lens: L,

    /// Current lens system.
    pub lens_system: LensSystem,
}

/// A physical camera
impl<L: Lens + Default> Default for PhysicalCamera<L> {
    fn default() -> Self {
        let lens = L::default();
        let lens_system = lens.lens_system(4.);
        Self {
            eye: glm::vec3(0.0, -0.5, 7.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            sensor_width: 8.,
            sensor_height: 6.,
            lens,
            lens_system,
        }
    }
}

impl<L: Lens> PhysicalCamera<L> {
    /// Points the camera in the given direction.
    pub fn look_at(&mut self, eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3) {
        self.eye = eye;
        self.direction = (center - eye).normalize();
        self.up = (up - up.dot(&self.direction) * self.direction).normalize();
    }

    /// Focuses the camera on the given point.
    pub fn focus(&mut self, focal_point: glm::DVec3) {
        self.lens_system = self
            .lens
            .lens_system((focal_point - self.eye).dot(&self.direction).abs());
    }
}

enum RgbColor {
    Red,
    Green,
    Blue,
}

impl RgbColor {
    pub fn wavelength(&self) -> f64 {
        match self {
            RgbColor::Red => 656.3e-9,
            RgbColor::Green => 537.7e-9,
            RgbColor::Blue => 486.1e-9,
        }
    }

    pub fn as_vec(&self) -> Color {
        match self {
            RgbColor::Red => vec3(1., 0., 0.),
            RgbColor::Green => vec3(0., 1., 0.),
            RgbColor::Blue => vec3(0., 0., 1.),
        }
    }
}

impl<L: Lens> Camera for PhysicalCamera<L> {
    fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> (Ray, Color, f64) {
        let right = glm::cross(&self.direction, &self.up).normalize();
        let color = match rng.gen_range(0..3) {
            0 => RgbColor::Red,
            1 => RgbColor::Green,
            2 => RgbColor::Blue,
            _ => unreachable!(),
        };

        loop {
            let mut p = self.eye
                + self.sensor_width * x / 2. * right
                + self.sensor_height * y / 2. * self.up;

            let new_p = if let Some(surface) = self.lens_system.surfaces.last() {
                let [x, y]: [f64; 2] = rng.sample(UnitDisc);
                let x = x * surface.aperture / 2.;
                let y = y * surface.aperture / 2.;
                let z = (surface.radius * surface.radius - x * x - y * y).sqrt();
                self.eye
                    + self.direction
                        * (surface.thickness
                            - surface.radius * (surface.radius.abs() - z) / surface.radius.abs())
                    + x * right
                    + y * self.up
            } else {
                let [x, y, z]: [f64; 3] = rng.sample(UnitSphere);
                return (
                    Ray {
                        origin: p,
                        dir: glm::vec3(x, y, z),
                    },
                    color.as_vec(),
                    1. / 3.,
                );
            };

            let mut dir = (new_p - p).normalize();
            let mut axial_loc = 0.;
            let mut valid = true;

            for i in (0..self.lens_system.surfaces.len()).rev() {
                let surface = &self.lens_system.surfaces[i];
                axial_loc += surface.thickness;
                let next_n = if i == 0 {
                    IMAGING_MEDIUM_N_D
                } else {
                    self.lens_system.surfaces[i - 1]
                        .n(color.wavelength())
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
                    valid = false;
                    break;
                }
                let t = (-b
                    + if surface.radius < 0. { -1. } else { 1. } * (b * b - 4. * a * c).sqrt())
                    / 2.
                    / a;
                let intersect = p + dir * t;
                let intersect2camera = intersect - self.eye;
                let axial_radius_squared = (intersect2camera
                    - (intersect2camera).dot(&self.direction) * self.direction)
                    .norm_squared();
                if axial_radius_squared > surface.aperture * surface.aperture / 4. {
                    valid = false;
                    break;
                }

                // Calculate refracted ray.
                let normal = (intersect - lens_center).normalize();
                let sin_theta1 = normal.cross(&dir).norm();
                let sin_theta2 = surface.n(color.wavelength()).unwrap_or(IMAGING_MEDIUM_N_D)
                    / next_n
                    * sin_theta1;
                let dir_norm = normal.dot(&dir) * normal;
                let dir_perp = dir - dir_norm;
                let new_dir_perp = sin_theta2 / sin_theta1 * dir_perp;
                dir = (dir_norm + new_dir_perp).normalize();

                // Update ray origin to next surface plane.
                p = intersect;
            }

            if valid {
                break (Ray { origin: p, dir }, color.as_vec(), 1. / 3.);
            }
        }
    }
}

impl ApertureShape {
    fn sample(&self, rng: &mut StdRng) -> [f64; 2] {
        match self {
            ApertureShape::Circle => rng.sample(UnitDisc),
            ApertureShape::Square => {
                let uniform = Uniform::new_inclusive(-1.0, 1.0);
                let x = rng.sample(uniform);
                let y = rng.sample(uniform);
                [x, y]
            }
            ApertureShape::Poly(ref poly) => {
                let uniform = Uniform::new_inclusive(-1.0, 1.0);
                loop {
                    let x = rng.sample(uniform);
                    let y = rng.sample(uniform);

                    if poly.contains(x, y) {
                        break [x, y];
                    }
                }
            }
        }
    }
}

impl Polygon {
    /// Generate points for a star with n points
    pub fn get_star(n: f64) -> Self {
        // https://math.stackexchange.com/questions/2135982/math-behind-creating-a-perfect-star
        let angle = 2.0 * std::f64::consts::PI / n; // angle = 2pi/n
        let mut pts: Vec<[f64; 2]> = Vec::new();
        for i in 0..n as i64 {
            // outer radius
            let a = angle * i as f64;
            let p_x = a.cos(); // can scale
            let p_y = a.sin();
            pts.push([p_x, p_y]);
            // inner radius
            let i_a = a + std::f64::consts::PI / n;
            let i_x = 0.5 * i_a.cos(); // can scale
            let i_y = 0.5 * i_a.sin();
            pts.push([i_x, i_y]);
        }
        Self { pts }
    }
    /// Generate points for a heart scaled by xscale and yscale
    pub fn get_heart(xscale: f64, yscale: f64) -> Self {
        // https://mathworld.wolfram.com/HeartCurve.html
        let mut pts: Vec<[f64; 2]> = Vec::new();
        for t in (-180..180).step_by(10) {
            let t = t as f64 * std::f64::consts::PI / 180.;
            let x = 16. * t.sin().powi(3);
            let y = 13. * t.cos() - 5. * (2. * t).cos() - 2. * (3. * t).cos() - (4. * t).cos();
            pts.push([x * xscale, y * yscale]);
        }
        Self { pts }
    }

    /// Taken from https://stackoverflow.com/questions/217578/how-can-i-determine-whether-a-2d-point-is-within-a-polygon
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let num_points = self.pts.len();
        let mut i = 0;
        let mut j = num_points - 1;
        let mut c = false;
        while i < num_points {
            let prev_point = self.pts[j];
            let curr_point = self.pts[i];
            if ((curr_point[1] > y) != (prev_point[1] > y))
                && (x
                    < (prev_point[0] - curr_point[0]) * (y - curr_point[1])
                        / (prev_point[1] - curr_point[1])
                        + curr_point[0])
            {
                c = !c;
            }
            j = i;
            i += 1;
        }
        c
    }
}
