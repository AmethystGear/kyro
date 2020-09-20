pub struct Terrain;

/*
use amethyst::renderer::rendy::mesh::{Normal, Position, TexCoord};
use noise::{Billow, NoiseFn, Perlin, Seedable};
use rand::{prelude::StdRng, Rng, SeedableRng};

pub struct Terrain {
    regions: Vec<Box<dyn Region>>,
    divisions: u8,
    chunk_size: f32,
}

trait Region {
    fn height(&mut self, x: f32, y: f32, z: f32) -> f32;
}

trait Feature {
    fn height(&mut self, x: f32, y: f32, z: f32) -> (f32, Option<(i)>) {

    }
}

struct Hills {
    noise: Vec<Perlin>,
    scale: f32,
    height: f32,
    persistence: f32,
    lacunarity: f32,
}

impl Hills {
    fn new(
        octaves: u8,
        scale: f32,
        height: f32,
        persistence: f32,
        lacunarity: f32,
        rng: &mut StdRng,
    ) -> Self {
        let mut noise = vec![];
        for _ in 0..octaves {
            let perlin = Perlin::new();
            perlin.set_seed(rng.gen());
            noise.push(perlin);
        }
        return Hills {
            noise,
            scale,
            height,
            persistence,
            lacunarity,
        };
    }
}

impl Region for Hills {
    fn height(&mut self, x: f32, _: f32, z: f32) -> f32 {
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut noise_height = 0.0;

        for noise in &self.noise {
            let sample_x = x / self.scale * frequency;
            let sample_z = z / self.scale * frequency;
            let perlin_value = noise.get([(sample_x) as f64, (sample_z) as f64]) as f32;
            noise_height += perlin_value * amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        return noise_height * self.height;
    }
}

struct Bumps {
    noise: Billow,
    scale: f32,
    factor: f32,
}

impl Bumps {
    fn new(rng: &mut StdRng, scale: f32, factor: f32) -> Self {
        let noise = Billow::new();
        let noise = noise.set_seed(rng.gen());
        return Bumps {
            noise,
            scale,
            factor,
        };
    }
}

impl Region for Bumps {
    fn height(&mut self, x: f32, _: f32, z: f32) -> f32 {
        return self
            .noise
            .get([(x / self.scale) as f64, (z / self.scale) as f64]) as f32
            * self.factor;
    }
}

struct Canyon {
    noise: Perlin,
    size: f32,
}

impl Canyon {
    fn new(rng : &mut StdRng, size : f32) -> Self {
        let noise = Perlin::new();
        let noise = noise.set_seed(rng.gen());
        return Canyon {
            noise,
            size
        };
    }
}

impl Region for Canyon {
    fn height(&mut self, x: f32, y: f32, z: f32) -> f32 {
        
    }
}

impl Terrain {
    pub fn new(seed: u128, divisions: u8, chunk_size: f32) -> Self {
        let bytes: [u8; 16] = seed.to_be_bytes();
        let mut seed: [u8; 32] = [0; 32];
        for i in 0..32 {
            seed[i] = bytes[i % 16];
        }
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        let regions: Vec<Box<dyn Region>> = vec![
            Box::new(Hills::new(5, chunk_size * 0.25, 15.0, 0.1, 3.0, &mut rng)),
            Box::new(Hills::new(5, chunk_size, 70.0, 0.5, 1.5, &mut rng)),
            Box::new(Bumps::new(&mut rng, 200.0, 10.0)),
        ];
        Terrain {
            regions,
            divisions,
            chunk_size,
        }
    }

    fn get_pt(&mut self, start_x: f32, start_z: f32, x: u8, z: u8) -> (f32, f32, f32) {
        let true_x = (x as f32) / ((self.divisions - 1) as f32) * self.chunk_size + start_x;
        let true_z = (z as f32) / ((self.divisions - 1) as f32) * self.chunk_size + start_z;

        let mut total = 0.0;
        for region in &mut self.regions {
            let height = region.as_mut().height(true_x, total, true_z);
            total += height;
        }
        println!("{}", total);
        return (true_x - start_x, total, true_z - start_z);
    }

    pub fn chunk_size(&self) -> f32 {
        return self.chunk_size;
    }

    pub fn get_chunk(&mut self, chunk_x: i16, chunk_z: i16) -> TerrainMeshData {
        let start_x: f32 = (chunk_x as f32) * self.chunk_size;
        let start_z: f32 = (chunk_z as f32) * self.chunk_size;

        let tris = [[0, 3, 1], [0, 2, 3]];
        let mut data = TerrainMeshData {
            posns: vec![],
            norms: vec![],
            coords: vec![],
        };
        let mut v = vec![];
        for x in 0..self.divisions {
            v.push(self.get_pt(start_x, start_z, x, 0));
        }

        for z in 0..(self.divisions - 1) {
            let mut curr = self.get_pt(start_x, start_z, 0, z + 1);
            let mut next_v = vec![curr];
            for x in 0..(self.divisions - 1) {
                let next = self.get_pt(start_x, start_z, x + 1, z + 1);
                let pts = [v[x as usize], v[x as usize + 1], curr, next];
                curr = next;
                next_v.push(curr);

                for i in 0..tris.len() {
                    let normal = cross(
                        sub(pts[tris[i][1]], pts[tris[i][0]]),
                        sub(pts[tris[i][2]], pts[tris[i][0]]),
                    );
                    for j in 0..tris[i].len() {
                        data.norms.push(Normal {
                            0: [normal.0, normal.1, normal.2],
                        });
                        data.coords.push(TexCoord { 0: [0.0, 0.0] });
                        data.posns.push(Position {
                            0: [pts[tris[i][j]].0, pts[tris[i][j]].1, pts[tris[i][j]].2],
                        });
                    }
                }
            }
            v = next_v;
        }
        return data;
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

pub struct TerrainMeshData {
    posns: Vec<Position>,
    norms: Vec<Normal>,
    coords: Vec<TexCoord>,
}

impl TerrainMeshData {
    pub fn get_mesh_data(self) -> (Vec<u16>, Vec<Position>, Vec<Normal>, Vec<TexCoord>) {
        return (
            (0..(self.posns.len() as u16)).collect(),
            self.posns,
            self.norms,
            self.coords,
        );
    }
}
*/