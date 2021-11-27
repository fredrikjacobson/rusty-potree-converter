use crate::hierarchy::create_hierarchy;
use crate::indexing::model::Hierarchy;
use crate::model::metadata::Attribute;
use crate::model::metadata::Metadata;
use crate::model::options::{Encoding, Options};
use crate::model::State;
use crate::potree::Node;
use crate::potree::Potree;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;

const HrcStepSize: usize = 5; // must be 2 or more
pub enum WriteError {
	PrepareDirError { msg: String },
}

pub fn write_potree(potree: Potree, dir: &Path) -> Option<WriteError> {
	let mut f = File::create(dir.join("octree.bin")).expect("Unable to create file");
	let mut writer = Writer::new();
	writer.write(&mut f, &potree);
	let hierarchy = create_hierarchy(&potree.root, writer.node_hierarchy);
	write_hierarchy(&potree, &hierarchy, dir).unwrap();
	write_head(&potree, &hierarchy, dir).unwrap();

	None
}

fn write_hierarchy(potree: &Potree, hierarchy: &Hierarchy, dir: &Path) -> Result<(), Error> {
	let mut file = File::create(dir.join("hierarchy.bin"))?;

	file.write_all(&hierarchy.buffer)?;
	Ok(())
}

fn write_head(potree: &Potree, hierarchy: &Hierarchy, dir: &Path) -> Result<(), Error> {
	let metadata = Metadata::create_2(
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
		0,
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

	fn write(&mut self, mut f: &mut File, potree: &Potree) {
		self.write_nodes(f, vec![&potree.root], potree.scale)
	}

	fn write_nodes(&mut self, mut f: &mut File, nodes: Vec<&Node>, scale: f64) {
		let mut children: Vec<&Node> = Vec::new();
		for node in nodes.iter() {
			self.write_points(&mut f, node, scale);

			for child in node.children.iter() {
				if let Some(child) = child {
					if child.num_points() > 0 {
						children.push(child);
					}
				};
			}
		}
		let new_hierarchy_level = match children.first() {
			Some(node) if node.name.len() > 1 && node.name.len() % HrcStepSize == 0 => true,
			_ => false,
		};
		if new_hierarchy_level {
			for node in children {
				self.write_nodes(&mut f, vec![node], scale)
			}
		} else if !children.is_empty() {
			self.write_nodes(&mut f, children, scale)
		}
	}

	fn write_points(&mut self, file: &mut File, node: &Node, scale: f64) {
		let byte_size = node.num_points() as u32 * self.bytes_per_point;
		let byte_offset = self.byte_offset;
		for point in node.points() {
			let cart_x = ((point.x - node.bounds.lx) / scale).round() as i32;
			let cart_y = ((point.y - node.bounds.ly) / scale).round() as i32;
			let cart_z = ((point.z - node.bounds.lz) / scale).round() as i32;
			file.write_i32::<LittleEndian>(cart_x).unwrap();
			file.write_i32::<LittleEndian>(cart_y).unwrap();
			file.write_i32::<LittleEndian>(cart_z).unwrap();
			// Add more props here
		}

		self.byte_offset += byte_size;

		self.node_hierarchy.insert(node.name.to_string(), (byte_size, byte_offset));
	}
}
