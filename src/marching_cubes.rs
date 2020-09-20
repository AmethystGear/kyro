
use ron::from_str;
use std::fs;
use crate::matrix_3d::Matrix3D;
use lazy_static::lazy_static;
use serde::Deserialize;
use amethyst::renderer::rendy::mesh::{Normal, Position, TexCoord};

lazy_static! {
    static ref TRI_TABLE: Vec<Vec<u8>> = {
        let triangulation: Triangulation = from_str(&fs::read_to_string("assets/triangulation.ron").unwrap()).unwrap();
        return triangulation.triangulation_table;
    };
}

#[derive(Deserialize)]
struct Triangulation {
    triangulation_table : Vec<Vec<u8>>
}

const cube_points: [(usize, usize, usize); 8] = [
    (0, 0, 0),
    (1, 0, 0),
    (1, 0, 1),
    (0, 0, 1),
    (0, 1, 0),
    (1, 1, 0),
    (1, 1, 1),
    (0, 1, 1)
];

const cube_edges: [(usize, usize); 12] = [
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
    (3, 7)
];

const cutoff : f32 = 0.5;

fn get_cube_tris (matrix: &Matrix3D, x : usize, y : usize, z : usize) -> Vec<(f32, f32, f32)> {
    let mut tris = vec![];
    let mut id = 0;
    for i in 0..8 {
        if matrix.get(x + cube_points[i].0, y + cube_points[i].1, z + cube_points[i].2) < cutoff {
            id += 2usize.pow(i as u32);
        }
    }
    for i in 0..TRI_TABLE[id].len()/3 {
        let edges = [
            TRI_TABLE[id][i],
            TRI_TABLE[id][i + 1],
            TRI_TABLE[id][i + 2]
        ];
        for j in 0..3 {
            let edge = cube_edges[edges[j] as usize];
            let start = cube_points[edge.0];
            let end = cube_points[edge.1];
            let x = (start.0 as f32 + end.0 as f32) / 2.0;
            let y = (start.1 as f32 + end.1 as f32) / 2.0;
            let z = (start.2 as f32 + end.2 as f32) / 2.0;
            tris.push((x, y, z));
        }
    }
    return tris;
}

fn correct(pts : Vec<(f32, f32, f32)>, scale: f32, displace : (usize, usize, usize)) -> Vec<(f32, f32, f32)> {
    let mut new = vec![];
    for pt in pts {
        new.push((
            pt.0 * scale + displace.0 as f32 * scale,
            pt.1 * scale + displace.1 as f32 * scale,
            pt.2 * scale + displace.2 as f32 * scale
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
                        0: [pt.0, pt.1, pt.2]
                    });
                }
                for i in 0..pts.len()/3 {
                    let normal = cross(
                        sub(pts[i + 1], pts[i]),
                        sub(pts[i + 2], pts[i])
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
    }
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