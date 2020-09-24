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

    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        return z * self.x * self.y + y * self.x + x;
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> f32 {
        return self.elems[self.index(x, y, z)];
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: f32) {
        let index = self.index(x, y, z);
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
