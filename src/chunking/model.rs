use crate::model::metadata::Attribute;
use serde::{Deserialize, Serialize};

const MAX_POINTS_PER_CHUNK: i32 = 5_000_000;
const GRID_SIZE: i32 = 128;

struct Point {
	x: f64,
	y: f64,
	z: f64,
}

pub struct Node {
	id: String,
	level: i64,
	x: i64,
	y: i64,
	z: i64,
	size: i64,
	num_points: i64,
}

impl Node {
	fn new(id: String, num_points: i64) -> Node {
		Node {
			id,
			num_points,
			level: 0,
			size: 0,
			x: 0,
			y: 0,
			z: 0,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct ChunkingMetadata {
	pub min: [f64; 3],
	pub max: [f64; 3],
	pub offset: [f64; 3],
	pub scale: [f64; 3],
	pub attributes: Vec<Attribute>
}
