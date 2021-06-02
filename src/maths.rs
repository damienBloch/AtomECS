//! Mathematical utilities

use crate::constant::EXP;
use crate::constant::PI;
extern crate nalgebra;
use nalgebra::Vector3;
use self::nalgebra::Complex;

/// Get miniminum distance between a point and a line.
///
/// # Arguments
///
/// `pos`: position of the point
///
/// `line_point`: a point on the line
///
/// `dir`: vector pointing along the line.
pub fn get_minimum_distance_line_point(
	pos: &Vector3<f64>,
	line_point: &Vector3<f64>,
	dir: &Vector3<f64>,
) -> f64 {
	let rela_cood = pos - line_point;
	let distance = (dir.cross(&rela_cood) / dir.norm()).norm();
	distance
}

/// A normalised gaussian distribution.
///
/// The distribution is normalised such that the 2D area underneath a gaussian dist with sigma_x=sigma_y=std is equal to 1.
pub fn gaussian_dis(std: f64, distance: f64) -> f64 {
	1.0 / (2.0 * PI * std * std) * EXP.powf(-distance * distance / 2.0 / (std * std))
}

/// Returns two vectors forming an orthogonal direct basis with the argument
pub fn get_ortho_basis(direction: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>)
{
	let dir = Vector3::new(0.23, 1.2, 0.4563).normalize();
	let perp_x: Vector3<f64> = direction.normalize().cross(&dir).normalize();
	let perp_y: Vector3<f64> = direction.normalize().cross(&perp_x);
	(perp_x, perp_y)
}

/// Dot product between two complex vectors
pub fn cdot(v1: &Vector3<Complex<f64>>, v2: &Vector3<Complex<f64>>) -> Complex<f64>
{
	v1.x.conj() * v2.x + v1.y.conj() * v2.y + v1.z.conj() * v2.z
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_minimum_distance_line_point() {
		let pos = Vector3::new(1., 1., 1.);
		let centre = Vector3::new(0., 1., 1.);
		let dir = Vector3::new(1., 2., 2.);
		let distance = get_minimum_distance_line_point(&pos, &centre, &dir);
		assert!(distance > 0.942, distance < 0.943);
	}
}
