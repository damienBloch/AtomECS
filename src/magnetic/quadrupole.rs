extern crate specs;
use super::MagneticFieldSampler;
use crate::atom::Position;
use crate::maths;
use specs::{Component, HashMapStorage, Join, ReadStorage, System, WriteStorage};

/// A component representing a 3D quadrupole field.
pub struct QuadrupoleField3D {
    /// Gradient of the quadrupole field, in units of Tesla/m
    pub gradient: f64,
}
impl QuadrupoleField3D {
    /// Creates a `QuadrupoleField3D` component with gradient specified in Gauss per cm.
    pub fn gauss_per_cm(gradient: f64) -> Self {
        Self {
            gradient: gradient * 0.01,
        }
    }
}

impl Component for QuadrupoleField3D {
    type Storage = HashMapStorage<Self>;
}

/// Updates the values of magnetic field samplers to include quadrupole fields in the world.
pub struct Sample3DQuadrupoleFieldSystem;

impl Sample3DQuadrupoleFieldSystem {
    /// Calculates the quadrupole magnetic field.
    /// The field is defined with components `Bx = grad*x`, `By = grad*y`, `Bz = -2 * grad * z`.
    ///
    /// # Arguments
    ///
    /// `pos`: position of the sampler, m
    ///
    /// `centre`: position of the quadrupole node, m
    ///
    /// `gradient`: quadrupole gradient, in Tesla/m
    pub fn calculate_field(pos: &[f64; 3], centre: &[f64; 3], gradient: f64) -> [f64; 3] {
        let rel_pos = maths::array_subtraction(&pos, &centre);
        [
            rel_pos[0] * gradient,
            rel_pos[1] * gradient,
            rel_pos[2] * -2. * gradient,
        ]
    }
}

impl<'a> System<'a> for Sample3DQuadrupoleFieldSystem {
    type SystemData = (
        WriteStorage<'a, MagneticFieldSampler>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, QuadrupoleField3D>,
    );
    fn run(&mut self, (mut sampler, pos, quadrupole): Self::SystemData) {
        for (centre, quadrupole) in (&pos, &quadrupole).join() {
            for (pos, mut sampler) in (&pos, &mut sampler).join() {
                let quad_field = Sample3DQuadrupoleFieldSystem::calculate_field(
                    &pos.pos,
                    &centre.pos,
                    quadrupole.gradient,
                );
                sampler.field = maths::array_addition(&quad_field, &sampler.field);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    /// Tests the correct implementation of the quadrupole 3D field
    #[test]
    fn test_quadrupole3dfield() {
        let pos = [1., 1., 1.];
        let centre = [0., 1., 0.];
        let gradient = 1.;
        let field = Sample3DQuadrupoleFieldSystem::calculate_field(&pos, &centre, gradient);
        assert_eq!(field, [1., 0., -2.]);
    }
}
