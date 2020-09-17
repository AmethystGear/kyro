use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn generate_perlin_noise(width: u16, height: u16, octave_count: u8, seed: i64) -> Vec<f64> {
    let mut white_noise = vec![0f64; (width as usize) * (height as usize)];
    let mut total_noise: Vec<Vec<f64>> = Vec::with_capacity(octave_count as usize);
    let mut p_noise = vec![0f64; (width as usize) * (height as usize)];

    let mut amplitude = 1.0f64;
    let mut total_amplitude = 0.0f64;
    let persistance = 0.5f64;

    let bytes = seed.to_le_bytes();
    let mut seed_bytes = [0; 32];
    for i in 0..8 {
        seed_bytes[i] = bytes[i];
    }

    let mut rng: StdRng = SeedableRng::from_seed(seed_bytes);
    for i in 0..((width as usize) * (height as usize)) {
        white_noise[i] = rng.gen::<f64>();
    }
    for i in 0..octave_count {
        let v = perlin_noise(width, height, i, &white_noise);
        total_noise.push(v);
    }
    for i in (0..octave_count as usize).rev() {
        amplitude *= persistance;
        total_amplitude += amplitude;
        for j in 0..((width as usize) * (height as usize)) {
            p_noise[j] = p_noise[j] + total_noise[i][j] * amplitude;
        }
    }
    for i in 0..((width as usize) * (height as usize)) {
        p_noise[i] /= total_amplitude;
    }
    return p_noise;
}

fn perlin_noise(width: u16, height: u16, octave: u8, white_noise: &Vec<f64>) -> Vec<f64> {
    let mut result = vec![0f64; (width as usize) * (height as usize)];
    let sample_period: usize = 1 << (octave as usize);
    let sample_frequency = 1.0f64 / sample_period as f64;

    for j in 0..(height as usize) {
        let y1: usize = (j / sample_period) * sample_period;
        let y2: usize = (y1 + sample_period) % height as usize;
        let y_blend = (j - y1) as f64 * sample_frequency;
        for i in 0..(width as usize) {
            let x1: usize = (i / sample_period) * sample_period;
            let x2: usize = (x1 + sample_period) % width as usize;
            let x_blend = (i - x1) as f64 * sample_frequency;

            let top = lerp(
                get(white_noise, width as usize, x1, y1),
                get(white_noise, width as usize, x2, y1),
                x_blend,
            );
            let bottom = lerp(
                get(white_noise, width as usize, x1, y2),
                get(white_noise, width as usize, x2, y2),
                x_blend,
            );
            set(
                &mut result,
                width as usize,
                i,
                j,
                lerp(top, bottom, y_blend),
            );
        }
    }
    return result;
}

fn get(a: &Vec<f64>, w: usize, x: usize, y: usize) -> f64 {
    return a[y * w + x];
}

fn set(a: &mut Vec<f64>, w: usize, x: usize, y: usize, val: f64) {
    a[y * w + x] = val;
}

fn lerp(a: f64, b: f64, blend: f64) -> f64 {
    return a * (1.0f64 - blend) + b * blend;
}
