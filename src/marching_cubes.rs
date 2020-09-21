use crate::matrix_3d::Matrix3D;
use amethyst::renderer::rendy::mesh::{Normal, Position, TexCoord};
use lazy_static::lazy_static;
use ron::from_str;
use serde::Deserialize;
use std::fs;

lazy_static! {
    static ref TRI_TABLE: Vec<Vec<u8>> = {
        let triangulation: Triangulation =
            from_str(&fs::read_to_string("assets/triangulation.ron").unwrap()).unwrap();
        return triangulation.triangulation_table;
    };
}

#[derive(Deserialize)]
struct Triangulation {
    triangulation_table: Vec<Vec<u8>>,
}

const CUBE_POINTS: [(usize, usize, usize); 8] = [
    (0, 0, 0),
    (1, 0, 0),
    (1, 0, 1),
    (0, 0, 1),
    (0, 1, 0),
    (1, 1, 0),
    (1, 1, 1),
    (0, 1, 1),
];

const CUBE_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

const CUTOFF: f32 = 0.0;

fn get_cube_tris(matrix: &Matrix3D, x: usize, y: usize, z: usize) -> Vec<(f32, f32, f32)> {
    let mut tris = vec![];
    let mut id = 0;
    let mut vals = [0.0; 8];
    for i in 0..8 {
        let val = matrix.get(
            x + CUBE_POINTS[i].0,
            y + CUBE_POINTS[i].1,
            z + CUBE_POINTS[i].2,
        );
        vals[i] = val;
        if val < CUTOFF {
            id += 2usize.pow(i as u32);
        }
    }
    for i in 0..TRI_TABLE[id].len() / 3 {
        let edges = [
            TRI_TABLE[id][i * 3],
            TRI_TABLE[id][i * 3 + 1],
            TRI_TABLE[id][i * 3 + 2],
        ];
        for j in 0..3 {
            let edge = CUBE_EDGES[edges[j] as usize];
            let start = CUBE_POINTS[edge.0];
            let end = CUBE_POINTS[edge.1];
            let start_density = vals[edge.0];
            let end_density = vals[edge.1];
            let start_weight;
            let end_weight;
            if end_density < start_density {
                start_weight = (CUTOFF - end_density) / (start_density - end_density);
                end_weight = 1.0 - start_weight;
            } else {
                end_weight = (CUTOFF - start_density) / (end_density - start_density);
                start_weight = 1.0 - end_weight;
            }
            let x = start.0 as f32 * start_weight + end.0 as f32 * end_weight;
            let y = start.1 as f32 * start_weight + end.1 as f32 * end_weight;
            let z = start.2 as f32 * start_weight + end.2 as f32 * end_weight;
            tris.push((x, y, z));
        }
    }
    return tris;
}

fn correct(
    pts: Vec<(f32, f32, f32)>,
    scale: f32,
    displace: (usize, usize, usize),
) -> Vec<(f32, f32, f32)> {
    let mut new = vec![];
    for pt in pts {
        new.push((
            pt.0 * scale + displace.0 as f32 * scale,
            pt.1 * scale + displace.1 as f32 * scale,
            pt.2 * scale + displace.2 as f32 * scale,
        ))
    }
    return new;
}

pub fn get_mesh_data(matrix: &Matrix3D, scale: f32) -> MeshData {
    let mut posns = vec![];
    let mut norms = vec![];
    let mut coords = vec![];
    for z in 0..(matrix.z() - 1) {
        for y in 0..(matrix.y() - 1) {
            for x in 0..(matrix.x() - 1) {
                let pts = correct(get_cube_tris(matrix, x, y, z), scale, (x, y, z));
                    
                for pt in &pts {
                    posns.push(Position {
                        0: [pt.0, pt.1, pt.2],
                    });
                }
                for i in 0..pts.len() / 3 {
                    let normal = cross(
                        sub(pts[i * 3 + 1], pts[i * 3]),
                        sub(pts[i * 3 + 2], pts[i * 3 + 1]),
                    );
                    for _ in 0..3 {
                        norms.push(Normal {
                            0: [normal.0, normal.1, normal.2],
                        });
                        coords.push(TexCoord { 0: [0.0, 0.0] });
                    }
                }
            }
        }
    }
    return MeshData {
        posns,
        norms,
        coords,
    };
}

fn sub(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32, f32) {
    return (a.0 - b.0, a.1 - b.1, a.2 - b.2);
}

fn cross(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32, f32) {
    return (
        a.1 * b.2 - a.2 * b.1,
        a.2 * b.0 - a.0 * b.2,
        a.0 * b.1 - a.1 * b.0,
    );
}

pub struct MeshData {
    posns: Vec<Position>,
    norms: Vec<Normal>,
    coords: Vec<TexCoord>,
}

impl MeshData {
    pub fn get_mesh_data(self) -> (Vec<u16>, Vec<Position>, Vec<Normal>, Vec<TexCoord>) {
        return (
            (0..(self.posns.len() as u16)).collect(),
            self.posns,
            self.norms,
            self.coords,
        );
    }
}
