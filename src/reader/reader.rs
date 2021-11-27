use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::convert::TryInto;
use std::fs;
use std::io::Cursor;

use crate::model::metadata::Metadata;

pub fn read_metadata(path: String) -> Metadata {
	let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
	serde_json::from_str(&contents).unwrap()
}

pub fn read() {
	let contents = fs::read_to_string("resources/metadata.json")
		.expect("Something went wrong reading the file");
	let metadata: Metadata = serde_json::from_str(&contents).unwrap();

	let mut skip_bytes = 0;
	for attr in metadata.attributes {
		println!("{}", attr.name);
		if attr.name != "position" {
			skip_bytes += attr.size;
		}
	}
	println!("{:?}", skip_bytes);
	println!("{:?}", metadata.scale);

	let octree = read_bytes();
	let size = octree.len();

	let mut xyz: Vec<[i32; 3]> = Vec::new();
	let mut rdr = Cursor::new(octree);
	while rdr.position() < size.try_into().unwrap() {
		let row = [
			rdr.read_i32::<LittleEndian>().unwrap(),
			rdr.read_i32::<LittleEndian>().unwrap(),
			rdr.read_i32::<LittleEndian>().unwrap(),
		];

		xyz.push(row);
		rdr.set_position(rdr.position() + u64::from(skip_bytes));
	}

	println!(
		"{:?}",
		f64::from(xyz[0][0]) * metadata.scale[0] + metadata.offset[0]
	);
	println!(
		"{:?}",
		f64::from(xyz[0][1]) * metadata.scale[1] + metadata.offset[1]
	);
	println!(
		"{:?}",
		f64::from(xyz[0][2]) * metadata.scale[2] + metadata.offset[2]
	);
}

fn read_bytes() -> Vec<u8> {
	fs::read("resources/octree.bin").expect("Unable to read file")
}
