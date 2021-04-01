extern crate rayon;
extern crate specs;
use crate::atom::AtomicTransition;
use crate::dipole::dipole_beam::DipoleLight;
use crate::laser::cooling::CoolingLight;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System};
extern crate nalgebra;

pub struct SwitchTransitionSystem;

impl<'a> System<'a> for SwitchTransitionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, AtomicTransition>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, atomic_transition, updater): Self::SystemData) {
        for (ent, _atomic_transition) in (&ent, &atomic_transition).join() {
            updater.remove::<AtomicTransition>(ent);
            updater.insert(ent, AtomicTransition::strontium());
        }
    }
}

pub struct DisableMOTBeamsSystem;

impl<'a> System<'a> for DisableMOTBeamsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, CoolingLight>,
        ReadStorage<'a, DipoleLight>,
    );

    fn run(&mut self, (ents, cooling, dipole): Self::SystemData) {
        for (entity, _cooling, _dipole) in (&ents, &cooling, !&dipole).join() {
            ents.delete(entity).expect("Could not delete beam entity!");
        }
    }
}
