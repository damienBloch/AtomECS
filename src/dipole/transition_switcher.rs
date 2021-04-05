extern crate rayon;
extern crate specs;
use crate::atom::{Atom, AtomicTransition, Kind};
use crate::destructor::ToBeDestroyed;
use crate::dipole::atom::AtomicDipoleTransition;
use crate::dipole::dipole_beam::DipoleLight;
use crate::laser::cooling::CoolingLight;
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System};
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
