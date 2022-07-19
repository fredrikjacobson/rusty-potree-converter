use crate::model::bounds::{find_bounds, Bounds};
use crate::model::node::empty_child_node_array;
use crate::model::node::Node;
use crate::model::vector3::Vector3;

pub struct Potree {
    pub bounds: Bounds,
    cubic_bounds: Bounds,
    size_len: f64,

    point_per_leaf_node_limit: u32,

    pub spacing: f64,
    pub scale: f64,
    pub size: u32,

    pub root: Node,
}

const DIAGONAL_FRACTION: f64 = 200.0;

impl Potree {
    pub fn new(points: Vec<Vector3>, point_per_leaf_node_limit: u32) -> Potree {
        let bounds = find_bounds(&points);
        let cubic_bounds = bounds.cubic();
        let size_len = ((cubic_bounds.size_x * cubic_bounds.size_x)
            + (cubic_bounds.size_y * cubic_bounds.size_y)
            + (cubic_bounds.size_z * cubic_bounds.size_z))
            .sqrt();
        let spacing = size_len / DIAGONAL_FRACTION;

        let mut root_node = Node::new(
            "r".to_string(),
            spacing,
            bounds.clone(),
            empty_child_node_array(),
            point_per_leaf_node_limit,
        );

        let size = points.len() as u32;

        for point in points {
            root_node.add_point(point);
        }

        Potree {
            spacing,
            scale: if size_len > 1_000_000.0 {
                0.01
            } else if size_len > 1.0 {
                0.001
            } else {
                0.0001
            },
            size,
            bounds,
            cubic_bounds,
            size_len,
            point_per_leaf_node_limit,
            root: root_node,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::model::node::Node;
    use crate::model::node::NodeGrid;
    use crate::potree::Potree;
    use crate::potree::Vector3;
    use crate::writer::write_potree;
    use byteorder::LittleEndian;
    use byteorder::ReadBytesExt;
    use rand::prelude::*;
    use std::fs;
    use std::path::Path;

    fn setup_potree(point_count: u32, node_limit: u32) -> Potree {
        let mut rng = rand::thread_rng();

        let point_per_leaf_node_limit = node_limit;
        let mut points = Vec::new();
        for _i in 0..point_count {
            points.push(Vector3 {
                x: rng.gen_range(0.0..100.0),
                y: rng.gen_range(0.0..10.0),
                z: rng.gen_range(0.0..10.0),
            });
        }

        Potree::new(points, point_per_leaf_node_limit)
    }

    #[test]
    fn test_write_potree() {
        let count = 100;
        let potree = setup_potree(count, 1000);
        let dir = Path::new("/tmp/test-output");
        write_potree(potree, dir);
        let file_size = fs::metadata(dir.join("octree.bin")).unwrap().len();
        let points_written = file_size / (3 * 4);
        assert_eq!(count as u64, points_written);
    }

    #[test]
    fn test_write_binary_points() {
        let buffer = fs::read("resources/points.bin").unwrap();
        let length = buffer.len();
        let mut cursor = Cursor::new(buffer);
        let pos = 0;
        let mut points: Vec<Vector3> = Vec::new();

        while cursor.position() < (length - 1) as u64 {
            points.push(Vector3 {
                x: cursor.read_f64::<LittleEndian>().unwrap(),
                y: cursor.read_f64::<LittleEndian>().unwrap(),
                z: cursor.read_f64::<LittleEndian>().unwrap(),
            })
        }
        for p in 0..5 {
            let point = &points[p];
            println!("x: {}, y: {}, z: {}", point.x, point.y, point.z);
        }

        println!("{}", points.len());
        let expected_points = 495934;
        assert_eq!(points.len(), expected_points);
        let potree = Potree::new(points, 20000);

        let dir = Path::new("/tmp/test-output");
        write_potree(potree, dir);

        let file_size = fs::metadata(dir.join("octree.bin")).unwrap().len();
        let points_written = file_size / (3 * 4);
        assert_eq!(expected_points as u64, points_written);
    }

    #[allow(dead_code)]
    fn print_grid(grid: &NodeGrid) -> String {
        let mut rows = Vec::new();
        for row in 0..8 {
            rows.push(format!(
                "[{}, {}, {}, {}, {}, {}, {}, {}]",
                grid[row][0].len(),
                grid[row][1].len(),
                grid[row][2].len(),
                grid[row][3].len(),
                grid[row][4].len(),
                grid[row][5].len(),
                grid[row][6].len(),
                grid[row][7].len(),
            ));
        }

        rows.join("\n")
    }

    #[allow(dead_code)]
    fn print_node(node: &Node, level: u8) {
        println!(
            "{} Node with store {} is_leaf {} count {}",
            "   ".repeat(level as usize),
            node.initial_store.len(),
            node.is_leaf_node(),
            node.num_points()
        );
        println!("{}", print_grid(&node.grid));
        for child in &node.children {
            if let Some(child) = child {
                print_node(&child, level + 1);
            } else {
            }
        }
    }
}
