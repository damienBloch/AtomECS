//! A set of modules used for integration the motion of laser-cooled atoms.

#[macro_use]
extern crate specs_derive;
pub mod atom;
pub mod atom_sources;
pub mod constant;
pub mod destructor;
pub mod detector;
pub mod ecs;
pub mod fileinput;
pub mod gravity;
pub mod initiate;
pub mod integrator;
pub mod laser;
pub mod magnetic;
pub mod maths;
pub mod optimization;
pub mod output;
pub mod simulation_templates;
pub mod tests;
pub mod sim_region;