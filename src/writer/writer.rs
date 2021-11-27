use crate::indexing::indexing::Indexer;
use crate::indexing::model::Node;
use std::fs;
use std::fs::File;
use std::io::BufReader;

pub struct Writer<'a> {
	indexer: &'a Indexer,
	capacity: i64,

	// copy node data here first
	activeBuffer: Vec<u8>,

	// backlog of buffers that reached capacity and are ready to be written to disk
	backlog: Vec<Vec<u8>>,

	close_requested: bool,
	closed: bool,
	// std::condition_variable cvClose;
	fsOctree: BufReader<File>,
}

impl Writer<'_> {
	pub fn new(indexer: &Indexer) -> Writer {
		let file = File::open("foo.txt").unwrap();
		let mut buf_reader = BufReader::new(file);
		Writer {
			indexer,
			capacity: 16 * 1024 * 1024,
			close_requested: false,
			closed: false,
			activeBuffer: Vec::new(),
			backlog: Vec::new(),
			fsOctree: buf_reader,
		}
	}

	fn writeAndUnload(node: &Node) {}

	fn launchWriterThread() {}
	fn closeAndWait() {}
}
