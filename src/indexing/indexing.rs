use crate::indexing::hierarchy::create_hierarchy;
use crate::indexing::model::childBoundingBoxOf;
use crate::indexing::model::BoundingBox;
use crate::indexing::model::Chunk;
use crate::indexing::model::Chunks;
use crate::indexing::model::Hierarchy;
use crate::indexing::model::Vector3;
use crate::indexing::model::{HierarchyChunk, Node, Type};
use crate::indexing::read_chunking_metadata;
use crate::model::attributes::Attribute;
use crate::model::attributes::{AttributeType, Attributes};
use crate::model::metadata::Attribute as JsonAttribute;
use crate::model::metadata::Metadata;
use crate::model::options::Options;
use crate::model::State;
use crate::sampling::Sampler;
use std::fs;
use std::fs::File;

pub struct Indexer {
	targetDir: String,
	options: Options,
	attributes: Attributes,
	root: Box<Node>,
	spacing: f64,

	bytesInMemory: u64,
	octree_depth: u8,
}

impl Indexer {}

fn getChunks(pathIn: &String) -> Chunks {
	let chunkDirectory = pathIn.to_string() + "/chunks";

	let metadata = read_chunking_metadata(chunkDirectory.to_string() + "/metadata.json");

	let metadata_min = Vector3 {
		x: metadata.min[0],
		y: metadata.min[1],
		z: metadata.min[2],
	};

	let metadata_max = Vector3 {
		x: metadata.max[0],
		y: metadata.max[1],
		z: metadata.max[2],
	};

	let mut attribute_list: Vec<Attribute> = Vec::new();
	for attr in metadata.attributes {
		let JsonAttribute {
			name,
			description,
			size,
			num_elements,
			element_size,
			r#type,
			min,
			max,
		} = attr;
		let mut attribute = Attribute {
			name,
			size: size.into(),
			num_elements: num_elements.into(),
			element_size: element_size.into(),
			r#type: AttributeType::UNDEFINED,
			description,
			min: Vector3::INFINITY(),
			max: Vector3::INFINITY(),
		};

		if num_elements >= 1 {
			attribute.min.x = min[0];
			attribute.max.x = max[0];
		}
		if num_elements >= 2 {
			attribute.min.y = min[1];
			attribute.max.y = max[1];
		}
		if num_elements >= 3 {
			attribute.min.z = min[2];
			attribute.max.z = max[2];
		}

		attribute_list.push(attribute);
	}

	let mut attributes = Attributes::from_attributes(attribute_list);
	attributes.posScale = Vector3 {
		x: metadata.scale[0],
		y: metadata.scale[1],
		z: metadata.scale[2],
	};
	attributes.posOffset = Vector3 {
		x: metadata.offset[0],
		y: metadata.offset[1],
		z: metadata.offset[2],
	};

	fn toID(filename: String) -> String {
		filename.replace("chunk_", "").replace(".bin", "")
	}

	let mut chunksToLoad: Vec<Box<Chunk>> = Vec::new();
	for entry in fs::read_dir(chunkDirectory).unwrap() {
		let entry = entry.unwrap();
		let filename = entry
			.path()
			.file_name()
			.unwrap()
			.to_str()
			.unwrap()
			.to_string();
		let path = entry.path().to_str().unwrap().to_string();
		let chunkID = toID(filename.to_string());

		if !filename.ends_with(".bin") {
			continue;
		}

		let mut chunk = Chunk::new(path, chunkID.to_string());

		let mut bbox = BoundingBox {
			min: metadata_min.clone(),
			max: metadata_max.clone(),
		};

		for i in 1..chunkID.chars().count() {
			let index = chunkID.chars().nth(i).unwrap() as usize - '0' as usize;

			bbox = childBoundingBoxOf(&bbox.min, &bbox.max, index);
		}

		chunk.min = bbox.min;
		chunk.max = bbox.max;

		chunksToLoad.push(Box::new(chunk));
	}

	Chunks {
		list: chunksToLoad,
		min: metadata_min,
		max: metadata_max,
		attributes,
	}
}

// 1. Counter grid
// 2. Hierarchy from counter grid
// 3. identify nodes that need further refinment
// 4. Recursively repeat at 1. for identified nodes
fn buildHierarchy(indexer: &Indexer, node: &Node, points: Vec<u8>, num_points: i64, depth: i64) {}

fn do_indexing(targetDir: String, state: &mut State, options: Options, sampler: Sampler) {
	state.name = "INDEXING".to_string();
	// state.currentPass = 3;
	// state.pointsProcessed = 0;
	// state.bytesProcessed = 0;
	// state.duration = 0;

	let chunks = getChunks(&targetDir);
	let attributes = chunks.attributes;

	let bytes_per_point = attributes.bytes;
	let mut indexer = Indexer {
		targetDir,
		options,
		attributes,
		spacing: (&chunks.max - &chunks.min).x / 128.0,
		root: Box::new(Node::new("r".to_string(), chunks.min, chunks.max)),
		bytesInMemory: 0,
		octree_depth: 0,
	};

	let mut totalPoints = 0;
	let mut totalBytes = 0;
	for chunk in &chunks.list {
		let filesize = fs::metadata(&chunk.file).unwrap().len();
		totalPoints += filesize as i32 / bytes_per_point;
		totalBytes += filesize;
	}

	let pointsProcessed = 0;

	let activeThreads = 0;
	let mut nodes: Vec<Box<Node>> = Vec::new();
	for chunk in &chunks.list {
		let chunkRoot = Box::new(Node::new(
			chunk.id.clone(),
			chunk.min.clone(),
			chunk.max.clone(),
		));

		let filesize = fs::metadata(&chunk.file).unwrap().len();

		indexer.bytesInMemory += filesize;
		let pointBuffer = fs::read(&chunk.file).unwrap();

		if !indexer.options.keep_chunks {
			fs::remove_file(&chunk.file);
		}

		let num_points = (pointBuffer.len() as i32 / bytes_per_point) as i64;

		buildHierarchy(&indexer, &chunkRoot, pointBuffer, num_points, 0);

		// let onNodeCompleted = |node| {
		// 	indexer.writer.writeAndUnload(node);
		// };

		// sampler.sample(chunkRoot, attributes, indexer.spacing, onNodeCompleted);

		// indexer.flushChunkRoot(chunkRoot);

		// add chunk root, provided it isn't the root.
		if chunkRoot.name.len() > 1 {
			indexer.root.addDescendant(chunkRoot);
		}

		// nodes.push(chunkRoot);
	}

	// indexer.reloadChunkRoots();

	if chunks.list.len() == 1 {
		// let node = nodes[0];

		// indexer.root = node;
	} else {
		// let onNodeCompleted = |node| {
		// 	indexer.writer.writeAndUnload(node);
		// };

		// sampler.sample(indexer.root, attributes, indexer.spacing, onNodeCompleted);
	}
	// indexer.writer->writeAndUnload(indexer.root.get());

	// indexer.writer->closeAndWait();

	let hierarchy_path = indexer.targetDir.to_string() + "/hierarchy.bin";
	let hierarchy = create_hierarchy(&indexer.root);
	fs::write(&hierarchy_path, &hierarchy.buffer);

	let metadata_path = indexer.targetDir.to_string() + "/metadata.json";
	let metadata = Metadata::create(
		&indexer.root,
		&indexer.attributes,
		&indexer.options,
		state,
		hierarchy,
		indexer.spacing,
		indexer.octree_depth,
	);
	serde_json::to_writer(&File::create(metadata_path).unwrap(), &metadata);

	{
		// delete chunk directory
		if indexer.options.keep_chunks {
			let chunks_metadata_path = format!("{}/chunks/metadata.json", indexer.targetDir);

			fs::remove_file(chunks_metadata_path);
			fs::remove_file(format!("{}/chunks", indexer.targetDir));
		}

		// delete chunk roots data
		let octreePath = format!("{}/tmpChunkRoots.bin", indexer.targetDir);
		fs::remove_file(octreePath);
	}
}
