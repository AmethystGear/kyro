use crate::matrix_3d::Matrix3D;
use amethyst::renderer::rendy::mesh::{Normal, Position, TexCoord};
use lazy_static::lazy_static;
use ron::from_str;
use serde::Deserialize;
use std::fs;
use amethyst::core::math::{Vector2, Vector3};

lazy_static! {
    static ref TRIANGULATION: Triangulation = {
        return from_str(&fs::read_to_string("assets/triangulation.ron").unwrap()).unwrap();
    };

    static ref TRI_TABLE: Vec<Vec<u8>> = {
        return TRIANGULATION.triangulation_table.clone();
    };

    static ref CUBE_POINTS: Vec<Vector3<f32>> = {
        let mut cube_points = vec![];
        for cube_point in &TRIANGULATION.cube_points {
            cube_points.push(Vector3::new(cube_point.0 as f32, cube_point.1 as f32, cube_point.2 as f32));
        }
        return cube_points;
    };

    static ref CUBE_EDGES: Vec<Vector2<usize>> = {
        let mut cube_edges = vec![];
        for cube_edge in &TRIANGULATION.cube_edges {
            cube_edges.push(Vector2::new(cube_edge.0, cube_edge.1));
        }
        return cube_edges;
    };
}

#[derive(Deserialize)]
struct Triangulation {
    triangulation_table: Vec<Vec<u8>>,
    cube_points: Vec<(usize, usize, usize)>,
    cube_edges: Vec<(usize, usize)>
}

const CUTOFF: f32 = 0.0;

fn get_cube_tris(matrix: &Matrix3D, vector: Vector3<usize>) -> Vec<Vector3<f32>> {
    let mut tris = vec![];
    let mut id = 0;
    let mut vals = [0.0; 8];
    for i in 0..8 {
        let point = &CUBE_POINTS[i];
        let val = matrix.get(vector + Vector3::new(point.x as usize, point.y as usize, point.z as usize));
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
            let start = CUBE_POINTS[edge.x];
            let end = CUBE_POINTS[edge.y];
            let start_density = vals[edge.x];
            let end_density = vals[edge.y];
            let start_weight;
            let end_weight;
            if end_density < start_density {
                start_weight = (CUTOFF - end_density) / (start_density - end_density);
                end_weight = 1.0 - start_weight;
            } else {
                end_weight = (CUTOFF - start_density) / (end_density - start_density);
                start_weight = 1.0 - end_weight;
            }
            tris.push(start.scale(start_weight) + end.scale(end_weight));
        }
    }
    return tris;
}

fn correct(
    pts: Vec<Vector3<f32>>,
    scale: f32,
    displace: Vector3<usize>,
) -> Vec<Vector3<f32>> {
    let mut new = vec![];
    for pt in pts {
        new.push((pt + Vector3::new(displace.x as f32,displace.y as f32, displace.z as f32)).scale(scale));
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
                let vec3 = Vector3::new(x, y, z);
                let pts = correct(get_cube_tris(matrix, vec3), scale, vec3);
                for pt in &pts {
                    posns.push(Position {
                        0: [pt.x, pt.y, pt.z],
                    });
                }
                for i in 0..pts.len() / 3 {
                    let normal: Vector3<f32> = (&pts[i * 3 + 1] - &pts[i * 3]).cross(&(&pts[i * 3 + 2] - &pts[i * 3]));
                    for _ in 0..3 {
                        norms.push(Normal {
                            0: [normal.x, normal.y, normal.z],
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
