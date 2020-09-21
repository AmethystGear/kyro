use crate::{marching_cubes, matrix_3d::Matrix3D};
use marching_cubes::MeshData;
use noise::{NoiseFn, OpenSimplex, Perlin, Point2, Point3, Seedable};
use rand::{prelude::StdRng, Rng, SeedableRng};
use splines::{Interpolation, Key, Spline};

pub struct Terrain {
    noise: Box<dyn NoiseFn<Point3<f64>>>,
    upper_bound: Spline<f32, f32>,
    lower_bound: Spline<f32, f32>,
    points_per_chunk: u8,
    scale: f32,
    sample_scale: f32,
}

impl Terrain {
    pub fn new(
        seed: u128,
        points_per_chunk: u8,
        scale: f32,
        sample_scale: f32,
    ) -> Self {
        let bytes: [u8; 16] = seed.to_be_bytes();
        let mut seed: [u8; 32] = [0; 32];
        for i in 0..32 {
            seed[i] = bytes[i % 16];
        }
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let noise = Box::new(OpenSimplex::new().set_seed(rng.gen()));
        let heightmap = Box::new(OpenSimplex::new().set_seed(rng.gen()));
        
        let floor = -40.0;
        let cave = -20.0;
        let surface = 0.0;
        let air = 10.0;
        
        let upper_bound: Spline<f32, f32> = Spline::from_vec(vec![
            Key::new(floor, 1.0, Interpolation::Linear),
            Key::new(cave, 0.2, Interpolation::Linear),
            Key::new(surface, 1.0, Interpolation::Linear),
            Key::new(air, -1.0, Interpolation::Linear)
        ]);

        let lower_bound: Spline<f32, f32> = Spline::from_vec(vec![
            Key::new(floor, 1.0, Interpolation::Linear),
            Key::new(cave, -1.0, Interpolation::Linear),
            Key::new(surface, -0.2, Interpolation::Linear),
            Key::new(air, -1.0, Interpolation::Linear)
        ]);

        Terrain {
            noise,
            upper_bound,
            lower_bound,
            points_per_chunk,
            scale,
            sample_scale
        }
    }

    fn get_matrix(&self, chunk_x: i16, chunk_y: i16, chunk_z: i16) -> Matrix3D {
        let mut matrix = Matrix3D::new(
            self.points_per_chunk as usize + 1,
            self.points_per_chunk as usize + 1,
            self.points_per_chunk as usize + 1,
        );
        let true_chunk_x = (chunk_x as isize * self.points_per_chunk as isize) as f32 * self.scale;
        let true_chunk_y = (chunk_y as isize * self.points_per_chunk as isize) as f32 * self.scale;
        let true_chunk_z = (chunk_z as isize * self.points_per_chunk as isize) as f32 * self.scale;
        for z in 0..(self.points_per_chunk + 1) {
            for y in 0..(self.points_per_chunk + 1) {
                for x in 0..(self.points_per_chunk + 1) {
                    let true_x = true_chunk_x + x as f32 * self.scale;
                    let true_y = true_chunk_y + y as f32 * self.scale;
                    let true_z = true_chunk_z + z as f32 * self.scale;

                    let val = self.noise.get([
                        (true_x * self.sample_scale) as f64,
                        (true_y * self.sample_scale) as f64,
                        (true_z * self.sample_scale) as f64,
                    ]) as f32;
                    let upper_bound = self.upper_bound.clamped_sample(true_y).unwrap();
                    let lower_bound = self.lower_bound.clamped_sample(true_y).unwrap();
                    let diff = upper_bound - lower_bound;
                    let adjusted_val = (val - (-1.0)) * diff + lower_bound;

                    matrix.set(x as usize, y as usize, z as usize, adjusted_val);
                }
            }
        }
        return matrix;
    }

    pub fn get_chunk(&self, chunk_x: i16, chunk_y: i16, chunk_z: i16) -> MeshData {
        return marching_cubes::get_mesh_data(
            &self.get_matrix(chunk_x, chunk_y, chunk_z),
            self.scale,
        );
    }

    pub fn chunk_size(&self) -> f32 {
        return self.scale * self.points_per_chunk as f32;
    }
}
