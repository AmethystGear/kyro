use crate::matrix_3d::Matrix3D;
use amethyst::core::math::{Vector2, Vector3};
use amethyst::renderer::rendy::mesh::{Normal, Position, TexCoord};
use arr_macro::arr;
use lazy_static::lazy_static;
use ron::from_str;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum TriangulationMethod {
    BASIC,
}

lazy_static! {
    static ref METHOD_FILES: HashMap<TriangulationMethod, String> = {
        let mut m = HashMap::new();
        m.insert(
            TriangulationMethod::BASIC,
            "assets/triangulation/methods/basic.ron".to_string(),
        );
        return m;
    };
    static ref TRIANGULATION: HashMap<TriangulationMethod, Triangulation> = {
        let mut m = HashMap::new();
        for (k, v) in &*METHOD_FILES {
            let triangulation: DeserTriangulation =
                from_str(&fs::read_to_string(v).unwrap()).unwrap();
            m.insert(k.clone(), triangulation.convert());
        }
        return m;
    };
    static ref CUBE_POINTS: Vec<Vector3<f32>> = {
        let mut cube_points = vec![];
        let pts: Vec<(usize, usize, usize)> =
            from_str(&fs::read_to_string("assets/triangulation/points.ron").unwrap()).unwrap();
        for cube_point in &pts {
            cube_points.push(Vector3::new(
                cube_point.0 as f32,
                cube_point.1 as f32,
                cube_point.2 as f32,
            ));
        }
        return cube_points;
    };
    static ref CUBE_EDGES: Vec<Vector2<usize>> = {
        let mut cube_edges = vec![];
        let edges: Vec<(usize, usize)> =
            from_str(&fs::read_to_string("assets/triangulation/edges.ron").unwrap()).unwrap();
        for cube_edge in &edges {
            cube_edges.push(Vector2::new(cube_edge.0, cube_edge.1));
        }
        return cube_edges;
    };
}

struct Triangulation {
    triangulation_table: [Vec<u8>; 256],
    connect_points: bool,
}

#[derive(Deserialize)]
struct DeserTriangulation {
    triangulation_table: HashMap<u8, Vec<u8>>,
    connect_points: bool,
}

impl DeserTriangulation {
    pub fn convert(self) -> Triangulation {
        let mut triangulation_table: [Vec<u8>; 256] = arr![Vec::new(); 256];
        for (k, v) in self.triangulation_table {
            triangulation_table[k as usize] = v;
        }
        return Triangulation {
            triangulation_table,
            connect_points: self.connect_points,
        };
    }
}

const CUTOFF: f32 = 0.0;

fn get_cube_tris(
    matrix: &Matrix3D,
    vector: Vector3<usize>,
    triangulation: &Triangulation,
    interpolated: bool,
) -> Vec<Vector3<f32>> {
    let mut tris = vec![];
    let mut id = 0;
    let mut vals = [0.0; 8];
    for i in 0..8 {
        let point = &CUBE_POINTS[i];
        let val =
            matrix.get(vector + Vector3::new(point.x as usize, point.y as usize, point.z as usize));
        vals[i] = val;
        if val < CUTOFF {
            id += 2usize.pow(i as u32);
        }
    }
    for i in 0..triangulation.triangulation_table[id].len() / 3 {
        let connect = [
            triangulation.triangulation_table[id][i * 3],
            triangulation.triangulation_table[id][i * 3 + 1],
            triangulation.triangulation_table[id][i * 3 + 2],
        ];
        for j in 0..3 {
            if triangulation.connect_points {
                tris.push(CUBE_POINTS[connect[j] as usize]);
            } else {
                let edge = CUBE_EDGES[connect[j] as usize];
                let start = CUBE_POINTS[edge.x];
                let end = CUBE_POINTS[edge.y];
                let start_density = vals[edge.x];
                let end_density = vals[edge.y];
                let start_weight;
                let end_weight;
                if interpolated {
                    if end_density < start_density {
                        start_weight = (CUTOFF - end_density) / (start_density - end_density);
                        end_weight = 1.0 - start_weight;
                    } else {
                        end_weight = (CUTOFF - start_density) / (end_density - start_density);
                        start_weight = 1.0 - end_weight;
                    }
                } else {
                    start_weight = 0.5;
                    end_weight = 0.5;
                }
                tris.push(start.scale(start_weight) + end.scale(end_weight));
            }
        }
    }
    return tris;
}

fn correct(pts: Vec<Vector3<f32>>, scale: f32, displace: Vector3<usize>) -> Vec<Vector3<f32>> {
    let mut new = vec![];
    for pt in pts {
        new.push(
            (pt + Vector3::new(displace.x as f32, displace.y as f32, displace.z as f32))
                .scale(scale),
        );
    }
    return new;
}

pub fn get_mesh_data(
    matrix: &Matrix3D,
    scale: f32,
    triangulation_method: TriangulationMethod,
    interpolated: bool,
) -> MeshData {
    let mut posns = vec![];
    let mut norms = vec![];
    let mut coords = vec![];
    let triangulation_method = TRIANGULATION.get(&triangulation_method).unwrap();
    for z in 0..(matrix.z() - 1) {
        for y in 0..(matrix.y() - 1) {
            for x in 0..(matrix.x() - 1) {
                let vector = Vector3::new(x, y, z);
                let pts = correct(
                    get_cube_tris(matrix, vector, triangulation_method, interpolated),
                    scale,
                    vector,
                );
                for pt in &pts {
                    posns.push(Position {
                        0: [pt.x, pt.y, pt.z],
                    });
                }
                for i in 0..pts.len() / 3 {
                    let normal: Vector3<f32> =
                        (&pts[i * 3 + 1] - &pts[i * 3]).cross(&(&pts[i * 3 + 2] - &pts[i * 3]));
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
