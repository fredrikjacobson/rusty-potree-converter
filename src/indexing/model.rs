use crate::model::attributes::Attributes;
use std::boxed::Box;
use std::cell::RefCell;
use std::convert::TryInto;
use std::ops;
use std::rc::Rc;

#[derive(Clone)]
pub struct Vector3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Vector3 {
	pub fn INFINITY() -> Vector3 {
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

pub struct Hierarchy {
	pub step_size: u8,
	pub buffer: Vec<u8>,
	pub first_chunk_size: i64,
}

pub struct Chunk {
	pub min: Vector3,
	pub max: Vector3,

	pub file: String,
	pub id: String,
}

impl Chunk {
	pub fn new(file: String, id: String) -> Chunk {
		Chunk {
			min: Vector3::empty(),
			max: Vector3::empty(),
			file,
			id,
		}
	}
}

pub struct HierarchyChunk<'a> {
	pub name: String,
	pub nodes: Vec<&'a Node>, // Should be a pointer Node*
}

pub struct Point {
	x: f64,
	y: f64,
	z: f64,
	point_index: i32,
	child_index: i32,
}

pub struct NodeCandidate {
	name: String,
	indexStart: i64,
	numPoints: i64,
	level: i64,
	x: i64,
	y: i64,
	z: i64,
}

#[repr(u8)]
pub enum Type {
	Normal = 0,
	Leaf = 1,
	Proxy = 2,
}

pub struct Chunks {
	pub list: Vec<Box<Chunk>>,
	pub min: Vector3,
	pub max: Vector3,
	pub attributes: Attributes,
}

pub struct Node {
	pub children: Vec<Option<Box<Node>>>,

	pub name: String,
	// points: Rc<Buffer>,
	// Vec<CumulativeColor> colors;
	pub min: Vector3,
	pub max: Vector3,

	indexStart: i64,

	pub byteOffset: usize,
	pub byteSize: usize,
	pub numPoints: i64,

	sampled: bool,
}

fn has_child_at(node: &Node, index: usize) -> bool {
	node.children[index].is_some()
}

fn add_node(
	current_node: &mut Node,
	new_node: Box<Node>,
	indices: Vec<usize>,
	current_level: usize,
	level: usize,
) {
	let index = indices[current_level];
	if current_level == level {
		let index: usize = indices[level];
		current_node.children[index].replace(new_node);
	} else {
		match &mut current_node.children[index] {
			&mut Some(ref mut child) => {
				add_node(child, new_node, indices, current_level + 1, level)
			}
			empty_child @ None => {
				let child_name = current_node.name.clone() + &index.to_string();
				let bbox = childBoundingBoxOf(&current_node.min, &current_node.max, index);
				let mut node = Box::new(Node::new(child_name, bbox.min, bbox.max));
				add_node(&mut node, new_node, indices, current_level, level);
				empty_child.replace(node);
			}
		}
	}
}

impl Node {
	pub fn new(name: String, min: Vector3, max: Vector3) -> Node {
		Node {
			name,
			min,
			max,
			indexStart: 0,
			byteOffset: 0,
			byteSize: 0,
			numPoints: 0,
			sampled: false,
			children: vec![None, None, None, None, None, None, None, None],
		}
	}

	pub fn level(&self) -> usize {
		return self.name.len() - 1;
	}

	pub fn addDescendant(&mut self, descendant: Box<Node>) {
		let descendant_name = &descendant.name;
		let descendant_level = descendant_name.len() - 1;
		let indices: Vec<usize> = descendant_name
			.chars()
			.map(|i| (i as usize - '0' as usize))
			.collect();

		add_node(self, descendant, indices, 0, descendant_level);
	}

	fn traverse(&self, callback: fn(&Node) -> ()) {
		callback(self);

		for child in &self.children {
			if let Some(child) = child {
				child.traverse(callback);
			}
		}
	}

	fn traversePost(&self, callback: fn(&Node) -> ()) {
		for child in &self.children {
			if let Some(child) = child {
				child.traversePost(callback);
			}
		}

		callback(self);
	}

	pub fn isLeaf(&self) -> bool {
		for child in &self.children {
			if let Some(child) = child {
				return false;
			}
		}

		return true;
	}
}

pub struct BoundingBox {
	pub min: Vector3,
	pub max: Vector3,
}

impl BoundingBox {
	pub fn empty() -> BoundingBox {
		BoundingBox {
			min: Vector3::INFINITY(),
			max: Vector3::INFINITY() * -1.0,
		}
	}

	pub fn new(min: Vector3, max: Vector3) -> BoundingBox {
		BoundingBox { min: min, max: max }
	}
}

pub fn childBoundingBoxOf(min: &Vector3, max: &Vector3, index: usize) -> BoundingBox {
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
