use crate::chunking::model::ChunkingMetadata;
use std::fs;

pub mod indexing;
pub mod model;
pub mod hierarchy;

pub fn read_chunking_metadata(path: String) -> ChunkingMetadata {
	let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
	serde_json::from_str(&contents).unwrap()
}
