use crate::model::vector3::Vector3;

use super::bounds::Bounds;

pub struct Node {
    spacing: f64,
    pub bounds: Bounds,
    pub children: [Option<Box<Node>>; 8],
    max_points_per_leaf_node: u32,
    pub grid: NodeGrid,
    pub initial_store: Vec<Vector3>,
    squared_spacing: f64,
    pub name: String,
    pub byte_size: u32,
    pub byte_offset: u32,
}

impl Node {
    pub fn new(
        name: String,
        spacing: f64,
        bounds: Bounds,
        children: [Option<Box<Node>>; 8],
        max_points_per_leaf_node: u32,
    ) -> Node {
        Node {
            spacing,
            bounds,
            children,
            max_points_per_leaf_node,
            grid: empty_grid_array(),
            initial_store: Vec::new(),
            squared_spacing: spacing * spacing,
            name,
            byte_size: 0,
            byte_offset: 0,
        }
    }

    pub fn level(&self) -> usize {
        return self.name.len() - 1;
    }

    pub fn is_leaf_node(&self) -> bool {
        self.children.iter().all(|child| child.is_none())
    }

    pub fn points(&self) -> Vec<&Vector3> {
        if self.is_leaf_node() {
            return self.initial_store.iter().collect();
        } else {
            let mut point_vectors: Vec<&Vector3> = Vec::new();
            for i in 0..8 {
                for j in 0..8 {
                    point_vectors.append(&mut self.grid[i][j].iter().collect());
                }
            }

            return point_vectors;
        }
    }

    pub fn num_points(&self) -> usize {
        if self.is_leaf_node() {
            self.initial_store.len()
        } else {
            let mut count = 0;
            for row in &self.grid {
                for cell in row {
                    count += cell.len();
                }
            }
            count
        }
    }

    pub fn add_point(&mut self, point: Vector3) {
        if self.is_leaf_node() {
            self.initial_store.push(point.clone());
            if self.initial_store.len() >= self.max_points_per_leaf_node as usize {
                let index = Node::find_grid_index(&point, &self.bounds);
                self.split(index.into())
            }
        } else {
            let grid_index_outer = Node::find_grid_index(&point, &self.bounds);
            let grid_index_inner =
                Node::find_grid_index(&point, &self.compute_child_bounds(grid_index_outer));
            if self.grid[grid_index_outer][grid_index_inner]
                .iter()
                .any(|p| Node::within_distance(&p, &point, self.squared_spacing))
            {
                match &mut self.children[grid_index_outer] {
                    None => {
                        let mut new_child = Box::new(self.new_child_node(grid_index_outer));
                        new_child.add_point(point);
                        self.children[grid_index_outer] = Some(new_child);
                    }
                    Some(ref mut child) => child.add_point(point),
                };
            } else {
                self.grid[grid_index_outer][grid_index_inner].push(point);
            }
        }
    }

    fn split(&mut self, index: usize) {
        self.children[index] = Some(Box::new(self.new_child_node(index)));
        // We should probably not clone here and deal with how to do this better
        for point in &self.initial_store.clone() {
            self.add_point(point.clone());
        }
        self.initial_store = Vec::new();
    }

    fn new_child_node(&self, index: usize) -> Node {
        Node::new(
            format!("{}{}", &self.name, index.to_string()),
            self.spacing / 2.0,
            self.compute_child_bounds(index),
            empty_child_node_array(),
            self.max_points_per_leaf_node,
        )
    }

    fn within_distance(a: &Vector3, b: &Vector3, squared_distance: f64) -> bool {
        let x_diff = (a.x - b.x) * (a.x - b.x);
        let y_diff = (a.y - b.y) * (a.y - b.y);
        let z_diff = (a.z - b.z) * (a.z - b.z);
        (x_diff + y_diff + z_diff) < squared_distance
    }
    fn find_grid_index(point: &Vector3, bounds: &Bounds) -> usize {
        let low_x = point.x < (bounds.lx + bounds.ux) / 2.0; //lower than mid x
        let low_y = point.y < (bounds.ly + bounds.uy) / 2.0; //lower than mid y
        let low_z = point.z < (bounds.lz + bounds.uz) / 2.0; //lower than mid z
        if low_x && low_y && low_z {
            0
        } else if low_x && low_y && !low_z {
            1
        } else if low_x && !low_y && low_z {
            2
        } else if low_x && !low_y && !low_z {
            3
        } else if !low_x && low_y && low_z {
            4
        } else if !low_x && low_y && !low_z {
            5
        } else if !low_x && !low_y && low_z {
            6
        } else {
            7
        }
    }

    fn compute_child_bounds(&self, index: usize) -> Bounds {
        let bounds = &self.bounds;
        let bounds_mid_x = (bounds.lx + bounds.ux) / 2.0;
        let bounds_mid_y = (bounds.ly + bounds.uy) / 2.0;
        let bounds_mid_z = (bounds.lz + bounds.uz) / 2.0;
        if index == 0 {
            Bounds::new(
                bounds_mid_x,
                bounds_mid_y,
                bounds_mid_z,
                bounds.lx,
                bounds.ly,
                bounds.lz,
            )
        } else if index == 1 {
            Bounds::new(
                bounds_mid_x,
                bounds_mid_y,
                bounds.uz,
                bounds.lx,
                bounds.ly,
                bounds_mid_z,
            )
        } else if index == 2 {
            Bounds::new(
                bounds_mid_x,
                bounds.uy,
                bounds_mid_z,
                bounds.lx,
                bounds_mid_y,
                bounds.lz,
            )
        } else if index == 3 {
            Bounds::new(
                bounds_mid_x,
                bounds.uy,
                bounds.uz,
                bounds.lx,
                bounds_mid_y,
                bounds_mid_z,
            )
        } else if index == 4 {
            Bounds::new(
                bounds.ux,
                bounds_mid_y,
                bounds_mid_z,
                bounds_mid_x,
                bounds.ly,
                bounds.lz,
            )
        } else if index == 5 {
            Bounds::new(
                bounds.ux,
                bounds_mid_y,
                bounds.uz,
                bounds_mid_x,
                bounds.ly,
                bounds_mid_z,
            )
        } else if index == 6 {
            Bounds::new(
                bounds.ux,
                bounds.uy,
                bounds_mid_z,
                bounds_mid_x,
                bounds_mid_y,
                bounds.lz,
            )
        } else {
            Bounds::new(
                bounds.ux,
                bounds.uy,
                bounds.uz,
                bounds_mid_x,
                bounds_mid_y,
                bounds_mid_z,
            )
        }
    }
}

pub type GridRow = [Vec<Vector3>; 8];
pub type NodeGrid = [GridRow; 8];

pub fn empty_child_node_array() -> [Option<Box<Node>>; 8] {
    [None, None, None, None, None, None, None, None]
}

pub fn empty_grid_array() -> NodeGrid {
    [
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
        empty_grid_row(),
    ]
}

pub fn empty_grid_row() -> GridRow {
    [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ]
}
