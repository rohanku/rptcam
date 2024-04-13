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
pub trait Lens {
    /// The minimum distance from the image sensor that can be brought into focus (mm).
    fn focus_min(&self) -> f64;
    /// The maximum distance from the image sensor that can be brought into focus (mm).
    fn focus_max(&self) -> f64;

    /// The lens system that brings an object at the given distance from the image sensor into focus.
    ///
    /// If the lens does not support this distance, returns the best valid lens configuration.
    fn lens_system(&self, object_distance: f64) -> LensSystem;
}
