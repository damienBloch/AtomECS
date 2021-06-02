//! Magnetic field from a straight wire.

extern crate nalgebra;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

use crate::atom::Position;
use crate::magnetic::MagneticFieldSampler;

/// A component representing a straight wire.
/// For example, it is possible to represent a rectangular coil by
/// adding four wires end-to-end.
#[derive(Serialize, Deserialize)]
pub struct MagneticWireField {
    /// Length of the wire, in m
    pub length: f64,
    /// Current in the wire, in Ampere
    pub current: f64,
    /// A unit vector pointing along the direction of the wire
    pub direction: Vector3<f64>,
}

impl Component for MagneticWireField {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include wires in the world.
pub struct SampleMagneticWireFieldSystem;

impl SampleMagneticWireFieldSystem {
    /// Calculates the magnetic field of the wire.
    ///
    /// # Arguments
    ///
    /// `location`: position of the sampler, m
    ///
    /// `position`: position of the wire center, m
    ///
    /// `length`: length of the wire, m
    ///
    /// `current`: current in the wire, in Ampere
    ///
    /// `direction`: A _normalized_ vector pointing in the direction of the wire.
    pub fn calculate_field(
        location: Vector3<f64>,
        position: Vector3<f64>,
        length: f64,
        current: f64,
        direction: Vector3<f64>,
    ) -> Vector3<f64> {
        let delta = location - position;
        let axial_dist = delta.dot(&direction);

        let perp = delta - axial_dist * direction;
        let radial_dist = perp.norm();
        let magnitude = 1e-7 * current / radial_dist * (
            (axial_dist - length / 2.) / (radial_dist.powi(2) + (axial_dist - length / 2.).powi(2)).sqrt()
                - (axial_dist + length / 2.) / (radial_dist.powi(2) + (axial_dist + length / 2.).powi(2)).sqrt()
        );

        magnitude * perp.cross(&direction) / (radial_dist + 1e-6)
    }
}

impl<'a> System<'a> for SampleMagneticWireFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, MagneticWireField>,
    );
    fn run(&mut self, (mut sampler, positions, wires): Self::SystemData) {
        use rayon::prelude::*;
        use specs::ParJoin;

        for (position, wire) in (&positions, &wires).join() {
            (&positions, &mut sampler)
                .par_join()
                .for_each(|(location, mut sampler)| {
                    let field = SampleMagneticWireFieldSystem::calculate_field(
                        location.pos,
                        position.pos,
                        wire.length,
                        wire.current,
                        wire.direction,
                    );
                    sampler.field = sampler.field + field;
                });
        }
    }
}

#[cfg(test)]
pub mod tests {
    use assert_approx_eq::assert_approx_eq;
    use nalgebra::Vector3;

    use super::*;

    extern crate nalgebra;

    /// Tests the correct implementation of the wire field.
    #[test]
    fn test_wire_field() {
        let location = Vector3::new(1., 0., 0.);
        let position = Vector3::new(0., 0., 0.);
        let length = 2.;
        let current = 1e7;
        let direction = Vector3::z();
        let field =
            SampleMagneticWireFieldSystem::calculate_field(location, position, length, current, direction);
        assert_approx_eq!(field.x, 0.0);
        assert_approx_eq!(field.y, 2f64.sqrt(), 1e-5);
        assert_approx_eq!(field.z, 0.0);
    }
}
