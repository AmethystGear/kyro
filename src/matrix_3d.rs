use amethyst::core::math::Vector3;

pub struct Matrix3D {
    dim: Vector3<usize>,
    elems: Vec<f32>,
}

impl Matrix3D {
    pub fn new(dim: Vector3<usize>) -> Self {
        Matrix3D {
            dim,
            elems: vec![0.0; dim.x * dim.y * dim.z],
        }
    }

    fn index(&self, vec: Vector3<usize>) -> usize {
        return vec.z * self.dim.x * self.dim.y + vec.y * self.dim.x + vec.x;
    }

    pub fn get(&self, vec: Vector3<usize>) -> f32 {
        return self.elems[self.index(vec)];
    }

    pub fn set(&mut self, vec: Vector3<usize>, val: f32) {
        let index = self.index(vec);
        self.elems[index] = val;
    }

    pub fn x(&self) -> usize {
        return self.dim.x;
    }

    pub fn y(&self) -> usize {
        return self.dim.y;
    }

    pub fn z(&self) -> usize {
        return self.dim.z;
    }
}
