use crate::math::{fbm, Vec2, Vec3};

/// A procedural tangent-space normal map: a small lattice of perturbed normals (roughly pointing
/// "up", i.e. +Z in tangent space, with small x/y tilt) derived from a noise height field. No
/// image files involved — generated once in code, consistent with the rest of the project.
pub struct NormalMap {
    width: u32,
    height: u32,
    texels: Vec<Vec3>,
}

impl NormalMap {
    pub fn generate(width: u32, height: u32, frequency: f32, strength: f32, seed: f32) -> Self {
        let sample_height = |x: i32, y: i32| -> f32 {
            let xw = x.rem_euclid(width as i32) as f32 / width as f32;
            let yw = y.rem_euclid(height as i32) as f32 / height as f32;
            fbm(Vec3::new(xw * frequency, yw * frequency, 0.0), 4, seed)
        };

        let mut texels = Vec::with_capacity((width * height) as usize);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                // Central differences (wrapping in x/y) give the local surface slope, which we
                // turn into a tilted tangent-space normal -- the same idea as a GPU bump map, but
                // sampled from our own noise function instead of an image file.
                let dx = (sample_height(x + 1, y) - sample_height(x - 1, y)) * strength;
                let dy = (sample_height(x, y + 1) - sample_height(x, y - 1)) * strength;
                texels.push(Vec3::new(-dx, -dy, 1.0).normalized());
            }
        }

        NormalMap { width, height, texels }
    }

    pub fn sample(&self, uv: Vec2) -> Vec3 {
        let x = (uv.x.rem_euclid(1.0) * self.width as f32) as u32 % self.width;
        let y = (uv.y.rem_euclid(1.0) * self.height as f32) as u32 % self.height;
        self.texels[(y * self.width + x) as usize]
    }
}
