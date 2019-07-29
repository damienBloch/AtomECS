extern crate magneto_optical_trap as lib;
//use lib::*;

//mod lib;

use lib::constant as constant;
use lib::constant::PI as PI;
use lib::integrator::{Timestep,Step};
use lib::atom::{Mass,Position,Velocity,Force,RandKick};
use lib::initiate::AtomInfo;
use lib::update::*;
use lib::laser::*;
use lib::magnetic::*;
use lib::initiate::atom_create::{AtomCreate,Oven,AtomInitiateMot};
use lib::integrator::EulerIntegrationSystem;
use specs::{World,Builder,DispatcherBuilder,RunNow};
use lib::output::{PrintOutput,Detector,DetectingAtom,PrintDetect,AtomOuput};
fn main() {
	mot_2d_plus();
}

fn mot_2d_plus(){
   // create the world
   let mut exp_mot = World::new();
   
	// create the resources and component, and entities for experimental setup
	exp_mot.register::<Velocity>();
	exp_mot.register::<Position>();
	exp_mot.register::<Oven>();
	exp_mot.register::<Force>();
	exp_mot.register::<AtomInfo>();
	exp_mot.register::<Laser>();
	exp_mot.register::<MagneticFieldSampler>();
	exp_mot.register::<InteractionLaserALL>();
	exp_mot.register::<QuadrupoleField3D>();
	exp_mot.register::<RandKick>();
	exp_mot.register::<Detector>();
	
	//component for the experiment
	let rb_atom = AtomInfo{
	mup:constant::MUP,
	mum:constant::MUM,
	muz:constant::MUZ,
	frequency:constant::ATOMFREQUENCY,
	gamma:constant::TRANSWIDTH
	};
	exp_mot.add_resource(Step{n:0});
	exp_mot.add_resource(AtomOuput{number_of_atom:0,total_velocity:[0.,0.,0.]});
	let mag= QuadrupoleField3D{
		gradient:0.002
	};
	exp_mot.create_entity().with(mag).with(Position{pos:[0.,0.,0.]}).build();
	// adding all six lasers
	let laser_1 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,2.0*PI/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:1,
	};
		let laser_2 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,-2.0*PI/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		
		index:2,
	};
		let laser_3 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,2.0*PI/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:3,
	};
		let laser_4 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,-2.0*PI/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:4,
	};
		let laser_5 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[2.0*PI/(461e-9),0.,0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:5,
	};

	//six laser introduced
	exp_mot.create_entity().with(laser_1).build();
	exp_mot.create_entity().with(laser_2).build();
	exp_mot.create_entity().with(laser_3).build();
	exp_mot.create_entity().with(laser_4).build();
	exp_mot.create_entity().with(laser_5).build();
	//detector introduced
	
	exp_mot.create_entity().with(Detector{centre:[0.2,0.,0.],range:[0.05,0.1,0.1]}).build();
	
	exp_mot.add_resource(Timestep{t:1e-6});
	// initiate
		// build a oven
	exp_mot.create_entity().with(Oven{temperature:200.,position:[0.0,0.0,0.0],direction:[1e-6,1e-6,1.],number:100,size:[1e-2,1e-2,1e-2]})
	.with(rb_atom)
	.with(Mass{value:87.})
	.build();
		// initiator dispatched
	let mut init_dispatcher=DispatcherBuilder::new()
			.with(AtomCreate,"atomcreate",&[])
      	.build();
		
	//init_dispatcher.setup(&mut exp_MOT.res);
	init_dispatcher.dispatch(&mut exp_mot.res);
	exp_mot.maintain();
	//two initiators cannot be dispatched at the same time apparently for some unknown reason
	let mut init_dispatcher2=DispatcherBuilder::new().with(AtomInitiateMot, "initiate", &[]).build();
	init_dispatcher2.dispatch(&mut exp_mot.res);
	// run loop
	let mut runner=DispatcherBuilder::new().
	with(UpdateLaser,"updatelaser",&[]).
	with(ClearMagneticFieldSamplerSystem,"clear",&[]).
	with(Sample3DQuadrupoleFieldSystem,"updatesampler",&[]).
	with(CalculateMagneticFieldMagnitudeSystem,"magnitudecal",&["updatesampler"]).
	with(UpdateInteractionLaser,"updateinter",&["updatelaser","updatesampler","magnitudecal"]).
	with(UpdateRandKick,"update_kick",&["updateinter"]).
	with(UpdateForce,"updateforce",&["update_kick","updateinter"]).
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutput,"print",&["updatepos"]).
	with(DetectingAtom,"detect",&["updatepos"]).build();
	runner.setup(&mut exp_mot.res);
	for _i in 0..10000{
		runner.dispatch(&mut exp_mot.res);
		exp_mot.maintain();
		//println!("t{}",time);
	}
	let mut print_detect = PrintDetect;
	print_detect.run_now(&exp_mot.res);	
}

#[allow(dead_code)]
fn mot_3d(){
   // create the world
   let mut exp_mot = World::new();
   
	// create the resources and component, and entities for experimental setup
	exp_mot.register::<Velocity>();
	exp_mot.register::<Position>();
	exp_mot.register::<Oven>();
	exp_mot.register::<Force>();
	exp_mot.register::<AtomInfo>();
	exp_mot.register::<Laser>();
	exp_mot.register::<MagneticFieldSampler>();
	exp_mot.register::<InteractionLaserALL>();
	exp_mot.register::<QuadrupoleField3D>();
	exp_mot.register::<RandKick>();
	
	//component for the experiment
	let rb_atom = AtomInfo{
	mup:constant::MUP,
	mum:constant::MUM,
	muz:constant::MUZ,
	frequency:constant::ATOMFREQUENCY,
	gamma:constant::TRANSWIDTH
	};
	exp_mot.add_resource(Step{n:0});
	let mag= QuadrupoleField3D{gradient:0.002};
	exp_mot.create_entity().with(mag).build();
	// adding all six lasers
	let laser_1 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,2.0*PI/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:1,
	};
		let laser_2 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,0.0,-2.0*PI/(461e-9)],
		polarization:-1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		
		index:2,
	};
		let laser_3 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,2.0*PI/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:3,
	};
		let laser_4 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[0.0,-2.0*PI/(461e-9),0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:4,
	};
		let laser_5 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[2.0*PI/(461e-9),0.,0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:5,
	};
		let laser_6 = Laser{
		centre:[0.,0.,0.],
		wavenumber:[-2.0*PI/(461e-9),0.,0.],
		polarization:1.,
		power:10.,
		std:0.1,
		frequency:constant::C/461e-9,
		index:6,
	};
	//six laser introduced
	exp_mot.create_entity().with(laser_1).build();
	exp_mot.create_entity().with(laser_2).build();
	exp_mot.create_entity().with(laser_3).build();
	exp_mot.create_entity().with(laser_4).build();
	exp_mot.create_entity().with(laser_5).build();
	exp_mot.create_entity().with(laser_6).build();
	
	
	exp_mot.add_resource(Timestep{t:1e-6});
	// initiate
		// build a oven
	exp_mot.create_entity().with(Oven{temperature:200.,position:[0.1,0.1,0.1],direction:[1e-6,1e-6,1.],number:1,size:[1e-2,1e-2,1e-2]})
	.with(rb_atom).build();
		// initiator dispatched
	let mut init_dispatcher=DispatcherBuilder::new()
			.with(AtomCreate,"atomcreate",&[])
      	.build();
		
	//init_dispatcher.setup(&mut exp_MOT.res);
	init_dispatcher.dispatch(&mut exp_mot.res);
	exp_mot.maintain();
	//two initiators cannot be dispatched at the same time apparently for some unknown reason
	let mut init_dispatcher2=DispatcherBuilder::new().with(AtomInitiateMot, "initiate", &[]).build();
	init_dispatcher2.dispatch(&mut exp_mot.res);
	// run loop
	let mut runner=DispatcherBuilder::new().
	with(UpdateLaser,"updatelaser",&[]).
	with(Sample3DQuadrupoleFieldSystem,"updatesampler",&[]).
	with(UpdateInteractionLaser,"updateinter",&["updatelaser","updatesampler"]).
	with(UpdateRandKick,"update_kick",&["updateinter"]).
	with(UpdateForce,"updateforce",&["update_kick","updateinter"]).
	with(EulerIntegrationSystem,"updatepos",&["update_kick"]).
	with(PrintOutput,"print",&["updatepos"]).build();
	runner.setup(&mut exp_mot.res);
	for _i in 0..2000{
		runner.dispatch(&mut exp_mot.res);
		exp_mot.maintain();
		//println!("t{}",time);
	}
	
}