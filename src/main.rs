pub mod chunking;
pub mod indexing;
mod model;
mod potree;
mod reader;
mod sampling;
pub mod writer;
pub mod hierarchy;

fn main() {
	reader::read()
}
