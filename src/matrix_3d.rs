use amethyst::core::math::Vector3;

pub struct Matrix3D {
    x: usize,
    y: usize,
    z: usize,
    elems: Vec<f32>,
}

impl Matrix3D {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Matrix3D {
            x,
            y,
            z,
            elems: vec![0.0; x * y * z],
        }
    }

    fn index(&self, vec: Vector3<usize>/*x: usize, y: usize, z: usize*/) -> usize {
        return vec.z * self.x * self.y + vec.y * self.x + vec.x;
    }

    pub fn get(&self, vec: Vector3<usize> /*x: usize, y: usize, z: usize*/) -> f32 {
        return self.elems[self.index(vec)];
    }

    pub fn set(&mut self, vec: Vector3<usize>,/*x: usize, y: usize, z: usize,*/ val: f32) {
        let index = self.index(vec);
        self.elems[index] = val;
    }

    pub fn x(&self) -> usize {
        return self.x;
    }

    pub fn y(&self) -> usize {
        return self.y;
    }

    pub fn z(&self) -> usize {
        return self.z;
    }
}
