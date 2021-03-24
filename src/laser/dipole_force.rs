extern crate rayon;
extern crate specs;
use crate::constant;

use crate::laser::intensity_gradient::LaserIntensityGradientSamplers;

use specs::{Join, ReadStorage, System, WriteStorage};
extern crate nalgebra;
use crate::atom::AtomicTransition;
use crate::laser::dipole_beam::{DipoleLight, DipoleLightIndex};
use nalgebra::Vector3;

use crate::atom::Force;

pub struct ApplyDipoleForceSystem;

impl<'a> System<'a> for ApplyDipoleForceSystem {
    type SystemData = (
        ReadStorage<'a, DipoleLight>,
        ReadStorage<'a, DipoleLightIndex>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, LaserIntensityGradientSamplers>,
        WriteStorage<'a, Force>,
    );

    fn run(
        &mut self,
        (dipole_light, dipole_index,atomic_transition, gradient_sampler, mut force): Self::SystemData,
    ) {
        use rayon::prelude::*;
        use specs::ParJoin;

        (&mut force, &atomic_transition, &gradient_sampler)
            .par_join()
            .for_each(|(mut force, atominfo, sampler)| {
                let prefactor = -3. * constant::PI * constant::C.powf(2.0)
                    / (2. * constant::PI * atominfo.frequency).powf(3.0)
                    * atominfo.linewidth;
                let mut temp_force_coeff = Vector3::new(0.0, 0.0, 0.0);
                for (index, dipole) in (&dipole_index, &dipole_light).join() {
                    temp_force_coeff = temp_force_coeff
                        - (1. / (atominfo.frequency - dipole.frequency())
                            + 1. / (atominfo.frequency + dipole.frequency()))
                            * sampler.contents[index.index].gradient;
                }
                force.force = force.force + prefactor * temp_force_coeff;
            });
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    extern crate specs;
    use assert_approx_eq::assert_approx_eq;
    use specs::{Builder, RunNow, World};
    extern crate nalgebra;
    use nalgebra::Vector3;

    #[test]
    fn test_sample_laser_intensity_gradient_system() {
        let mut test_world = World::new();

        test_world.register::<DipoleLightIndex>();
        test_world.register::<DipoleLight>();
        test_world.register::<Force>();
        test_world.register::<LaserIntensityGradientSamplers>();
        test_world.register::<AtomicTransition>();

        test_world
            .create_entity()
            .with(DipoleLightIndex {
                index: 0,
                initiated: true,
            })
            .with(DipoleLight {
                wavelength: 1064.0e-9,
            })
            .build();

        let transition = AtomicTransition::strontium();
        let atom1 = test_world
            .create_entity()
            .with(Force {
                force: Vector3::new(0.0, 0.0, 0.0),
            })
            .with(LaserIntensityGradientSamplers {
                contents: [crate::laser::intensity_gradient::LaserIntensityGradientSampler {
                    gradient: Vector3::new(0.0, 1.0, -2.0),
                }; crate::laser::COOLING_BEAM_LIMIT],
            })
            .with(transition)
            .build();
        let mut system = ApplyDipoleForceSystem;
        system.run_now(&test_world.res);
        test_world.maintain();
        let sampler_storage = test_world.read_storage::<Force>();
        let sim_result_force = sampler_storage.get(atom1).expect("Entity not found!").force;

        let actual_force = 3. * constant::PI * constant::C.powf(2.0)
            / (2. * (2. * constant::PI * transition.frequency).powf(3.0))
            * transition.linewidth
            * (1. / (transition.frequency - 1064.0e-9) + 1. / (transition.frequency + 1064.0e-9))
            * Vector3::new(0.0, 1.0, -2.0);

        assert_approx_eq!(actual_force[0], sim_result_force[0], 1e+8_f64);
        assert_approx_eq!(actual_force[1], sim_result_force[1], 1e+8_f64);
        assert_approx_eq!(actual_force[2], sim_result_force[2], 1e+8_f64);
    }
}
