use crate::model::hierarchy::create_hierarchy;
use crate::model::hierarchy::Hierarchy;
use crate::model::metadata::Attribute;
use crate::model::metadata::Metadata;
use crate::model::node::Node;
use crate::model::options::{Encoding, Options};
use crate::model::vector3::Vector3;
use crate::model::State;
use crate::potree::Potree;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::path::Path;

const HRC_STEP_SIZE: usize = 5; // must be 2 or more

#[derive(Debug)]
pub enum WriteError {
    PrepareDirError { msg: String },
}
use core::fmt;
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not prepare directory")
    }
}


pub fn write_potree(potree: Potree, dir: &Path) -> Result<(), Error> {
    let mut f = File::create(dir.join("octree.bin")).expect("Unable to create file");
    let mut writer = Writer::new(&mut f);
    writer.write(&potree);
    let hierarchy = create_hierarchy(&potree.root, writer.node_hierarchy);

    write_hierarchy(&hierarchy, dir).unwrap();

    let metadata = create_metadata(&potree, &hierarchy);
    write_metadata(metadata, dir).unwrap();

    Ok(())
}

fn write_hierarchy(hierarchy: &Hierarchy, dir: &Path) -> Result<(), Error> {
    let mut file = File::create(dir.join("hierarchy.bin"))?;
    file.write_all(&hierarchy.buffer)?;
    Ok(())
}

pub fn create_metadata(potree: &Potree, hierarchy: &Hierarchy) -> Metadata {
    Metadata::create(
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
    )
}

fn write_metadata(metadata: Metadata, dir: &Path) -> Result<(), Error> {
    let file = File::create(dir.join("metadata.json"))?;
    serde_json::to_writer(file, &metadata)?;

    Ok(())
}

pub struct Writer<'a, T: std::io::Write> {
    byte_offset: u32,
    bytes_per_point: u32,
    pub node_hierarchy: HashMap<String, (u32, u32)>,
	buf_writer: &'a mut T
}

impl<T: std::io::Write> Writer<'_, T> {
    pub fn new<'a>(buf_writer: &'a mut T) -> Writer<T> {
        Writer {
            byte_offset: 0,
            bytes_per_point: 12,
            node_hierarchy: HashMap::new(),
			buf_writer
        }
    }

    pub fn write(&mut self, potree: &Potree)
    {
        let offset = Vector3 {
            x: potree.bounds.lx,
            y: potree.bounds.ly,
            z: potree.bounds.lz,
        };
        self.write_nodes(vec![&potree.root], potree.scale, &offset)
    }

    fn write_nodes(
        &mut self,
        nodes: Vec<&Node>,
        scale: f64,
        offset: &Vector3,
    ) {
        let mut children: Vec<&Node> = Vec::new();
        for node in nodes.iter() {
            self.write_points(node, scale, offset);

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
                self.write_nodes(vec![node], scale, offset)
            }
        } else if !children.is_empty() {
            self.write_nodes(children, scale, offset)

        }
    }

    fn write_points(
        &mut self,
        node: &Node,
        scale: f64,
        offset: &Vector3,
    ) {
        let byte_size = node.num_points() as u32 * self.bytes_per_point;
        let byte_offset = self.byte_offset;
        for point in node.points() {
            let cart_x = ((point.x - offset.x) / scale).round() as i32;
            let cart_y = ((point.y - offset.y) / scale).round() as i32;
            let cart_z = ((point.z - offset.z) / scale).round() as i32;
            self.buf_writer.write_i32::<LittleEndian>(cart_x).unwrap();
            self.buf_writer.write_i32::<LittleEndian>(cart_y).unwrap();
            self.buf_writer.write_i32::<LittleEndian>(cart_z).unwrap();
            // Add more props here
        }

        self.byte_offset += byte_size;

        self.node_hierarchy
            .insert(node.name.to_string(), (byte_size, byte_offset));
    }
}