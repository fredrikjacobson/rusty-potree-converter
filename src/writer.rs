use crate::model::hierarchy::create_hierarchy;
use crate::model::hierarchy::Hierarchy;
use crate::model::metadata::Attribute;
use crate::model::metadata::Metadata;
use crate::model::options::{Encoding, Options};
use crate::model::vector3::Vector3;
use crate::model::State;
use crate::potree::Node;
use crate::potree::Potree;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;

const HRC_STEP_SIZE: usize = 5; // must be 2 or more
pub enum WriteError {
	PrepareDirError { msg: String },
}

pub fn write_potree(potree: Potree, dir: &Path) -> Option<WriteError> {
	let mut f = File::create(dir.join("octree.bin")).expect("Unable to create file");
	let mut writer = Writer::new();
	writer.write(&mut f, &potree);
	let hierarchy = create_hierarchy(&potree.root, writer.node_hierarchy);
	write_hierarchy(&hierarchy, dir).unwrap();
	write_metadata(&potree, &hierarchy, dir).unwrap();

	None
}

fn write_hierarchy(hierarchy: &Hierarchy, dir: &Path) -> Result<(), Error> {
	let mut file = File::create(dir.join("hierarchy.bin"))?;

	file.write_all(&hierarchy.buffer)?;
	Ok(())
}

fn write_metadata(potree: &Potree, hierarchy: &Hierarchy, dir: &Path) -> Result<(), Error> {
	let metadata = Metadata::create(
		&potree.root,
		vec![Attribute {
			name: "position".to_string(),
			description: "".to_string(),
			size: 12,
			num_elements: 3,
			element_size: 4,
			r#type: "int32".to_string(),

			min: vec![potree.bounds.lx, potree.bounds.ly, potree.bounds.lz],
			max: vec![potree.bounds.ux, potree.bounds.uy, potree.bounds.uz],
		}],
		&Options {
			encoding: Encoding::DEFAULT,
			keep_chunks: false,
			name: "".to_string(),
		},
		&State {
			name: "".to_string(),
			points_total: potree.size as u64,
		},
		hierarchy,
		potree.spacing,
		potree.scale,
	);

	let file = File::create(dir.join("metadata.json"))?;

	serde_json::to_writer(file, &metadata)?;

	Ok(())
}

struct Writer {
	byte_offset: u32,
	bytes_per_point: u32,
	node_hierarchy: HashMap<String, (u32, u32)>,
}

impl Writer {
	fn new() -> Writer {
		Writer {
			byte_offset: 0,
			bytes_per_point: 12,
			node_hierarchy: HashMap::new(),
		}
	}

	fn write(&mut self, f: &mut File, potree: &Potree) {
		let offset = Vector3 {
			x: potree.bounds.lx,
			y: potree.bounds.ly,
			z: potree.bounds.lz,
		};
		self.write_nodes(f, vec![&potree.root], potree.scale, &offset)
	}

	fn write_nodes(&mut self, mut f: &mut File, nodes: Vec<&Node>, scale: f64, offset: &Vector3) {
		let mut children: Vec<&Node> = Vec::new();
		for node in nodes.iter() {
			self.write_points(&mut f, node, scale, offset);

			for child in node.children.iter() {
				if let Some(child) = child {
					if child.num_points() > 0 {
						children.push(child);
					}
				};
			}
		}
		let new_hierarchy_level = match children.first() {
			Some(node) if node.name.len() > 1 && node.name.len() % HRC_STEP_SIZE == 0 => true,
			_ => false,
		};
		if new_hierarchy_level {
			for node in children {
				self.write_nodes(&mut f, vec![node], scale, offset)
			}
		} else if !children.is_empty() {
			self.write_nodes(&mut f, children, scale, offset)
		}
	}

	fn write_points(&mut self, file: &mut File, node: &Node, scale: f64, offset: &Vector3) {
		let byte_size = node.num_points() as u32 * self.bytes_per_point;
		let byte_offset = self.byte_offset;
		for point in node.points() {
			let cart_x = ((point.x - offset.x) / scale).round() as i32;
			let cart_y = ((point.y - offset.y) / scale).round() as i32;
			let cart_z = ((point.z - offset.z) / scale).round() as i32;
			file.write_i32::<LittleEndian>(cart_x).unwrap();
			file.write_i32::<LittleEndian>(cart_y).unwrap();
			file.write_i32::<LittleEndian>(cart_z).unwrap();
			// Add more props here
		}

		self.byte_offset += byte_size;

		self.node_hierarchy
			.insert(node.name.to_string(), (byte_size, byte_offset));
	}
}
