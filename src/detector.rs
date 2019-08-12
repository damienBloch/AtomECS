use crate::atom::{Atom, Position, Velocity};
use crate::integrator::{Step, Timestep};
extern crate specs;
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, NullStorage, Read, ReadExpect,
    ReadStorage, System,
};

use std::fs::File;
use std::fs::OpenOptions;
extern crate csv;

use std::error::Error;
extern crate nalgebra;

use crate::destructor::ToBeDestroyed;
use nalgebra::Vector3;

pub struct ClearerCSV {
    filename: &'static str,
}

impl Component for ClearerCSV {
    type Storage = HashMapStorage<Self>;
}

pub struct ClearCSVSystem;

impl<'a> System<'a> for ClearCSVSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, ClearerCSV>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (ent, clearer, lazy): Self::SystemData) {
        for (ent, clearer) in (&ent, &clearer).join() {
            match clearcsv(clearer.filename) {
                Ok(_) => (),
                Err(why) => panic!("output error{}", why.description()),
            };
            lazy.insert(ent, ToBeDestroyed);
        }
    }
}

pub struct Detector {
    radius: f64,
    thickness: f64,
    direction: Vector3<f64>,
    filename: &'static str,
}


impl Detector {
    pub fn if_detect(&self, pos: &Vector3<f64>) -> bool {
        let dir = pos.normalize();
        let dis_vertical = dir.dot(&pos);
        let dis_radial = (pos.norm_squared() - dis_vertical.powf(2.0)).powf(0.5);
        (dis_vertical > -0.5 * self.thickness)
            && (dis_vertical < 0.5 * self.thickness)
            && (dis_radial < self.radius)
    }
}

impl Component for Detector {
    type Storage = HashMapStorage<Self>;
}

pub struct DetectingAtomSystem;

impl<'a> System<'a> for DetectingAtomSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Detector>,
        Entities<'a>,
        ReadStorage<'a, Atom>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Velocity>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
    );
    fn run(&mut self, (pos, detector, ent, _atom, lazy, vel, step, timestep): Self::SystemData) {
        let time = step.n as f64 * timestep.delta;
        for (detector_pos, detector) in (&pos, &detector).join() {
            for (atom_pos, _, ent, vel) in (&pos, &_atom, &ent, &vel).join() {
                let rela_pos = atom_pos.pos - detector_pos.pos;
                if detector.if_detect(&rela_pos) {
                    lazy.insert(ent, ToBeDestroyed);
                    let content = vec![
                        vel.vel[0],
                        vel.vel[1],
                        vel.vel[2],
                        time,
                        atom_pos.pos[0],
                        atom_pos.pos[1],
                        atom_pos.pos[2],
                    ];
                    match print_detected_to_file(detector.filename, &content) {
                        Ok(_) => (),
                        Err(why) => panic!("error writing file,{}", why.description()),
                    };
                }
            }
        }
    }
}
pub fn print_detected_to_file(
    filename: &'static str,
    content: &Vec<f64>,
) -> Result<(), Box<Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(&content)?;

    Ok(())
}

pub fn clearcsv(filename: &str) -> Result<(), Box<Error>> {
    let mut file = OpenOptions::new().write(true).open(filename).unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(&[
        "Velocity_Upon_Capture_X",
        "Velocity_Upon_Capture_Y",
        "Velocity_Upon_Capture_Z",
        "Initial_Velocity_X",
        "Initial_Velocity_Y",
        "Initial_Velocity_Z",
        "Time_Captured",
        "Position_Captured_X",
        "Position_Captured_Y",
        "Position_Captured_Z",
    ])?;

    Ok(())
}