extern crate rayon;
extern crate specs;
use crate::atom::{Atom, AtomicTransition, Kind};
use crate::constant::C;
use crate::destructor::ToBeDestroyed;
use crate::dipole::atom::AtomicDipoleTransition;
use crate::dipole::dipole_beam::DipoleLight;
use crate::integrator::Timestep;
use crate::laser::cooling::CoolingLight;
use crate::laser::gaussian::GaussianBeam;
use specs::ReadExpect;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage};
extern crate nalgebra;

pub struct DisableMOTBeamsSystem;

impl<'a> System<'a> for DisableMOTBeamsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, DipoleLight>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ents, cooling, dipole, updater): Self::SystemData) {
        for (entity, _cooling, _dipole) in (&ents, &cooling, !&dipole).join() {
            updater.insert(entity, ToBeDestroyed);
        }
    }
}

pub struct AttachAtomicDipoleTransitionToAtomsSystem;

impl<'a> System<'a> for AttachAtomicDipoleTransitionToAtomsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Atom>,
        ReadStorage<'a, AtomicTransition>,
        ReadStorage<'a, AtomicDipoleTransition>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (ents, atom, atomic_transition, atomic_dipole_transition, updater): Self::SystemData,
    ) {
        for (entity, _atom, atominfo, _atomdipole_info) in
            (&ents, &atom, &atomic_transition, !&atomic_dipole_transition).join()
        {
            updater.insert(
                entity,
                match atominfo.kind {
                    Kind::Rubidium => AtomicDipoleTransition::rubidium(),
                    Kind::Strontium => AtomicDipoleTransition::strontium(),
                    Kind::StrontiumRed => AtomicDipoleTransition::strontium(),
                    Kind::Erbium => AtomicDipoleTransition::erbium(),
                    Kind::Erbium401 => AtomicDipoleTransition::erbium_401(),
                },
            );
        }
    }
}

pub struct MOTRelativePowerRampRate {
    /// factor that is applied every second to the power (i.e. 0.9 means reduction by 10% every second)
    pub relative_rate: f64,
}

pub struct MOTAbsoluteDetuningRampRate {
    /// subtracting this amount every second, in Hz
    pub absolute_rate: f64,
}

pub struct DipoleRelativePowerRampRate {
    /// factor that is applied every second to the power (i.e. 0.9 means reduction by 10% every second)
    pub relative_rate: f64,
}

pub struct RampMOTBeamsSystem;

impl<'a> System<'a> for RampMOTBeamsSystem {
    type SystemData = (
        WriteStorage<'a, CoolingLight>,
        WriteStorage<'a, GaussianBeam>,
        ReadExpect<'a, MOTRelativePowerRampRate>,
        ReadExpect<'a, MOTAbsoluteDetuningRampRate>,
        ReadExpect<'a, Timestep>,
    );

    fn run(
        &mut self,
        (mut cooling_light, mut gaussian_beam, power_rate, detuning_rate, timestep): Self::SystemData,
    ) {
        // convert rate per second to rate per step
        let power_rate_factor = power_rate.relative_rate.powf(timestep.delta);
        let detuning_rate_summand = timestep.delta * detuning_rate.absolute_rate;
        use rayon::prelude::ParallelIterator;
        use specs::ParJoin;
        (&mut cooling_light, &mut gaussian_beam)
            .par_join()
            .for_each(|(mut cooling, mut gaussian)| {
                cooling.wavelength = C / (C / cooling.wavelength + detuning_rate_summand);
                gaussian.power = gaussian.power * power_rate_factor;
            });
    }
}
