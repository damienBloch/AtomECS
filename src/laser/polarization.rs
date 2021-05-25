//! Component for the polarization

use nalgebra::{Complex, Vector3};
use specs::{Component, HashMapStorage};

/// A representation of the light polarization as a 3D complex vector.
///
/// With this representation, the polarization is defined by a constant vector.
/// The complex coefficients hold the magnitude and dephasing of the polarization
/// in all three directions.
/// The vector must be normalized and orthogonal to the light wavevector.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Polarization {
    pub vector: Vector3<Complex<f64>>,
}

impl Component for Polarization {
    type Storage = HashMapStorage<Self>;
}

impl Polarization {
    /// Returns a possible sigma_plus polarization with respect to a given direction.
    pub fn sigma_plus(direction: Vector3<f64>) -> Self {
        let perp_pair = get_ortho_basis(direction);
        let perp_x = perp_pair.0;
        let perp_y = perp_pair.1;
        let polarization = Vector3::new(
            Complex::new(perp_x.x, -perp_y.x),
            Complex::new(perp_x.y, -perp_y.y),
            Complex::new(perp_x.z, -perp_y.z),
        ) / Complex::new(2.0f64.sqrt(), 0.0);
        Polarization {
            vector: polarization,
        }
    }

    /// Returns a possible sigma_minus polarization with respect to a given direction.
    pub fn sigma_minus(direction: Vector3<f64>) -> Self {
        let perp_pair = get_ortho_basis(direction);
        let perp_x = perp_pair.0;
        let perp_y = perp_pair.1;
        let polarization = Vector3::new(
            Complex::new(perp_x.x, perp_y.x),
            Complex::new(perp_x.y, perp_y.y),
            Complex::new(perp_x.z, perp_y.z),
        ) / Complex::new(2.0f64.sqrt(), 0.0);
        Polarization {
            vector: polarization,
        }
    }

    /// Returns linear polarization, aligned with a given direction.
    pub fn linear(direction: Vector3<f64>) -> Self {
        Polarization {
            vector: Vector3::new(
                Complex::new(direction.x, 0.0),
                Complex::new(direction.y, 0.0),
                Complex::new(direction.z, 0.0),
            )
            .normalize(),
        }
    }
}
