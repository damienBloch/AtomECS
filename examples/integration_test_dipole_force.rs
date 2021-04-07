//! Loading a Sr cross beam dipole trap from center.

extern crate atomecs as lib;
extern crate nalgebra;
use crate::lib::laser::force::EmissionForceOption;
use atomecs::laser::photons_scattered::ScatteringFluctuationsOption;
use lib::atom;
use lib::atom::{Position, Velocity};
use lib::dipole;
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser;
use lib::laser::gaussian::GaussianBeam;
use lib::output::file::Text;
use lib::output::{file, xyz_file};
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
            name: format!("{}", "cross_beam_transition_exp"),
        })
        .build();
    // BEGIN MOT PART

    // Horizontal beams along z
    // END MOT part

    // Create dipole laser.
    let power = 10.0;
    let e_radius = 100.0e-6 / (2.0_f64.sqrt());

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

    world.add_resource(EmissionForceOption::default());
    world.add_resource(ScatteringFluctuationsOption::default());

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });
    // Use a simulation bound so that atoms that escape the capture region are deleted from the simulation
    let atom1 = world
        .create_entity()
        .with(atom::Mass { value: 87.0 })
        .with(atom::Force::new())
        .with(atom::Position {
            pos: Vector3::new(100.0e-6, 10.0e-6, 0.0),
        })
        .with(atom::AtomicTransition::strontium_red())
        .with(atom::Atom)
        .with(lib::initiate::NewlyCreated)
        .build();
    for _i in 0..1 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    let mut switcher_system =
        dipole::transition_switcher::AttachAtomicDipoleTransitionToAtomsSystem;
    switcher_system.run_now(&world.res);

    for _i in 0..3 {
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }

    let force = world.read_storage::<atom::Force>();
    let sim_result_force = force.get(atom1).expect("Entity not found!").force / (87.0);
    let position = world.read_storage::<atom::Position>();
    let sim_result_position = position.get(atom1).expect("Entity not found!").pos;

    println!(
        "force is: {} \n and pos is: {}",
        sim_result_force, sim_result_position
    );
    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
