//! Lenses.

use crate::{Aperture, ApertureShape};

/// Refractive index of imaging medium.
pub const IMAGING_MEDIUM_N_D: f64 = 1.;

/// The wavelength of the sodium D line (yellow).
pub const WAVELENGTH_D_LINE: f64 = 589.3e-9;

/// The wavelength of the hydrogen F line (blue).
pub const WAVELENGTH_F_LINE: f64 = 486.1e-9;

/// The wavelength of the hydrogen C line (red).
pub const WAVELENGTH_C_LINE: f64 = 656.3e-9;

/// An object-facing surface of a lens element within a lens system
#[derive(Clone, Debug)]
pub struct LensSurface {
    /// Radius of curvature
    pub radius: f64,
    /// Element thickness
    pub thickness: f64,
    /// Aperture
    pub aperture: Aperture,
    /// Element index of refraction for sodium `d` line
    pub n_d: Option<f64>,
    /// V number, characterizing dispersion.
    pub v_no: f64,
}

/// A lens system
#[derive(Clone, Debug)]
pub struct LensSystem {
    /// Surfaces of lens elements from closest to the object to farthest from the object
    pub surfaces: Vec<LensSurface>,
}

/// A lens
pub trait Lens: Send + Sync {
    /// The minimum distance from the image sensor that can be brought into focus.
    fn focus_min(&self) -> Option<f64>;
    /// The maximum distance from the image sensor that can be brought into focus.
    fn focus_max(&self) -> Option<f64>;

    /// The lens system that brings an object at the given distance from the image sensor into focus.
    ///
    /// If the lens does not support this distance, returns the best valid lens configuration.
    fn lens_system(&self, object_distance: f64) -> LensSystem;
}

/// A single lens
#[derive(Clone, Debug)]
pub struct SingleLens {
    /// Outward-facing radius of curvature.
    ///
    /// Positive radius of curvature indicates a convex surface.
    pub r1: f64,
    /// Inward-facing radius of curvature.
    ///
    /// Positive radius of curvature indicates a convex surface.
    pub r2: f64,
    /// Aperture
    pub aperture: Aperture,
    /// Thickness.
    pub thickness: f64,
    /// Index of refraction at sodium `d` line.
    pub n_d: f64,
    /// V number.
    pub v_no: f64,
}

impl Default for SingleLens {
    fn default() -> Self {
        Self {
            r1: 4.,
            r2: 4.,
            aperture: Aperture {
                scale: 0.035,
                shape: ApertureShape::Circle,
            },
            thickness: 0.01,
            n_d: 1.8,
            v_no: 1000.,
        }
    }
}

impl SingleLens {
    /// Focal length of this [`SingleLens`]
    pub fn focal_length(&self) -> f64 {
        1. / ((self.n_d - 1.)
            * (1. / self.r1 + 1. / self.r2
                - (self.n_d - 1.) * self.thickness / (self.n_d * self.r1 * self.r2)))
    }
}

impl Lens for SingleLens {
    fn focus_max(&self) -> Option<f64> {
        None
    }
    fn focus_min(&self) -> Option<f64> {
        Some(4. * self.focal_length())
    }
    fn lens_system(&self, object_distance: f64) -> LensSystem {
        let object_distance = object_distance.max(4. * self.focal_length());
        let a = 1.;
        let b = -object_distance;
        let c = self.focal_length() * object_distance;
        let discriminant = b * b - 4. * a * c;
        let min_distance = self.thickness / 2. + 0.2;
        let image_distance = if discriminant < 0. {
            min_distance
        } else {
            let opt1 = (-b - discriminant.sqrt()) / 2. / a;
            let opt2 = (-b + discriminant.sqrt()) / 2. / a;
            if opt1 > min_distance {
                opt1
            } else if opt2 > min_distance {
                opt2
            } else {
                min_distance
            }
        };

        LensSystem {
            surfaces: vec![
                LensSurface {
                    radius: self.r1,
                    thickness: self.thickness,
                    aperture: self.aperture.clone(),
                    n_d: Some(self.n_d),
                    v_no: self.v_no,
                },
                LensSurface {
                    radius: -self.r2,
                    thickness: image_distance - self.thickness / 2.,
                    aperture: self.aperture.clone(),
                    n_d: None,
                    v_no: 0.0,
                },
            ],
        }
    }
}

impl LensSurface {
    /// The index of refraction at the given wavelength.
    pub fn n(&self, wavelength: f64) -> Option<f64> {
        if let Some(n_d) = self.n_d {
            let k = (n_d - 1.) / (self.v_no * (WAVELENGTH_F_LINE - WAVELENGTH_C_LINE));
            Some(n_d + k * (wavelength - WAVELENGTH_D_LINE))
        } else {
            None
        }
    }
}

/// An achromatic doublet.
///
/// Lens 1: positive (convex, high vno, low n).
/// Lens 2: negative (concave, low vno, high n),
#[derive(Clone, Debug)]
pub struct AchromaticDoublet {
    v1: f64,
    v2: f64,
    n1: f64,
    n2: f64,
    feq: f64,
    thickness: f64,
    aperture: Aperture,
    r1: f64,
    r2: f64,
    r3: f64,
}

#[derive(Clone, Debug)]
pub struct AchromaticDoubletParams {
    v1: f64,
    v2: f64,
    n1: f64,
    n2: f64,
    feq: f64,
    thickness: f64,
    aperture: Aperture,
    r1: f64,
}

impl Default for AchromaticDoublet {
    fn default() -> Self {
        Self::new(AchromaticDoubletParams {
            v1: 64.17,
            v2: 1.,
            n1: 1.3,
            n2: 1.8,
            feq: 2.5,
            thickness: 0.005,
            aperture: Aperture {
                scale: 0.100,
                shape: ApertureShape::Circle,
            },
            r1: 4.,
        })
    }
}

impl AchromaticDoublet {
    pub fn new(params: AchromaticDoubletParams) -> Self {
        let w1 = 1. / params.v1;
        let w2 = 1. / params.v2;

        let ptot = 1. / params.feq;
        let p1 = ptot / (1. - w1 / w2);
        let p2 = ptot - p1;
        let r2 = 1. / (p1 / (params.n1 - 1.) - 1. / params.r1);
        let r3 = -1. / (p2 / (params.n2 - 1.) - 1. / r2);

        assert!(params.r1 > 0.);
        assert!(r2 > 0.);
        assert!(r3 > 0.);

        Self {
            v1: params.v1,
            v2: params.v2,
            n1: params.n1,
            n2: params.n2,
            feq: params.feq,
            thickness: params.thickness,
            aperture: params.aperture,
            r1: params.r1,
            r2,
            r3,
        }
    }

    /// Focal length of this lens.
    pub fn focal_length(&self) -> f64 {
        self.feq
    }
}

impl Lens for AchromaticDoublet {
    fn focus_max(&self) -> Option<f64> {
        None
    }
    fn focus_min(&self) -> Option<f64> {
        None
    }

    fn lens_system(&self, object_distance: f64) -> LensSystem {
        println!(
            "dist = {object_distance}, r1 = {}, r2 = {}, r3 = {}, feq = {}",
            self.r1, self.r2, self.r3, self.feq
        );
        let object_distance = object_distance.max(4. * self.focal_length());
        let a = 1.;
        let b = -object_distance;
        let c = self.focal_length() * object_distance;
        let discriminant = b * b - 4. * a * c;
        let min_distance = self.thickness + 0.2;
        let image_distance = if discriminant < 0. {
            min_distance
        } else {
            let opt1 = (-b - discriminant.sqrt()) / 2. / a;
            let opt2 = (-b + discriminant.sqrt()) / 2. / a;
            if opt1 > min_distance {
                opt1
            } else if opt2 > min_distance {
                opt2
            } else {
                min_distance
            }
        };

        LensSystem {
            surfaces: vec![
                LensSurface {
                    radius: self.r1,
                    thickness: self.thickness,
                    aperture: self.aperture.clone(),
                    n_d: Some(self.n1),
                    v_no: self.v1,
                },
                LensSurface {
                    radius: -self.r2,
                    thickness: self.thickness,
                    aperture: self.aperture.clone(),
                    n_d: Some(self.n2),
                    v_no: self.v2,
                },
                LensSurface {
                    radius: -self.r3,
                    thickness: image_distance - self.thickness,
                    aperture: self.aperture.clone(),
                    n_d: None,
                    v_no: 0.0,
                },
            ],
        }
    }
}
