use pcd_rs::{anyhow, DynReader, DynRecord};

use crate::{model::vector3::Vector3, potree::Potree};

pub fn from_pcd(buf: &[u8]) -> Result<Potree, Box<dyn std::error::Error>> {
    let reader = DynReader::from_bytes(buf)?;
    let pcd: Result<Vec<DynRecord>, anyhow::Error> = reader.collect();

    let mut points: Vec<Vector3> = Vec::new();

    let mut num_points = 0;
    for point in pcd?.iter() {
        let [x, y, z] = point.to_owned().xyz::<f64>().unwrap();
        points.push(Vector3 { x, y, z });

        num_points = num_points + 1;
    }

    let potree = Potree::new(points, 20000);

    Ok(potree)
}
