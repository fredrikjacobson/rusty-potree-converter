use csv::{Reader};
use serde::{Deserialize};

use crate::{model::vector3::Vector3, potree::Potree};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Intensity  {
	Int(u8),
	Float(f32)
}

#[derive(Debug, Deserialize)]
struct Point {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub intensity: Option<Intensity>
}

pub fn from_csv(buf: &[u8]) -> Result<Potree, Box<dyn std::error::Error>> {
    let mut rdr = Reader::from_reader(buf);
	let mut points: Vec<Vector3> = Vec::new();
	let mut num_points = 0;
    for result in rdr.deserialize() {
        let record: Point = result?;
		  
		let Point { x, y, z, intensity } = record;
		points.push(Vector3 { x, y, z });

		num_points = num_points + 1;
    }
	let potree = Potree::new(points, 20000);

    Ok(potree)
}


#[cfg(test)]
mod tests {

	use std::fs;
	use crate::csv_reader;
   
    #[test]
    fn test_read_csv() -> Result<(), Box<dyn std::error::Error>> {
		let buffer = fs::read("resources/points_integer_intensity.csv")?;
		let potree = csv_reader::from_csv(&buffer)?;

		assert_eq!(potree.size, 10);

		Ok(())
    }
   
    #[test]
    fn test_read_csv_intensity() -> Result<(), Box<dyn std::error::Error>> {
		let buffer = fs::read("resources/points_intensity.csv")?;
		let potree = csv_reader::from_csv(&buffer)?;

		assert_eq!(potree.size, 10);

		Ok(())
    }

	
}
