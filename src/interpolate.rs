
// implement the ability to ramp components by linearly interpolating between keyframe values

// Define a table of keyframe values for a component that implements trait Interpolatable
// A system then runs over tables and components with this trait and updates the components

use specs::{Component, Join,ReadStorage,WriteStorage,System,ReadExpect,HashMapStorage};
use std::marker::PhantomData;
use crate::integrator::{Step,Timestep};

/// trait implemented by every interpolatable component
pub trait Interpolatable {
	fn lerp(&self, A : &Self, time : f64) -> Self;
}

// struct that contains keyframe values and timesteps for ramping 
// an interpolatable component
pub struct Table<C: Component + Clone + Interpolatable + Sync + Send> {
	keyframe_values: Vec<C>,
	keyframe_times: Vec<f64>
}

impl<C: Component + Clone + Interpolatable + Sync + Send> Component for Table<C>{
	type Storage = HashMapStorage<Self>;
}

// system that updates interpolatable components
pub struct TableSystem<C: Component + Interpolatable + Clone>{
    marker: PhantomData<C>,
}

impl<'a, C: Component + Interpolatable + Clone + Sync + Send> System<'a> for TableSystem<C> {
    type SystemData = (
        ReadStorage<'a, Table<C>>,
        WriteStorage<'a, C>,
        ReadExpect<'a, Step>,
        ReadExpect<'a, Timestep>,
    );   
    fn run(&mut self, (tables, mut components, step, t): Self::SystemData) {

        let time = step.n as f64 * t.delta;
        for (table, mut component) in (&tables, &mut components).join(){

            component = component.lerp(table, time);

        }
    }
}
