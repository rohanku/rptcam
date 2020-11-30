use rand::{rngs::StdRng, Rng};
use rand_distr::UnitCircle;

use super::{HitRecord, Physics, Ray, Shape};
use crate::kdtree::{Bounded, BoundingBox};

/// Represents a glass-shaped surface with height and exp parameters
///
/// Points satisfy the relation y = height * sqrt(x^2 + z^2)^exp, x^2 + z^2 <= 1.
pub struct MonomialSurface {
    /// The height of the surface
    pub height: f64,
    /// The surface exponent
    pub exp: f64,
}

impl Shape for MonomialSurface {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let dist = |t: f64| {
            let x = ray.origin.x + t * ray.dir.x;
            let y = ray.origin.y + t * ray.dir.y;
            let z = ray.origin.z + t * ray.dir.z;
            return y - self.height * (x * x + z * z).powf(self.exp / 2.0); // can make exp / 2 integer to speed up
        };
        let maximize: bool = dist(t_min) < 0.0;
        let t_max: f64 = {
            let mut l: f64 = t_min;
            let mut r: f64 = 10000.0;
            for _ in 0..60 {
                let ml = (2.0 * l + r) / 3.0;
                let mr = (l + 2.0 * r) / 3.0;
                if maximize && dist(ml) < dist(mr) || !maximize && dist(ml) > dist(mr) {
                    l = ml;
                } else {
                    r = mr;
                }
            }
            l
        };
        if (dist(t_min) < 0.0) == (dist(t_max) < 0.0) {
            return false;
        }
        let mut l = t_min;
        let mut r = t_max;
        for _ in 0..60 {
            let m = (l + r) / 2.0;
            if (dist(m) >= 0.0) == maximize {
                r = m;
            } else {
                l = m;
            }
        }
        if r > record.time {
            return false;
        }
        let pos = ray.at(r);
        if pos.x * pos.x + pos.z * pos.z > 1.0 {
            // Check the second equation
            return false;
        }
        record.time = r;

        // TODO: this is valid only for exp = 4, not sure how to do it in general case
        record.normal = glm::normalize(&glm::vec3(
            self.height * 4.0 * pos.x * (pos.x * pos.x + pos.z * pos.z),
            -1.0,
            self.height * 4.0 * pos.z * (pos.x * pos.x + pos.z * pos.z),
        ));

        // The surface is two-sided, so we choose the appropriate normal
        if glm::dot(&record.normal, &ray.dir) > 0.0 {
            record.normal = -record.normal;
        }

        true
    }

    fn sample(&self, _target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64) {
        let [x, z]: [f64; 2] = rng.sample(UnitCircle);
        let pos = glm::vec3(x, self.height * (x * x + z * z).powf(self.exp / 2.), z);
        let mut normal = glm::normalize(&glm::vec3(
            self.height * 4. * pos.x * (pos.x * pos.x + pos.z * pos.z),
            -1.,
            self.height * 4. * pos.z * (pos.x * pos.x + pos.z * pos.z),
        ));
        // Again, only valid for exp = 4
        const AREA: f64 = 6.3406654362; // thanks WolframAlpha, hope I have set up the integrals correctly
        if rng.gen_bool(0.5) == true {
            normal = -normal;
        }
        (pos, normal, 1. / (2. * AREA)) // 2 * AREA because there are two sides
    }
}
impl Physics for MonomialSurface {
    fn closest_point(&self, point: &glm::DVec3) -> glm::DVec3 {
        if glm::length(point) < 1e-12 {
            return *point;
        }
        // Move to the 2d coordinate system
        let px = point.x.hypot(point.z);
        let py = point.y;
        // We want to find a point at which tangent is perpendicular to the vector to our point
        // Do it with binary search
        let mut l: f64;
        let mut r: f64;
        // There can be two such points, so we must choose one
        if px <= 0. {
            l = -1.;
            r = 0.;
        } else {
            l = 0.;
            r = 1.;
        }
        for _ in 0..50 {
            let m: f64 = (l + r) / 2.;
            let tang = glm::vec2(m, 4. * self.height * m.powi(3));
            let vec = glm::vec2(px - m, py - self.height * m.powi(4));
            if tang.dot(&vec) < 0.0 {
                r = m;
            } else {
                l = m;
            }
        }
        let xz = l * glm::normalize(&glm::vec2(point.x, point.z));
        return glm::vec3(
            xz.x,
            self.height * (xz.x.powi(2) + xz.y.powi(2)).powi(2),
            xz.y,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monomial_closest_point_works() {
        let surf = MonomialSurface {
            height: 1.,
            exp: 4.,
        };
        let test_xz = |x: f64, z: f64| {
            let pt = glm::vec3(x, (x.powi(2) + z.powi(2)).powi(2), z);
            assert!(glm::distance(&pt, &surf.closest_point(&pt)) < 1e-3);
        };
        test_xz(0.0, 1.0);
        test_xz(0.0, -1.0);
        test_xz(0.23234, 0.723423);
        test_xz(0.12323, -0.23423);
        test_xz(0.0, 0.00001);
        test_xz(0.0, -0.00001);
        for i in 1..10000 {
            test_xz(0.0, i as f64 / 10000.0);
            test_xz(0.0, -i as f64 / 10000.0);
        }
        test_xz(0.0, 0.0);
        test_xz(0.0, 1e-13);
        test_xz(0.0, 1e-12);
        test_xz(0.0, 1e-11);
        test_xz(0.0, 1e-10);
    }
}

impl Bounded for MonomialSurface {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            p_min: glm::vec3(-1.0, 0.0, -1.0),
            p_max: glm::vec3(1.0, self.height, 1.0),
        }
    }
}