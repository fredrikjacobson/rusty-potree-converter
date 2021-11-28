use std::ops;

#[derive(Clone)]
pub struct Vector3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Vector3 {
	pub fn infinity() -> Vector3 {
		Vector3 {
			x: f64::INFINITY,
			y: f64::INFINITY,
			z: f64::INFINITY,
		}
	}

	pub fn empty() -> Vector3 {
		Vector3 {
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}
	}

	pub fn to_array(&self) -> [f64; 3] {
		[self.x, self.y, self.z]
	}
}

impl ops::Mul<Vector3> for Vector3 {
	type Output = Vector3;

	fn mul(self, _rhs: Vector3) -> Vector3 {
		Vector3 {
			x: self.x * _rhs.x,
			y: self.y * _rhs.y,
			z: self.z * _rhs.z,
		}
	}
}

impl ops::Mul<f64> for Vector3 {
	type Output = Vector3;

	fn mul(self, scalar: f64) -> Vector3 {
		Vector3 {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar,
		}
	}
}

impl ops::Sub<&Vector3> for &Vector3 {
	type Output = Vector3;

	fn sub(self, _rhs: &Vector3) -> Vector3 {
		Vector3 {
			x: self.x - _rhs.x,
			y: self.y - _rhs.y,
			z: self.z - _rhs.z,
		}
	}
}

impl ops::Sub<Vector3> for Vector3 {
	type Output = Vector3;

	fn sub(self, _rhs: Vector3) -> Vector3 {
		Vector3 {
			x: self.x - _rhs.x,
			y: self.y - _rhs.y,
			z: self.z - _rhs.z,
		}
	}
}

impl ops::Sub<&Vector3> for Vector3 {
	type Output = Vector3;

	fn sub(self, _rhs: &Vector3) -> Vector3 {
		Vector3 {
			x: self.x - _rhs.x,
			y: self.y - _rhs.y,
			z: self.z - _rhs.z,
		}
	}
}

impl ops::Add<Vector3> for Vector3 {
	type Output = Vector3;

	fn add(self, _rhs: Vector3) -> Vector3 {
		Vector3 {
			x: self.x + _rhs.x,
			y: self.y + _rhs.y,
			z: self.z + _rhs.z,
		}
	}
}

impl ops::Add<f64> for Vector3 {
	type Output = Vector3;
	fn add(self, scalar: f64) -> Vector3 {
		Vector3 {
			x: self.x + scalar,
			y: self.y + scalar,
			z: self.z + scalar,
		}
	}
}