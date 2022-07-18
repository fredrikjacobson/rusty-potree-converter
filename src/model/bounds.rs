use crate::model::vector3::Vector3;
use ord_subset::OrdSubsetIterExt;

#[derive(Clone)]
pub struct Bounds {
    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,
    pub ux: f64,
    pub uy: f64,
    pub uz: f64,
    pub lx: f64,
    pub ly: f64,
    pub lz: f64,
}

impl Bounds {
    pub fn new(ux: f64, uy: f64, uz: f64, lx: f64, ly: f64, lz: f64) -> Bounds {
        Bounds {
            size_x: (lx - ux).abs(),
            size_y: (ly - uy).abs(),
            size_z: (lz - uz).abs(),
            ux,
            uy,
            uz,
            lx,
            ly,
            lz,
        }
    }

    pub fn cubic(&self) -> Bounds {
        let max_size = [self.size_x, self.size_y, self.size_z]
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let new_ux = self.lx + max_size;
        let new_uy = self.ly + max_size;
        let new_uz = self.lz + max_size;
        Bounds::new(new_ux, new_uy, new_uz, self.lx, self.ly, self.lz)
    }
}

#[allow(dead_code)]
pub fn find_bounds(points: &Vec<Vector3>) -> Bounds {
    let xs: Vec<f64> = points.iter().map(|p| p.x).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.y).collect();
    let zs: Vec<f64> = points.iter().map(|p| p.z).collect();

    Bounds::new(
        *xs.iter().ord_subset_max().unwrap(),
        *ys.iter().ord_subset_max().unwrap(),
        *zs.iter().ord_subset_max().unwrap(),
        *xs.iter().ord_subset_min().unwrap(),
        *ys.iter().ord_subset_min().unwrap(),
        *zs.iter().ord_subset_min().unwrap(),
    )
}
