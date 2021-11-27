use serde::{Deserialize, Serialize};

pub struct Options {
	pub keep_chunks: bool,
	pub name: String,
	pub encoding: Encoding
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone)]
pub enum Encoding {
	DEFAULT,
	BROTLI,
}