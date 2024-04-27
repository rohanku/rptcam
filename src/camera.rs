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
        let _polygon = Polygon {
            pts: Vec::from([[0.0, 0.0], [0.5, 0.0], [1.0, 0.5], [1.0, 1.0]]),
        };
        let _triangle = Polygon {pts: vec![[0.0, 1.0], [-1.0, 0.0], [1.0, 0.0]],};
        let _star = Polygon {pts: Polygon::get_star(5.0)}; // number of points
        let _heart =  Polygon {pts: Polygon::get_heart(0.05, 0.05)}; // scale <0.1
        self.aperture = Some(Aperture {
            scale: aperture,
            focal_distance,
            shape: ApertureShape::Poly(_star), // change shape
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
    /// Generate points for a star with n points
    pub fn get_star(n: f64) -> Vec<[f64; 2]> {
        // https://math.stackexchange.com/questions/2135982/math-behind-creating-a-perfect-star
        let angle = 2.0*std::f64::consts::PI/n; // angle = 2pi/n
        let mut pts : Vec<[f64; 2]> = Vec::new();
        for i in 0..n as i64 {
            // outer radius
            let a = angle*i as f64;
            let p_x = a.cos(); // can scale
            let p_y = a.sin();
            pts.push([p_x, p_y]);
            // inner radius
            let i_a = a+std::f64::consts::PI/n;
            let i_x = 0.5*i_a.cos(); // can scale
            let i_y = 0.5*i_a.sin();
            pts.push([i_x, i_y]);
        }
        pts
    }
    /// Generate points for a heart scaled by xscale and yscale
    pub fn get_heart(xscale: f64, yscale: f64) -> Vec<[f64; 2]> {
        // https://mathworld.wolfram.com/HeartCurve.html
        let mut pts : Vec<[f64; 2]> = Vec::new();
        for t in (-180..180).step_by(10) {
            let t = t as f64*std::f64::consts::PI/180.;
            let x = (16. * t.sin().powi(3)) as f64;
            let y = (13. * t.cos() - 5.*(2.*t).cos() - 2.*(3.*t).cos() - (4.*t).cos()) as f64;
            pts.push([x*xscale,y*yscale]);   
        }
        // println!("{:?}", pts);
        pts
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
            if ((curr_point[1]>y) != (prev_point[1]>y)) &&
                (x < (prev_point[0]-curr_point[0]) * (y-curr_point[1]) / (prev_point[1]-curr_point[1]) + curr_point[0]) {
                    c = !c;

            }
            j = i;
            i += 1;
        }
        return c;
    }
}