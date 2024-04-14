//!

/// Refractive index of imaging medium.
pub const IMAGING_MEDIUM_N_D: f64 = 1.;

/// An object-facing surface of a lens element within a lens system
pub struct LensSurface {
    /// Radius of curvature (mm)
    pub radius: f64,
    /// Element thickness (mm)
    pub thickness: f64,
    /// Element index of refraction for sodium `d` line (587.6 nm)
    pub n_d: Option<f64>,
    /// Aperture diameter (mm)
    pub aperture: f64,
}

/// A lens system
pub struct LensSystem {
    /// Surfaces of lens elements from closest to the object to farthest from the object
    pub surfaces: Vec<LensSurface>,
}

/// A lens
pub trait Lens: Send + Sync {
    /// The minimum distance from the image sensor that can be brought into focus (mm).
    fn focus_min(&self) -> Option<f64>;
    /// The maximum distance from the image sensor that can be brought into focus (mm).
    fn focus_max(&self) -> Option<f64>;

    /// The lens system that brings an object at the given distance from the image sensor into focus.
    ///
    /// If the lens does not support this distance, returns the best valid lens configuration.
    fn lens_system(&self, object_distance: f64) -> LensSystem;
}

pub struct SingleLens {
    pub r1: f64,
    pub r2: f64,
    pub aperture: f64,
    pub thickness: f64,
    pub n_d: f64,
}

impl Default for SingleLens {
    fn default() -> Self {
        Self {
            r1: 4.,
            r2: 4.,
            aperture: 2.,
            thickness: 0.01,
            n_d: 1.6,
        }
    }
}

impl SingleLens {
    fn focal_length(&self) -> f64 {
        (self.n_d - 1.) * (1. / self.r1 + 1. / self.r2)
    }
}

impl Lens for SingleLens {
    fn focus_max(&self) -> Option<f64> {
        None
    }
    fn focus_min(&self) -> Option<f64> {
        Some((2. * self.focal_length()).sqrt())
    }
    fn lens_system(&self, object_distance: f64) -> LensSystem {
        let a = 1.;
        let b = -object_distance;
        let c = self.focal_length();
        let discriminant = b * b - 4. * a * c;
        let image_distance = if discriminant < 0. {
            self.thickness / 2. + 0.1
        } else {
            (self.thickness / 2. + 0.1).max((-b + discriminant.sqrt()) / 2. / a)
        };

        LensSystem {
            surfaces: vec![
                LensSurface {
                    radius: self.r1,
                    thickness: self.thickness,
                    n_d: Some(self.n_d),
                    aperture: self.aperture,
                },
                LensSurface {
                    radius: -self.r2,
                    thickness: image_distance - self.thickness / 2.,
                    n_d: None,
                    aperture: self.aperture,
                },
            ],
        }
    }
}
