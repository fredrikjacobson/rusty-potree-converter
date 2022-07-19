mod utils;

use serde_json::from_slice;
use rusty_potree_converter::model::vector3::Vector3;
use crate::utils::set_panic_hook;
use std::error::Error;
use serde_json::Value;
use rusty_potree_converter::model::hierarchy::create_hierarchy;
use rusty_potree_converter::writer::{Writer, WriteError, create_metadata};
use rusty_potree_converter::potree::Potree;
use wasm_bindgen::prelude::*;
use std::io::Write;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
pub fn process_array_buffer(file_type: &str, buffer: &[u8]) -> Result<PotreeData, JsError> {
	set_panic_hook();

	let potree = rusty_potree_converter::pcd_reader::from_pcd(buffer).expect("To parse pcd");
	
	let potree_data = write_potree_to_struct(potree).expect("To Get Potree Struct");
	std::mem::forget(&potree_data);
	Ok(potree_data)
}

#[wasm_bindgen]
pub struct PotreeData {
    octree: Vec<u8>,
    hierarchy: Vec<u8>,
    metadata: Vec<u8>,
}

#[wasm_bindgen]
impl PotreeData {
	pub fn get_metadata(&self) -> Result<JsValue, JsError> {
		let untyped_json: Value = serde_json::from_slice::<Value>(&self.metadata)?;
		let res = JsValue::from_serde(&untyped_json)?;
		Ok(res)
	}
	pub fn get_hierarchy(&self) -> js_sys::Uint8Array {
		return js_sys::Uint8Array::from(&self.hierarchy[..]);
	}
	pub fn get_octree(&self) -> js_sys::Uint8Array {
		return js_sys::Uint8Array::from(&self.octree[..]);
	}
}

pub fn write_potree_to_struct(potree: Potree) -> Result<PotreeData, Box<dyn Error>> {
    let mut octree_data: Vec<u8> = Vec::new();
    let mut writer = Writer::new(&mut octree_data);
    writer.write(&potree);

    let mut hierarchy_data: Vec<u8> = Vec::new();
    let hierarchy = create_hierarchy(&potree.root, writer.node_hierarchy);
    hierarchy_data.write(&hierarchy.buffer)?;

    let mut metadata_data: Vec<u8> = Vec::new();
    let metadata = create_metadata(&potree, &hierarchy);
    serde_json::to_writer(&mut metadata_data, &metadata)?;

    Ok(PotreeData {
        hierarchy: hierarchy_data,
        metadata: metadata_data,
        octree: octree_data,
    })
}


#[cfg(test)]
mod tests {

    use serde_json::Value;
use crate::write_potree_to_struct;
use rusty_potree_converter::model::metadata::Metadata;
	use rusty_potree_converter::model::vector3::Vector3;
    use rusty_potree_converter::potree::Potree;
    use byteorder::LittleEndian;
    use byteorder::ReadBytesExt;
    use std::fs;
    use std::io::Cursor;

    #[test]
    fn test_write_to_struct() {
        let buffer = fs::read("../resources/points.bin").unwrap();
        let mut cursor = Cursor::new(buffer);
        let mut points: Vec<Vector3> = Vec::new();
        let num_points = 100;

		let record_size = 8 * 3;
        while cursor.position() < (num_points * record_size) as u64 {
            points.push(Vector3 {
                x: cursor.read_f64::<LittleEndian>().unwrap(),
                y: cursor.read_f64::<LittleEndian>().unwrap(),
                z: cursor.read_f64::<LittleEndian>().unwrap(),
            })
        }
        let potree = Potree::new(points, 20000);

        let potree_data = write_potree_to_struct(potree).unwrap();

        let points_written = potree_data.octree.len() / (3 * 4);
        assert_eq!(num_points as usize, points_written);
		let metadata = serde_json::from_slice::<Metadata>(&potree_data.metadata).unwrap();
		
		assert_eq!(metadata.points, 100);
		assert_eq!(metadata.version, "2.0");
    }

	
}
