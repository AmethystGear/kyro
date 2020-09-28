use crate::{marching_cubes, matrix_3d::Matrix3D};
use amethyst::core::math::Vector3;
use marching_cubes::{MeshData, TriangulationMethod};
use noise::{NoiseFn, OpenSimplex, Point3, Seedable};
use rand::{prelude::StdRng, Rng, SeedableRng};
use splines::{Interpolation, Key, Spline};

pub struct Terrain {
    noise: Vec<Box<dyn NoiseFn<Point3<f64>>>>,
    noise_weights: Vec<f32>,
    noise_scales: Vec<f32>,
    upper_bound: Spline<f32, f32>,
    lower_bound: Spline<f32, f32>,
    points_per_chunk: u8,
    scale: f32,
}

impl Terrain {
    pub fn new(
        seed: u128,
        points_per_chunk: u8,
        scale: f32,
        noise_weights: Vec<f32>,
        noise_scales: Vec<f32>,
    ) -> Self {
        let bytes: [u8; 16] = seed.to_be_bytes();
        let mut seed: [u8; 32] = [0; 32];
        for i in 0..32 {
            seed[i] = bytes[i % 16];
        }
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let mut noise: Vec<Box<dyn NoiseFn<Point3<f64>>>> = vec![];
        for _ in 0..noise_weights.len() {
            noise.push(Box::new(OpenSimplex::new().set_seed(rng.gen())));
        }

        let floor = -140.0;
        let cave = -5.0;
        let surface = 0.0;
        let hills = 20.0;
        let air = 50.0;

        let upper_bound: Spline<f32, f32> = Spline::from_vec(vec![
            Key::new(floor, -1.0, Interpolation::Bezier(0.0)),
            Key::new(cave, 0.5, Interpolation::Bezier(0.0)),
            Key::new(surface, 0.35, Interpolation::Bezier(0.0)),
            Key::new(hills, 0.8, Interpolation::Bezier(0.0)),
            Key::new(air, 1.0, Interpolation::Bezier(0.0)),
        ]);

        let lower_bound: Spline<f32, f32> = Spline::from_vec(vec![
            Key::new(floor, -1.0, Interpolation::Bezier(0.0)),
            Key::new(cave, -0.5, Interpolation::Bezier(0.0)),
            Key::new(surface, -0.65, Interpolation::Bezier(0.0)),
            Key::new(hills, -0.2, Interpolation::Bezier(0.0)),
            Key::new(air, 1.0, Interpolation::Bezier(0.0)),
        ]);

        Terrain {
            noise,
            noise_weights,
            noise_scales,
            upper_bound,
            lower_bound,
            points_per_chunk,
            scale,
        }
    }

    fn true_chunk(&self, chunk: Vector3<isize>) -> Vector3<f32> {
        return Vector3::new(chunk.x as f32, chunk.y as f32, chunk.z as f32)
            .scale(self.points_per_chunk as f32 * self.scale);
    }

    fn true_coord(&self, true_chunk: &Vector3<f32>, posn: &Vector3<usize>) -> Vector3<f32> {
        return true_chunk
            + Vector3::new(posn.x as f32, posn.y as f32, posn.z as f32).scale(self.scale);
    }

    fn get_matrix(&self, chunk: Vector3<isize>) -> Matrix3D {
        let points = self.points_per_chunk as usize + 1;
        let mut matrix = Matrix3D::new(Vector3::new(points, points, points));

        let true_chunk = self.true_chunk(chunk);
        for z in 0..points {
            for y in 0..points {
                for x in 0..points {
                    let true_coord: Vector3<f32> =
                        self.true_coord(&true_chunk, &Vector3::new(x, y, z));
                    let mut val = 0.0;
                    for i in 0..self.noise.len() {
                        val += self.noise[i].get([
                            (true_coord.x * self.noise_scales[i]) as f64,
                            (true_coord.y * self.noise_scales[i]) as f64,
                            (true_coord.z * self.noise_scales[i]) as f64,
                        ]) as f32
                            * self.noise_weights[i];
                    }

                    let upper_bound = self.upper_bound.clamped_sample(true_coord.y).unwrap();
                    let lower_bound = self.lower_bound.clamped_sample(true_coord.y).unwrap();
                    let diff = upper_bound - lower_bound;
                    let adjusted_val = (val - (-1.0)) * 0.5 * diff + lower_bound;
                    matrix.set(Vector3::new(x, y, z), adjusted_val);
                }
            }
        }
        return matrix;
    }

    pub fn get_chunk(&self, chunk: Vector3<isize>) -> MeshData {
        return marching_cubes::get_mesh_data(
            &self.get_matrix(chunk),
            self.scale,
            TriangulationMethod::BASIC,
            false,
        );
    }

    pub fn chunk_size(&self) -> f32 {
        return self.scale * self.points_per_chunk as f32;
    }
}
