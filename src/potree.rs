use crate::indexing::model::Vector3;
use ord_subset::OrdSubsetIterExt;
use std::slice::Iter;

pub struct Potree {
	pub bounds: Bounds,
	cubic_bounds: Bounds,
	size_len: f64,

	point_per_leaf_node_limit: u32,

	pub spacing: f64,
	pub scale: f64,
	pub size: u32,

	pub root: Node,
}
const DIAGONAL_FRACTION: f64 = 200.0;

impl Potree {
	fn new(points: Vec<Vector3>, bounds: Bounds, point_per_leaf_node_limit: u32) -> Potree {
		let cubic_bounds = bounds.cubic();
		let size_len = ((cubic_bounds.size_x * cubic_bounds.size_x)
			+ (cubic_bounds.size_y * cubic_bounds.size_y)
			+ (cubic_bounds.size_z * cubic_bounds.size_z))
			.sqrt();
		let spacing = size_len / DIAGONAL_FRACTION;

		let mut root_node = Node::new(
			"r".to_string(),
			spacing,
			bounds.clone(),
			emptyChildNodeArray(),
			point_per_leaf_node_limit,
		);

		let size = points.len() as u32;

		for point in points {
			root_node.add_point(point);
		}

		Potree {
			spacing,
			scale: if size_len > 1_000_000.0 {
				0.01
			} else if size_len > 1.0 {
				0.001
			} else {
				0.0001
			},
			size,
			bounds,
			cubic_bounds,
			size_len,
			point_per_leaf_node_limit,
			root: root_node,
		}
	}
}

pub struct Node {
	spacing: f64,
	pub bounds: Bounds,
	pub children: [Option<Box<Node>>; 8],
	max_points_per_leaf_node: u32,
	grid: NodeGrid,
	initial_store: Vec<Vector3>,
	squared_spacing: f64,
	pub name: String,
	pub byte_size: u32,
	pub byte_offset: u32,
}

impl Node {
	fn new(
		name: String,
		spacing: f64,
		bounds: Bounds,
		children: [Option<Box<Node>>; 8],
		max_points_per_leaf_node: u32,
	) -> Node {
		Node {
			name,
			spacing: spacing,
			squared_spacing: spacing * spacing,
			max_points_per_leaf_node,
			bounds: bounds,
			children,
			grid: emptyGridArray(),
			initial_store: Vec::new(),
			byte_offset: 0,
			byte_size: 0,
		}
	}

	pub fn level(&self) -> usize {
		return self.name.len() - 1;
	}

	pub fn is_leaf_node(&self) -> bool {
		self.children.iter().all(|child| child.is_none())
	}

	pub fn points(&self) -> Vec<&Vector3> {
		if self.is_leaf_node() {
			return self.initial_store.iter().collect();
		} else {
			let mut point_vectors: Vec<&Vector3> = Vec::new();
			for i in 0..8 {
				for j in 0..8 {
					point_vectors.append(&mut self.grid[i][j].iter().collect());
				}
			}

			return point_vectors;
		}
	}

	pub fn num_points(&self) -> usize {
		if self.is_leaf_node() {
			self.initial_store.len()
		} else {
			let mut count = 0;
			for row in &self.grid {
				for cell in row {
					count += cell.len();
				}
			}
			count
		}
	}

	fn add_point(&mut self, point: Vector3) {
		if self.is_leaf_node() {
			self.initial_store.push(point.clone());
			if self.initial_store.len() >= self.max_points_per_leaf_node as usize {
				let index = Node::find_grid_index(&point, &self.bounds);
				self.split(index.into())
			}
		} else {
			let grid_index_outer = Node::find_grid_index(&point, &self.bounds);
			let grid_index_inner =
				Node::find_grid_index(&point, &self.computeChildBounds(grid_index_outer));
			if self.grid[grid_index_outer][grid_index_inner]
				.iter()
				.any(|p| Node::within_distance(&p, &point, self.squared_spacing))
			{
				match &mut self.children[grid_index_outer] {
					None => {
						let mut new_child = Box::new(self.new_child_node(grid_index_outer));
						new_child.add_point(point);
						self.children[grid_index_outer] = Some(new_child);
					}
					Some(ref mut child) => child.add_point(point),
				};
			} else {
				self.grid[grid_index_outer][grid_index_inner].push(point);
			}
		}
	}

	fn split(&mut self, index: usize) {
		self.children[index] = Some(Box::new(self.new_child_node(index)));
		// We should probably not clone here and deal with how to do this better
		for point in &self.initial_store.clone() {
			self.add_point(point.clone());
		}
		self.initial_store = Vec::new();
	}

	fn new_child_node(&self, index: usize) -> Node {
		Node::new(
			format!("{}{}", &self.name, index.to_string()),
			self.spacing / 2.0,
			self.computeChildBounds(index),
			emptyChildNodeArray(),
			self.max_points_per_leaf_node,
		)
	}

	fn within_distance(a: &Vector3, b: &Vector3, squared_distance: f64) -> bool {
		let x_diff = (a.x - b.x) * (a.x - b.x);
		let y_diff = (a.y - b.y) * (a.y - b.y);
		let z_diff = (a.z - b.z) * (a.z - b.z);
		x_diff + y_diff + z_diff < squared_distance
	}
	fn find_grid_index(point: &Vector3, bounds: &Bounds) -> usize {
		let low_x = point.x < (bounds.lx + bounds.ux) / 2.0; //lower than mid x
		let low_y = point.y < (bounds.ly + bounds.uy) / 2.0; //lower than mid y
		let low_z = point.z < (bounds.lz + bounds.uz) / 2.0; //lower than mid z
		if low_x && low_y && low_z {
			0
		} else if low_x && low_y && !low_z {
			1
		} else if low_x && !low_y && low_z {
			2
		} else if low_x && !low_y && !low_z {
			3
		} else if !low_x && low_y && low_z {
			4
		} else if !low_x && low_y && !low_z {
			5
		} else if !low_x && !low_y && low_z {
			6
		} else {
			7
		}
	}
	fn computeChildBounds(&self, index: usize) -> Bounds {
		let bounds = &self.bounds;
		let boundsMidX = (bounds.lx + bounds.ux) / 2.0;
		let boundsMidY = (bounds.ly + bounds.uy) / 2.0;
		let boundsMidZ = (bounds.lz + bounds.uz) / 2.0;
		if index == 0 {
			Bounds::new(
				boundsMidX, boundsMidY, boundsMidZ, bounds.lx, bounds.ly, bounds.lz,
			)
		} else if index == 1 {
			Bounds::new(
				boundsMidX, boundsMidY, bounds.uz, bounds.lx, bounds.ly, boundsMidZ,
			)
		} else if index == 2 {
			Bounds::new(
				boundsMidX, bounds.uy, boundsMidZ, bounds.lx, boundsMidY, bounds.lz,
			)
		} else if index == 3 {
			Bounds::new(
				boundsMidX, bounds.uy, bounds.uz, bounds.lx, boundsMidY, boundsMidZ,
			)
		} else if index == 4 {
			Bounds::new(
				bounds.ux, boundsMidY, boundsMidZ, boundsMidX, bounds.ly, bounds.lz,
			)
		} else if index == 5 {
			Bounds::new(
				bounds.ux, boundsMidY, bounds.uz, boundsMidX, bounds.ly, boundsMidZ,
			)
		} else if index == 6 {
			Bounds::new(
				bounds.ux, bounds.uy, boundsMidZ, boundsMidX, boundsMidY, bounds.lz,
			)
		} else {
			Bounds::new(
				bounds.ux, bounds.uy, bounds.uz, boundsMidX, boundsMidY, boundsMidZ,
			)
		}
	}
}

type GridRow = [Vec<Vector3>; 8];
type NodeGrid = [GridRow; 8];

fn emptyChildNodeArray() -> [Option<Box<Node>>; 8] {
	[None, None, None, None, None, None, None, None]
}

fn emptyGridArray() -> NodeGrid {
	[
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
		emptyGridRow(),
	]
}

fn emptyGridRow() -> GridRow {
	[
		Vec::new(),
		Vec::new(),
		Vec::new(),
		Vec::new(),
		Vec::new(),
		Vec::new(),
		Vec::new(),
		Vec::new(),
	]
}

fn findBounds(points: &Vec<Vector3>) -> Bounds {
	let xs: Vec<f64> = points.iter().map(|p| p.x).collect();
	let ys: Vec<f64> = points.iter().map(|p| p.y).collect();
	let zs: Vec<f64> = points.iter().map(|p| p.z).collect();

	Bounds::new(
		*xs.iter().ord_subset_max().unwrap(),
		*ys.iter().ord_subset_max().unwrap(),
		*zs.iter().ord_subset_max().unwrap(),
		*xs.iter().ord_subset_min().unwrap(),
		*ys.iter().ord_subset_min().unwrap(),
		*zs.iter().ord_subset_min().unwrap(),
	)
}

#[derive(Clone)]
pub struct Bounds {
	size_x: f64,
	size_y: f64,
	size_z: f64,
	pub ux: f64,
	pub uy: f64,
	pub uz: f64,
	pub lx: f64,
	pub ly: f64,
	pub lz: f64,
}

impl Bounds {
	fn new(ux: f64, uy: f64, uz: f64, lx: f64, ly: f64, lz: f64) -> Bounds {
		Bounds {
			size_x: (lx - ux).abs(),
			size_y: (ly - uy).abs(),
			size_z: (lz - uz).abs(),
			ux,
			uy,
			uz,
			lx,
			ly,
			lz,
		}
	}

	fn cubic(&self) -> Bounds {
		let max_size = [self.size_x, self.size_y, self.size_z]
			.iter()
			.cloned()
			.fold(f64::NEG_INFINITY, f64::max);
		let new_ux = self.lx + max_size;
		let new_uy = self.ly + max_size;
		let new_uz = self.lz + max_size;
		Bounds::new(new_ux, new_uy, new_uz, self.lx, self.ly, self.lz)
	}
}

#[cfg(test)]
mod tests {
	use crate::hierarchy::create_hierarchy;
	use crate::potree::findBounds;
	use crate::potree::GridRow;
	use crate::potree::Node;
	use crate::potree::NodeGrid;
	use crate::potree::Potree;
	use crate::potree::Vector3;
	use crate::writer::write_potree;
	use rand::prelude::*;
	use std::collections::HashMap;
	use std::fs;
	use std::path::Path;

	fn setup_potree(point_count: u32, node_limit: u32) -> Potree {
		let mut rng = rand::thread_rng();
		let y: f64 = rng.gen();

		let point_per_leaf_node_limit = node_limit;
		let mut points = Vec::new();
		for i in 0..point_count {
			points.push(Vector3 {
				x: rng.gen_range(0.0..100.0),
				y: rng.gen_range(0.0..100.0),
				z: rng.gen_range(0.0..10.0),
			});
		}

		let bounds = findBounds(&points);
		Potree::new(points, bounds, point_per_leaf_node_limit)
	}

	#[test]
	fn node_points_returns_points() {
		let count = 10_000;
		let potree = setup_potree(count, 1000);

		assert_eq!(potree.root.num_points(), count as usize);
	}

	#[test]
	fn test_write_potree() {
		let count = 10_000;
		let potree = setup_potree(count, 1000);
		let dir = Path::new("/Users/fredrikjacobson/stuff/rusty-potree-converter/test-output");
		write_potree(potree, dir);

		let file_size = fs::metadata(dir.join("octree.bin")).unwrap().len();
		let points_written = file_size / (3 * 4);
		assert_eq!(count as u64, points_written);
	}

	// #[test]
	// fn test_write_hierarchy() {
	// 	let potree = setup_potree(10_000, 1000);
	// 	let mut fake_hierarchy = HashMap::new();
	// 	fn add_node(map: &mut HashMap<String, (u32, u32)>, node: &Node) {
	// 		map.insert(node.name.to_string(), (0, 0));
	// 		for child in &node.children {
	// 			if let Some(child) = child {
	// 				add_node(map, &child);
	// 			}
	// 		}
	// 	}

	// 	fake_hierarchy.insert(potree.root.name.to_string(), (0, 0));
	// 	for child in &potree.root.children {
	// 		if let Some(child) = child {
	// 			add_node(&mut fake_hierarchy, child);
	// 		}
	// 	}
	// 	create_hierarchy(&potree.root, fake_hierarchy);
	// }

	fn print_grid(grid: &NodeGrid) -> String {
		let mut rows = Vec::new();
		for row in 0..8 {
			rows.push(format!(
				"[{}, {}, {}, {}, {}, {}, {}, {}]",
				grid[row][0].len(),
				grid[row][1].len(),
				grid[row][2].len(),
				grid[row][3].len(),
				grid[row][4].len(),
				grid[row][5].len(),
				grid[row][6].len(),
				grid[row][7].len(),
			));
		}

		rows.join("\n")
	}

	fn print_node(node: &Node, level: u8) {
		println!(
			"{} Node with store {} is_leaf {} count {}",
			"   ".repeat(level as usize),
			node.initial_store.len(),
			node.is_leaf_node(),
			node.num_points()
		);
		println!("{}", print_grid(&node.grid));
		let mut i = 0;
		for child in &node.children {
			if let Some(child) = child {
				print_node(&child, level + 1);
			} else {
			}
			i += 1;
		}
	}
}