extern crate specs;
use crate::atom::AtomicTransition;
use serde::{Deserialize, Serialize};
use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage,
};

#[derive(Clone, Copy)]
pub struct DipoleLightIndex {
	pub index: usize,
	pub initiated: bool,
}
impl Component for DipoleLightIndex {
	type Storage = HashMapStorage<Self>;
}
impl Default for DipoleLightIndex {
	fn default() -> Self {
		DipoleLightIndex {
			index: 0,
			initiated: false,
		}
	}
}
