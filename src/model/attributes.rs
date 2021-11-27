use crate::indexing::model::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum AttributeType {
	INT8 = 0,
	INT16 = 1,
	INT32 = 2,
	INT64 = 3,

	UINT8 = 10,
	UINT16 = 11,
	UINT32 = 12,
	UINT64 = 13,

	FLOAT = 20,
	DOUBLE = 21,

	UNDEFINED = 123456,
}

pub struct Attribute {
	pub name: String,
	pub description: String,
	pub size: i32,
	pub num_elements: i32,
	pub element_size: i32,
	pub r#type: AttributeType,

	pub min: Vector3,
	pub max: Vector3,
}

pub struct Attributes {
	pub list: Vec<Attribute>,
	pub bytes: i32,

	pub posScale: Vector3,
	pub posOffset: Vector3,
}

impl Attributes {
	pub fn new() -> Attributes {
		Attributes {
			bytes: 0,
			posScale: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			posOffset: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			list: Vec::new(),
		}
	}

	pub fn from_attributes(attributes: Vec<Attribute>) -> Attributes {
		Attributes {
			bytes: 0,
			posScale: Vector3 {
				x: 1.0,
				y: 1.0,
				z: 1.0,
			},
			posOffset: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			list: attributes,
		}
	}

	fn getOffset(&self, name: String) -> i32 {
		let mut offset = 0;

		for attribute in &self.list {
			if attribute.name == name {
				return offset;
			}

			offset += attribute.size;
		}

		return -1;
	}

	fn get(&self, name: String) -> Option<&Attribute> {
		for attribute in &self.list {
			if attribute.name == name {
				return Some(&attribute);
			}
		}
		return None;
	}
}
