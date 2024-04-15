use rand::{rngs::StdRng, Rng};
use rand::distributions::Uniform;
use rand_distr::UnitDisc;

use crate::shape::Ray;

/// A simple thin-lens perspective camera
#[derive(Clone, Debug)]
pub struct Camera {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::DVec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,

    /// The camera aperture size and shape
    pub aperture: Option<Aperture>,
}

#[derive(Clone, Debug)]
pub struct Aperture {
    /// Aperture radius for depth-of-field effects
    pub scale: f64,

    /// Focal distance
    pub focal_distance: f64,

    /// The shape of the aperture
    pub shape: ApertureShape,
}

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

#[derive(Clone, Debug)]
pub struct Polygon {
    pts: Vec<[f64; 2]>,
}

impl Default for Camera {
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

impl Camera {
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
    pub fn focus(mut self, focal_point: glm::DVec3, aperture: f64) -> Self {
        let focal_distance = (focal_point - self.eye).dot(&self.direction);
        let polygon = Polygon {
            pts: Vec::from([[0.0, 0.0], [0.5, 0.0], [1.0, 0.5], [1.0, 1.0]]),
        };
        self.aperture = Some(Aperture {
            scale: aperture,
            focal_distance,
            shape: ApertureShape::Poly(polygon),
            // shape: ApertureShape::Circle,
        });
        self
    }

    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    pub fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray {
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
        Ray {
            origin,
            dir: new_dir.normalize(),
        }
    }
}

impl ApertureShape {
    fn sample(&self, rng: &mut StdRng) -> [f64; 2] {
        match self {
            ApertureShape::Circle => {
                rng.sample(UnitDisc)
            }
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
    /// Taken from https://stackoverflow.com/questions/217578/how-can-i-determine-whether-a-2d-point-is-within-a-polygon
    /// Not tested yet
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let numPoints = self.pts.len();
        let mut i = 0;
        let mut j = numPoints - 1;
        let mut c = false;
        while i < numPoints {
            let prevPoint = self.pts[j];
            let currPoint = self.pts[i];
            if ( ((currPoint[1]>y) != (prevPoint[1]>y)) &&
                (x < (prevPoint[0]-currPoint[0]) * (y-currPoint[1]) / (prevPoint[1]-currPoint[1]) + currPoint[0]) ) {
                    c = !c;

            }
            j = i;
            i += 1;
        }
        return c;
    }
}