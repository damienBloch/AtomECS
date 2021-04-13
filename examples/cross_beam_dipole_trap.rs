//! Loading a Sr cross beam dipole trap from center.

extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::atom_sources::central_creator::CentralCreator;
use lib::atom_sources::emit::AtomNumberToEmit;
use lib::atom_sources::mass::{MassDistribution, MassRatio};
use lib::destructor::ToBeDestroyed;
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file::Text;
use lib::output::{file, xyz_file};
use lib::shapes::Cuboid;
use lib::sim_region::{SimulationVolume, VolumeType};
use nalgebra::Vector3;
use specs::{Builder, RunNow, World};
use std::time::Instant;

fn main() {
    let now = Instant::now();

    // Create the simulation world and builder for the ECS dispatcher.
    let mut world = World::new();
    ecs::register_components(&mut world);
    ecs::register_resources(&mut world);
    let mut builder = ecs::create_simulation_dispatcher_builder();

    // Configure simulation output.
    builder = builder.with(
        file::new::<Position, Text>("pos_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel_dipole.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(xyz_file::WriteToXYZFileSystem, "", &[]);

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    world
        .create_entity()
        .with(xyz_file::XYZWriteHelper {
            overwrite: true,
            initialized: false,
            scale_factor: 20000.,
            discard_place: Vector3::new(2., 2., 2.),
            name: format!("{}", "cross_beam_basic_heating_escape"),
        })
        .build();

    // Create dipole laser.
    let power = 10.0;
    let e_radius = 60.0e-6 / (2.0_f64.sqrt());

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::x(),
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::gaussian::GaussianReferenceFrame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
            ellipticity: 0.0,
        })
        .with(laser::gaussian::make_gaussian_rayleigh_range(
            &1064.0e-9,
            &gaussian_beam,
        ))
        .build();

    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: power,
        direction: Vector3::y(),
    };
    world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::dipole_beam::DipoleLight {
            wavelength: 1064.0e-9,
        })
        .with(laser::gaussian::GaussianReferenceFrame {
            x_vector: Vector3::x(),
            y_vector: Vector3::z(),
            ellipticity: 0.0,
        })
        .with(laser::gaussian::make_gaussian_rayleigh_range(
            &1064.0e-9,
            &gaussian_beam,
        ))
        .build();
    // creating the entity that represents the source
    //
    // contains a central creator
    let number_to_emit = 1_000;
    let size_of_cube = 1.0e-5;
    let speed = 0.1; // m/s

    world
        .create_entity()
        .with(CentralCreator::new_uniform_cubic(size_of_cube, speed))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(MassDistribution::new(vec![MassRatio {
            mass: 87.0,
            ratio: 1.0,
        }]))
        .with(AtomicTransition::strontium_red())
        .with(AtomNumberToEmit {
            number: number_to_emit,
        })
        .with(ToBeDestroyed)
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-5 });
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    world
        .create_entity()
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(Cuboid {
            half_width: Vector3::new(0.01, 0.01, 0.01), //(0.1, 0.01, 0.01)
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    let mut switcher_system =
        dipole::transition_switcher::AttachAtomicDipoleTransitionToAtomsSystem;
    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        dispatcher.dispatch(&mut world.res);
        switcher_system.run_now(&world.res);
        world.maintain();
    }

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}