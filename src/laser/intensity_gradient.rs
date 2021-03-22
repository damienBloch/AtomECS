//! Calculation of the intensity of CoolingLight entities at a specific position
//!

// This file exists because - in the spirit of keeping things general - I thought that the intensity sampler should not be in
// gaussian.rs since other beam profiles (although they're less common) should not be excluded.

extern crate rayon;
extern crate specs;

use super::gaussian::{get_gaussian_beam_intensity_gradient, GaussianBeam, GaussianReferenceFrame};
use crate::atom::Position;
use crate::laser::gaussian::GaussianRayleighRange;
use nalgebra::Vector3;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

/// Represents the laser intensity at the position of the atom with respect to a certain laser beam
#[derive(Clone, Copy)]
pub struct LaserIntensityGradientSampler {
    /// Intensity in SI units of W/m^2
    pub gradient: Vector3<f64>,
}

impl Default for LaserIntensityGradientSampler {
    fn default() -> Self {
        LaserIntensityGradientSampler {
            /// Intensity in SI units of W/m^2
            gradient: Vector3::new(f64::NAN, f64::NAN, f64::NAN),
        }
    }
}

impl Component for LaserIntensityGradientSampler {
    type Storage = VecStorage<Self>;
}

pub struct SampleLaserIntensityGradientSystem;

impl<'a> System<'a> for SampleLaserIntensityGradientSystem {
    type SystemData = (
        ReadStorage<'a, GaussianBeam>,
        ReadStorage<'a, GaussianRayleighRange>,
        ReadStorage<'a, GaussianReferenceFrame>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, LaserIntensityGradientSampler>,
    );

    fn run(
        &mut self,
        (gaussian, rayleigh_range, reference_frame, pos, mut sampler): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (beam, rayleigh, reference) in (&gaussian, &rayleigh_range, &reference_frame).join() {
            (&pos, &mut sampler).par_join().for_each(|(pos, sampler)| {
                sampler.gradient =
                    get_gaussian_beam_intensity_gradient(beam, pos, rayleigh, reference);
            });
        }
    }
}
