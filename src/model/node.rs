use crate::model::vector3::Vector3;

pub struct BoundingBox {
	pub min: Vector3,
	pub max: Vector3,
}

impl BoundingBox {
	pub fn empty() -> BoundingBox {
		BoundingBox {
			min: Vector3::infinity(),
			max: Vector3::infinity() * -1.0,
		}
	}

	pub fn new(min: Vector3, max: Vector3) -> BoundingBox {
		BoundingBox { min: min, max: max }
	}
}

fn child_bounding_box_of(min: &Vector3, max: &Vector3, index: usize) -> BoundingBox {
	let mut bbox = BoundingBox::empty();
	let size = max.clone() - min;
	let center = min.clone() + (size * 0.5);

	if (index & 0b100) == 0 {
		bbox.min.x = min.x;
		bbox.max.x = center.x;
	} else {
		bbox.min.x = center.x;
		bbox.max.x = max.x;
	}

	if (index & 0b010) == 0 {
		bbox.min.y = min.y;
		bbox.max.y = center.y;
	} else {
		bbox.min.y = center.y;
		bbox.max.y = max.y;
	}

	if (index & 0b001) == 0 {
		bbox.min.z = min.z;
		bbox.max.z = center.z;
	} else {
		bbox.min.z = center.z;
		bbox.max.z = max.z;
	}

	return bbox;
}