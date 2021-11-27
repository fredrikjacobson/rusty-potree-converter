use crate::indexing::model::Node;
use crate::indexing::model::{Hierarchy, HierarchyChunk, Type};
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem;

fn sort_breadth_first(nodes: &mut Vec<&Node>) {
	nodes.sort_by(|a, b| {
		if a.name.len() != b.name.len() {
			a.name.len().cmp(&b.name.len())
		} else {
			a.name.cmp(&b.name)
		}
	});
}

fn child_mask_of(node: &Node) -> u8 {
	let mut mask: u8 = 0;

	for i in 0..8 {
		let child = &node.children[i];

		if let Some(child) = child {
			mask = mask | (1 << i);
		}
	}

	mask
}

fn gather_chunk(start: &Node, levels: usize) -> HierarchyChunk {
	// create vector containing start node and all descendants up to and including levels deeper
	// e.g. start 0 and levels 5 -> all nodes from level 0 to inclusive 5.
	let start_level = start.name.len() - 1;

	let mut chunk = HierarchyChunk {
		name: start.name.to_string(),
		nodes: Vec::new(),
	};
	let mut stack = vec![start];
	while !stack.is_empty() {
		let node = stack.pop().unwrap();
		chunk.nodes.push(node);
		let child_level = node.name.len();
		if child_level <= start_level + levels {
			for child in &node.children {
				if let Some(child) = child {
					stack.push(&child);
				}
			}
		}
	}
	return chunk;
}

fn create_hierarchy_chunks(root: &Node, hierarchy_step_size: usize) -> Vec<HierarchyChunk> {
	let mut hierarchy_chunks = Vec::new();
	let mut stack = vec![root];
	while !stack.is_empty() {
		let chunk_root = stack.pop().unwrap();

		let chunk = gather_chunk(chunk_root, hierarchy_step_size);

		for node in &chunk.nodes {
			let isProxy = node.level() == chunk_root.level() + hierarchy_step_size;

			if isProxy {
				stack.push(node);
			}
		}

		hierarchy_chunks.push(chunk);
	}

	hierarchy_chunks
}

pub fn create_hierarchy(root: &Node) -> Hierarchy {
	const HIERARCHY_STEP_SIZE: u8 = 4;
	// type + childMask + numPoints + offset + size
	const BYTES_PER_NODE: usize = 1 + 1 + 4 + 8 + 8;

	fn chunkSize(chunk: &HierarchyChunk) -> usize {
		chunk.nodes.len() * BYTES_PER_NODE
	}
	let mut chunks = create_hierarchy_chunks(root, HIERARCHY_STEP_SIZE.into());
	let mut chunkPointers: HashMap<String, usize> = HashMap::new();
	let mut chunkByteOffsets = vec![0; chunks.len()];
	let mut hierarchyBufferSize = 0;
	for i in 0..chunks.len() {
		{
			let chunk = &mut chunks[i];
			chunkPointers.insert(chunk.name.clone(), i);
			sort_breadth_first(&mut chunk.nodes);
		}
		if i >= 1 {
			chunkByteOffsets[i] = chunkByteOffsets[i - 1] + chunkSize(&chunks[i - 1]);
		}
		hierarchyBufferSize += chunkSize(&chunks[i])
	}

	let mut hierarchy_buffer: Vec<u8> = vec![0; hierarchyBufferSize];
	let mut offset = 0;
	for i in 0..chunks.len() {
		let chunk = &chunks[i];
		let chunk_level = chunk.name.len() - 1;
		for node in &chunk.nodes {
			let is_proxy = node.level() == (chunk_level + usize::from(HIERARCHY_STEP_SIZE));
			let child_mask = child_mask_of(&node);
			let target_offset: u64;
			let target_size: u64;
			let num_points: u32 = node.numPoints.try_into().unwrap();
			let mut node_type: u8 = if node.isLeaf() {
				Type::Leaf as u8
			} else {
				Type::Normal as u8
			};
			if is_proxy {
				let target_chunk_index = chunkPointers[&node.name];
				let target_chunk = &chunks[target_chunk_index];
				node_type = Type::Proxy as u8;
				target_offset = chunkByteOffsets[target_chunk_index] as u64;
				target_size = chunkSize(target_chunk) as u64;
			} else {
				target_offset = node.byteOffset as u64;
				target_size = node.byteSize as u64;
			}
			hierarchy_buffer[(offset + 0)..(1 + offset + 0)].copy_from_slice(&[node_type]);
			let mut bs = [0u8; mem::size_of::<u32>()];
			bs.as_mut()
				.write_u32::<LittleEndian>(num_points)
				.expect("Unable to write");
			hierarchy_buffer[(offset + 1)..(1 + offset + 1)].copy_from_slice(&[child_mask]);
			hierarchy_buffer[(offset + 2)..(4 + offset + 2)].copy_from_slice(&bs);
			let mut bs = [0u8; mem::size_of::<u64>()];
			bs.as_mut()
				.write_u64::<LittleEndian>(target_offset)
				.expect("Unable to write");
			hierarchy_buffer[(offset + 6)..(8 + offset + 6)].copy_from_slice(&bs);
			bs.as_mut()
				.write_u64::<LittleEndian>(target_size)
				.expect("Unable to write");
			hierarchy_buffer[(offset + 14)..(8 + offset + 14)].copy_from_slice(&bs);
			offset += BYTES_PER_NODE;
		}
	}
	Hierarchy {
		step_size: HIERARCHY_STEP_SIZE,
		buffer: hierarchy_buffer,
		first_chunk_size: (chunks[0].nodes.len() * BYTES_PER_NODE) as i64,
	}
}
