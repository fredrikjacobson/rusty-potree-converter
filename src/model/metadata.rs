use crate::indexing::model::Hierarchy as IndexingHierarchy;
use crate::indexing::model::Node;
use crate::model::attributes::{Attribute as InternalAttribute, Attributes};

use crate::potree;
use crate::model::metadata;
use crate::model::options::Encoding;
use crate::model::options::Options;
use crate::model::State;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hierarchy {
	first_chunk_size: u16,
	step_size: u8,
	depth: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
	pub name: String,
	pub description: String,
	pub size: u8,
	pub num_elements: u8,
	pub element_size: u8,
	pub r#type: String,
	pub min: Vec<f64>,
	pub max: Vec<f64>,
}

impl Attribute {
	fn from_attribute(attribute: &InternalAttribute) -> Attribute {
		let InternalAttribute {
			name,
			description,
			size,
			num_elements,
			element_size,
			r#type,
			min,
			max,
		} = attribute;
		let min = match num_elements {
			1 => vec![min.x],
			2 => vec![min.x, min.y],
			3 => vec![min.x, min.y, min.z],
			_ => Vec::new(),
		};
		let max = match num_elements {
			1 => vec![max.x],
			2 => vec![max.x, max.y],
			3 => vec![max.x, max.y, max.z],
			_ => Vec::new(),
		};

		Attribute {
			name: name.to_string(),
			description: description.to_string(),
			size: *size as u8,
			num_elements: *num_elements as u8,
			element_size: *element_size as u8,
			r#type: serde_json::to_string(&r#type).unwrap(),
			min,
			max,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct BoundingBox {
	min: [f64; 3],
	max: [f64; 3],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
	pub version: String,
	pub name: String,
	pub description: String,
	pub points: u64,
	pub projection: String,
	pub hierarchy: Hierarchy,
	pub offset: [f64; 3],
	pub scale: [f64; 3],
	pub spacing: f64,
	pub bounding_box: BoundingBox,
	pub encoding: Encoding,
	pub attributes: Vec<Attribute>,
}

impl Metadata {
	pub fn create(
		root: &Node,
		attributes: &Attributes,
		options: &Options,
		state: &State,
		hierarchy: IndexingHierarchy,
		spacing: f64,
		depth: u8,
	) -> Metadata {
		let min = root.min.clone();
		let max = root.max.clone();
		Metadata {
			version: "2.0".to_string(),
			name: options.name.to_string(),
			description: "".to_string(),
			points: state.points_total,
			projection: "".to_string(),
			hierarchy: Hierarchy {
				first_chunk_size: hierarchy.first_chunk_size as u16,
				step_size: hierarchy.step_size,
				depth,
			},
			offset: attributes.posOffset.to_array(),
			scale: attributes.posOffset.to_array(),
			spacing: spacing,
			bounding_box: BoundingBox {
				min: [min.x, min.y, min.z],
				max: [max.x, max.y, max.z],
			},
			encoding: options.encoding,
			attributes: attributes
				.list
				.iter()
				.map(|attribute| Attribute::from_attribute(attribute))
				.collect(),
		}
	}

	pub fn create_2(
		root: &potree::Node,
		attributes: Vec<metadata::Attribute>,
		options: &Options,
		state: &State,
		hierarchy: &IndexingHierarchy,
		spacing: f64,
		depth: u8,
	) -> Metadata {
		let min = &root.bounds;
		let max = &root.bounds;
		Metadata {
			version: "2.0".to_string(),
			name: options.name.to_string(),
			description: "".to_string(),
			points: state.points_total,
			projection: "".to_string(),
			hierarchy: Hierarchy {
				first_chunk_size: hierarchy.first_chunk_size as u16,
				step_size: hierarchy.step_size,
				depth,
			},
			offset: [0.0, 0.0, 0.0],
			scale: [0.0, 0.0, 0.0],
			spacing: spacing,
			bounding_box: BoundingBox {
				min: [min.lx, min.ly, min.lz],
				max: [max.ux, max.uy, max.uz],
			},
			encoding: options.encoding,
			attributes: attributes
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::model::metadata::Metadata;
	use std::fs::File;
	use std::io::BufReader;
	use std::io::Error;

	#[test]
	fn it_serializes() -> Result<(), Error> {
		// Open the file in read-only mode with buffer.

		let file = File::open("resources/metadata.json")?;
		let reader = BufReader::new(file);

		// Read the JSON contents of the file as an instance of `User`.
		let metadata: Metadata = serde_json::from_reader(reader)?;

		let metadata_json = serde_json::to_string_pretty(&metadata)?;
		print!("{}", metadata_json);

		Ok(())
	}
}
