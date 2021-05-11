//! Components for the CoolingLight entities and their initilization

extern crate specs;
use crate::atom::AtomicTransition;
use serde::{Deserialize, Serialize};
use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage,
};

use crate::constant;
use nalgebra::{Vector3, Complex};
use crate::maths::get_ortho_basis;

/// A representation of the light polarization as a 3D complex vector.
///
/// With this representation, the polarization is defined by a constant vector.
/// The complex coefficients hold the magnitude and dephasing of the polarization
/// in all three directions.
/// The vector must be normalized and orthogonal to the light wavevector.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct PolarizedLight {
    pub vector: Vector3<Complex<f64>>,
}

impl PolarizedLight {
    /// Returns one of the right hand polarizations with respect to a given direction.
    pub fn circular_right(direction: Vector3<f64>) -> Self {
        let perp_pair = get_ortho_basis(direction);
        let perp_x = perp_pair.0;
        let perp_y = perp_pair.1;
        let polarization = Vector3::new(Complex::new(perp_x.x, -perp_y.x),
                                        Complex::new(perp_x.y, -perp_y.y),
                                        Complex::new(perp_x.z, -perp_y.z)) / Complex::new(2.0f64.sqrt(), 0.0);
        PolarizedLight { vector: polarization }
    }

    /// Returns one of left hand polarizations with respect to a given direction.
    pub fn circular_left(direction: Vector3<f64>) -> Self {
        let perp_pair = get_ortho_basis(direction);
        let perp_x = perp_pair.0;
        let perp_y = perp_pair.1;
        let polarization = Vector3::new(Complex::new(perp_x.x, perp_y.x),
                                        Complex::new(perp_x.y, perp_y.y),
                                        Complex::new(perp_x.z, perp_y.z)) / Complex::new(2.0f64.sqrt(), 0.0);
        PolarizedLight { vector: polarization }
    }

    /// Returns light linearly polarized and aligned with a given direction.
    pub fn linear(direction: Vector3<f64>) -> Self {
        PolarizedLight {
            vector: Vector3::new(Complex::new(direction.x, 0.0),
                                 Complex::new(direction.y, 0.0),
                                 Complex::new(direction.z, 0.0)).normalize()
        }
    }
}

/// A component representing light properties used for laser cooling.
///
/// Currently only holds the information about polarization and wavelength
/// and works as a marker for all laser cooling processes. This will be
/// split into different components in a future version.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct CoolingLight {
    /// Polarisation of the laser light
    pub polarization: PolarizedLight,

	/// wavelength of the laser light, in SI units of m.
	pub wavelength: f64,
}
impl CoolingLight {
	/// Frequency of the cooling light in units of Hz
	pub fn frequency(&self) -> f64 {
		constant::C / self.wavelength
	}

	/// Wavenumber of the cooling light, in units of 2pi inverse metres.
	pub fn wavenumber(&self) -> f64 {
		2.0 * constant::PI / self.wavelength
	}

    /// Creates a `CoolingLight` component from the desired atomic species.
    ///
    /// # Arguments
    ///
    /// `species`: The atomic species to take the base wavelength from.
    ///
    /// `detuning`: Detuning of the laser from transition in units of MHz
    ///
    /// `polarization`: Polarization of the cooling beam.
    pub fn for_species(species: AtomicTransition, detuning: f64, polarization: PolarizedLight) -> Self {
        let freq = species.frequency + detuning * 1.0e6;
        CoolingLight {
            wavelength: constant::C / freq,
            polarization: polarization,
        }
    }
}
impl Component for CoolingLight {
	type Storage = HashMapStorage<Self>;
}

/// An index that uniquely identifies this cooling light in the interaction list for each atom.
/// The index value corresponds to the position of each cooling light in the per-atom interaction list array.
///
/// Default `CoolingLightIndex`s are created with `initiated: false`.
/// Once the index is set, initiated is set to true.
/// This is used to detect if all lasers in the simulation are correctly indexed, in case new lasers are added.
#[derive(Clone, Copy)]
pub struct CoolingLightIndex {
	pub index: usize,
	pub initiated: bool,
}
impl Component for CoolingLightIndex {
	type Storage = HashMapStorage<Self>;
}
impl Default for CoolingLightIndex {
	fn default() -> Self {
		CoolingLightIndex {
			index: 0,
			initiated: false,
		}
	}
}

/// Assigns unique indices to cooling light entities.
///
/// The indices are used to uniquely identify each cooling light when populating the interaction list.
pub struct IndexCoolingLightsSystem;
impl<'a> System<'a> for IndexCoolingLightsSystem {
	type SystemData = (
		ReadStorage<'a, CoolingLight>,
		WriteStorage<'a, CoolingLightIndex>,
	);

	fn run(&mut self, (cooling_light, mut indices): Self::SystemData) {
		let mut iter = 0;
		let mut need_to_assign_indices = false;
		for (_, index) in (&cooling_light, &indices).join() {
			if index.initiated == false {
				need_to_assign_indices = true;
			}
		}
		if need_to_assign_indices {
			for (_, mut index) in (&cooling_light, &mut indices).join() {
				index.index = iter;
				index.initiated = true;
				iter = iter + 1;
			}
		}
	}
}

/// A system that attaches `CoolingLightIndex` components to entities which have `CoolingLight` but no index.
pub struct AttachIndexToCoolingLightSystem;
impl<'a> System<'a> for AttachIndexToCoolingLightSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, CoolingLight>,
		ReadStorage<'a, CoolingLightIndex>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (ent, cooling_light, cooling_light_index, updater): Self::SystemData) {
		for (ent, _, _) in (&ent, &cooling_light, !&cooling_light_index).join() {
			updater.insert(ent, CoolingLightIndex::default());
		}
	}
}

#[cfg(test)]
pub mod tests {

	use super::*;
	use assert_approx_eq::assert_approx_eq;
	extern crate specs;
	use specs::{Builder, RunNow, World};

	#[test]
	fn test_index_cooling_lights() {
		let mut test_world = World::new();
		test_world.register::<CoolingLightIndex>();
		test_world.register::<CoolingLight>();

        let test_entity_1 = test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: PolarizedLight::circular_right(Vector3::z()),
                wavelength: 780e-9,
            })
            .build();
        let test_entity_2 = test_world
            .create_entity()
            .with(CoolingLightIndex::default())
            .with(CoolingLight {
                polarization: PolarizedLight::circular_right(Vector3::z()),
                wavelength: 780e-9,
            })
            .build();

		let mut system = IndexCoolingLightsSystem;
		system.run_now(&test_world.res);
		test_world.maintain();

		let cooling_storage = test_world.read_storage::<CoolingLightIndex>();
		let index_1 = cooling_storage
			.get(test_entity_1)
			.expect("entity not found");
		let index_2 = cooling_storage
			.get(test_entity_2)
			.expect("entity not found");

		assert_ne!(index_1.index, index_2.index);
	}

	#[test]
	fn test_add_index_component_to_cooling_lights() {
		let mut test_world = World::new();
		test_world.register::<CoolingLightIndex>();
		test_world.register::<CoolingLight>();

        let test_entity = test_world
            .create_entity()
            .with(CoolingLight {
                polarization: PolarizedLight::circular_right(Vector3::z()),
                wavelength: 780e-9,
            })
            .build();

		let mut system = AttachIndexToCoolingLightSystem;
		system.run_now(&test_world.res);
		test_world.maintain();

		assert_eq!(
			test_world
				.read_storage::<CoolingLightIndex>()
				.get(test_entity)
				.is_none(),
			false
		);
	}

	#[test]
	fn test_for_species() {
		let detuning = 12.0;
		let light = CoolingLight::for_species(AtomicTransition::rubidium(), detuning, PolarizedLight::circular_right(Vector3::x()));
		assert_approx_eq!(
			light.frequency(),
			AtomicTransition::rubidium().frequency + 1.0e6 * detuning
		);
	}
}
