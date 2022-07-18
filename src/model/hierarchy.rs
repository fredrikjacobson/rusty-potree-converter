use byteorder::{LittleEndian, WriteBytesExt};
use std::cmp::max;
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem;

use super::node::Node;

#[repr(u8)]
pub enum Type {
    Normal = 0,
    Leaf = 1,
    Proxy = 2,
}

pub struct Hierarchy {
    pub step_size: u8,
    pub buffer: Vec<u8>,
    pub first_chunk_size: i64,
    pub depth: u8,
}

struct HierarchyChunk<'a> {
    pub name: String,
    pub nodes: Vec<&'a Node>,
}

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

        if let Some(_child) = child {
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
            let is_proxy = node.level() == chunk_root.level() + hierarchy_step_size;

            if is_proxy {
                stack.push(node);
            }
        }

        hierarchy_chunks.push(chunk);
    }

    hierarchy_chunks
}

pub fn create_hierarchy(root: &Node, node_hierarchy: HashMap<String, (u32, u32)>) -> Hierarchy {
    const HIERARCHY_STEP_SIZE: u8 = 4;
    // type + childMask + numPoints + offset + size
    const BYTES_PER_NODE: usize = 1 + 1 + 4 + 8 + 8;

    fn chunk_size(chunk: &HierarchyChunk) -> usize {
        chunk.nodes.len() * BYTES_PER_NODE
    }
    let mut chunks = create_hierarchy_chunks(root, HIERARCHY_STEP_SIZE.into());
    let mut chunk_pointers: HashMap<String, usize> = HashMap::new();
    let mut chunk_byte_offsets = vec![0; chunks.len()];
    let mut hierarchy_buffer_size = 0;
    let mut depth = 0;
    for i in 0..chunks.len() {
        {
            let chunk = &mut chunks[i];
            chunk_pointers.insert(chunk.name.clone(), i);
            sort_breadth_first(&mut chunk.nodes);
        }
        if i >= 1 {
            chunk_byte_offsets[i] = chunk_byte_offsets[i - 1] + chunk_size(&chunks[i - 1]);
        }
        hierarchy_buffer_size += chunk_size(&chunks[i])
    }

    let mut hierarchy_buffer: Vec<u8> = vec![0; hierarchy_buffer_size];
    let mut offset = 0;
    for i in 0..chunks.len() {
        let chunk = &chunks[i];
        let chunk_level = chunk.name.len() - 1;
        for node in &chunk.nodes {
            let is_proxy = node.level() == (chunk_level + usize::from(HIERARCHY_STEP_SIZE));
            let child_mask = child_mask_of(&node);
            let target_offset: u64;
            let target_size: u64;
            let num_points: u32 = node.num_points().try_into().unwrap();
            let mut node_type: u8 = if node.is_leaf_node() {
                Type::Leaf as u8
            } else {
                Type::Normal as u8
            };
            if is_proxy {
                let target_chunk_index = chunk_pointers[&node.name];
                let target_chunk = &chunks[target_chunk_index];
                node_type = Type::Proxy as u8;
                target_offset = chunk_byte_offsets[target_chunk_index] as u64;
                target_size = chunk_size(target_chunk) as u64;
            } else {
                let (byte_size, byte_offset) = node_hierarchy[&node.name];
                target_offset = byte_offset as u64;
                target_size = byte_size as u64;
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
            depth = max(node.level(), depth);
        }
    }
    Hierarchy {
        step_size: HIERARCHY_STEP_SIZE,
        buffer: hierarchy_buffer,
        first_chunk_size: (chunks[0].nodes.len() * BYTES_PER_NODE) as i64,
        depth: depth as u8,
    }
}
