extern crate rayon;
extern crate specs;
use crate::constant;

use crate::laser::intensity_gradient::LaserIntensityGradientSampler;

use specs::{Join, Read, ReadExpect, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use nalgebra::Vector3;

use crate::atom::Force;
use crate::constant::HBAR;
use crate::integrator::Timestep;

pub struct ApplyDipoleForceSystem;

impl<'a> System<'a> for ApplyDipoleForceSystem {
    type SystemData = (
        ReadStorage<'a, LaserIntensityGradientSampler>,
        WriteStorage<'a, Force>,
        ReadExpect<'a, Timestep>,
    );

    fn run(&mut self, (gradient_sampler, mut force, timestep): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut force, &gradient_sampler)
            .par_join()
            .for_each(|(mut force, sampler)| {});
    }
}
