extern crate atomecs as lib;
extern crate nalgebra;
use atomecs::laser::force::EmissionForceOption;
use atomecs::laser::photons_scattered::ScatteringFluctuationsOption;
use lib::atom;
use lib::atom::{AtomicTransition, Position, Velocity};
use lib::ecs;
use lib::integrator::Timestep;
use lib::laser::cooling::CoolingLight;
use lib::laser::gaussian::GaussianBeam;
use lib::magnetic::quadrupole::QuadrupoleField3D;
use lib::output::file;
use lib::output::file::Text;
use nalgebra::Vector3;
use specs::Component;
use specs::Join;
use specs::ReadStorage;
use specs::RunNow;
use specs::System;
use specs::VecStorage;
use specs::WriteStorage;
use specs::{Builder, World};
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
        file::new::<Position, Text>("pos.txt".to_string(), 100),
        "",
        &[],
    );
    builder = builder.with(
        file::new::<Velocity, Text>("vel.txt".to_string(), 100),
        "",
        &[],
    );

    let mut dispatcher = builder.build();
    dispatcher.setup(&mut world.res);

    // BEGIN MOT PART

    world
        .create_entity()
        .with(QuadrupoleField3D::gauss_per_cm(1.0, Vector3::z()))
        .with(Position {
            pos: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    let detuning = -0.12; //MHz
    let power = 0.1; //W total power of all Lasers together
    let radius = 1.0e-2 / (2.0 * 2.0_f64.sqrt()); // 10mm 1/e^2 diameter

    // Horizontal beams along z
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.0,
            direction: -Vector3::z(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            -1,
        ))
        .build();

    // Angled vertical beams
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::x(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: -Vector3::x(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: Vector3::y(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world
        .create_entity()
        .with(GaussianBeam {
            intersection: Vector3::new(0.0, 0.0, 0.0),
            e_radius: radius,
            power: power / 6.,
            direction: -Vector3::y(),
        })
        .with(CoolingLight::for_species(
            AtomicTransition::strontium_red(),
            detuning,
            1,
        ))
        .build();
    world.add_resource(EmissionForceOption::default());
    world.add_resource(ScatteringFluctuationsOption::default());

    world
        .create_entity()
        .with(atom::Force::new())
        .with(atom::Position {
            pos: Vector3::new(0.0e-4, 0.0e-4, 0.0e-4),
        })
        .with(atom::Velocity {
            vel: Vector3::new(0.0, 0.0, 0.0),
        })
        .with(lib::atom::AtomicTransition::strontium_red())
        .with(atom::Atom)
        .with(lib::initiate::NewlyCreated)
        .build();

    // Define timestep
    world.add_resource(Timestep { delta: 1.0e-6 });

    pub struct ComponentSummer {
        pub sum: Vector3<f64>,
    }

    impl Component for ComponentSummer {
        type Storage = VecStorage<Self>;
    }

    world.register::<ComponentSummer>();
    let summer = world
        .create_entity()
        .with(ComponentSummer {
            sum: Vector3::new(0.0, 0.0, 0.0),
        })
        .build();

    pub struct CheckComponentSystem;

    impl<'a> System<'a> for CheckComponentSystem {
        type SystemData = (
            ReadStorage<'a, lib::atom::Force>,
            WriteStorage<'a, ComponentSummer>,
        );
        fn run(&mut self, (rate_coefficients, mut summer): Self::SystemData) {
            for sum in (&mut summer).join() {
                for rate in (&rate_coefficients).join() {
                    sum.sum = sum.sum
                        + Vector3::new(
                            rate.force[0].abs(),
                            rate.force[1].abs(),
                            rate.force[2].abs(),
                        );
                }
                //println!("temp1 {}", sum.sum);
            }
        }
    }

    let mut system = CheckComponentSystem;
    // Run the simulation for a number of steps.
    for _i in 0..100_000 {
        dispatcher.dispatch(&mut world.res);
        system.run_now(&world.res);
        world.maintain();
    }
    let sampler_storage = world.read_storage::<ComponentSummer>();
    println!(
        "{}",
        sampler_storage.get(summer).expect("entity not found").sum
    );

    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}
