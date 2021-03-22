//! Gaussian beam intensity distribution

extern crate nalgebra;
extern crate rayon;
extern crate specs;
use nalgebra::Vector3;
use specs::{Component, HashMapStorage};

use crate::atom::Position;
use crate::maths;
use serde::{Deserialize, Serialize};
use crate::constant::PI;

/// A component representing an intensity distribution with a gaussian profile.
///
/// The beam will propagate in vacuum. Inhomogenous media, gravitational lensing, refractions and
/// reflections (other than through a `CircularMask` are not implemented.
///
/// Also, attenuation effects are not yet implemented but they might come in a version
/// that accounts for atom-atom intereactions in the future.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct GaussianBeam {
	/// A point that the laser beam intersects
	pub intersection: Vector3<f64>,

	/// Direction the beam propagates with respect to cartesian `x,y,z` axes.
	pub direction: Vector3<f64>,

	/// Radius of the beam at which the intensity is 1/e of the peak value, SI units of m.
	/// 
	/// Since in the literature the e^2_radius (where intensity is 1/e^2 of peak value) is used
	/// very often as well, it is useful to note the following relation:
	/// 
	/// e_radius = e^2_radius / sqrt(2)
	pub e_radius: f64,

	/// Power of the laser in W
	pub power: f64,
}
impl Component for GaussianBeam {
	type Storage = HashMapStorage<Self>;
}

/// A component that covers the central portion of a laser beam.
///
/// The mask is assumed to be coaxial to the GaussianBeam.
#[derive(Clone, Copy)]
pub struct CircularMask {
	/// Radius of the masked region in units of m.
	pub radius: f64,
}
impl Component for CircularMask {
	type Storage = HashMapStorage<Self>;
}

/// Returns the intensity of a gaussian laser beam at the specified position.
pub fn get_gaussian_beam_intensity(
	beam: &GaussianBeam,
	pos: &Position,
	mask: Option<&CircularMask>,
	rayleigh: Option<&GaussianRayleighRange>,
) -> f64 {
	let (min_dist, z) =
		maths::get_minimum_distance_line_point(&pos.pos, &beam.intersection, &beam.direction);
	let power = match mask {
		Some(mask) => {
			if min_dist < mask.radius {
				0.0
			} else {
				beam.power
			}
		}
		None => beam.power,
	};
	let broadening_factor = match rayleigh {
		Some(rayleigh_range) => {
			1. / (1. + z.powf(2.0)/rayleigh_range.rayleigh_range.powf(2.0))
		}
		None => 1.0
	};
	power * broadening_factor * maths::gaussian_dis(beam.e_radius / 2.0_f64.powf(0.5), min_dist)
}

/// A component that enables the correct treatment of the `GaussianBeam` for cases where 
/// it is strongly focused, i.e. the beam waist is very small compared to the axial
/// length on which the intensity is required. 
/// 
/// This is especially important for the dipole force since the axial gradient is 
/// crucial for optical transport. For most MOT simulations, this component is not 
/// required since rayleigh ranges are typically several hundreds of metres.
#[derive(Clone, Copy)]
pub struct GaussianRayleighRange{
	/// The distance along the propagation direction of a beam from the
	///  waist to the place where the area of the cross section is doubled in units of metres
	pub rayleigh_range: f64,
}

fn make_gaussian_rayleigh_range(wavelength: &f64, gaussian: &GaussianBeam) -> GaussianRayleighRange {
    GaussianRayleighRange{
		rayleigh_range: 2.0 * PI * gaussian.e_radius.powf(2.0) / wavelength,
	}
}

impl Component for GaussianRayleighRange{
	type Storage = HashMapStorage<Self>;
}


#[cfg(test)]
pub mod tests {

	use super::*;

	extern crate specs;
	use crate::constant::PI;
	use assert_approx_eq::assert_approx_eq;

	extern crate nalgebra;
	use nalgebra::Vector3;

	#[test]
	fn test_get_gaussian_beam_intensity() {
		let beam = GaussianBeam {
			direction: Vector3::x(),
			intersection: Vector3::new(0.0, 0.0, 0.0),
			e_radius: 2.0,
			power: 1.0,
		};

		let pos1 = Position { pos: Vector3::x() };
		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0),
			get_gaussian_beam_intensity(&beam, &pos1, None, None),
			1e-6_f64
		);

		let pos2 = Position { pos: Vector3::y() };
		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
				* (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2, None, None),
			1e-6_f64
		);

		let rayleigh_range = GaussianRayleighRange{
			rayleigh_range: 1.0,
		};

		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0) / 2.,
			get_gaussian_beam_intensity(&beam, &pos1, None, Some(&rayleigh_range)),
			1e-6_f64
		);

		assert_approx_eq!(
			1.0 / (PI.powf(0.5) * beam.e_radius).powf(2.0)
				* (-pos2.pos[1] / beam.e_radius.powf(2.0)).exp(),
			get_gaussian_beam_intensity(&beam, &pos2, None, Some(&rayleigh_range)),
			1e-6_f64
		);
		let rayleigh_range_2 = make_gaussian_rayleigh_range(&461.0e-6, &beam);

		let pos3 = Position { pos: Vector3::x()*rayleigh_range_2.rayleigh_range};
		assert_approx_eq!(
			beam.power / (PI.powf(0.5) * beam.e_radius).powf(2.0)/2.,
			get_gaussian_beam_intensity(&beam, &pos3, None, Some(&rayleigh_range_2)),
			1e-6_f64
		);
	}
}
