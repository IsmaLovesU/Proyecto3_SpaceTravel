use super::vec3::Vec3;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn from_point(p: Vec3) -> Self {
        Vec4::new(p.x, p.y, p.z, 1.0)
    }

    pub fn from_dir(d: Vec3) -> Self {
        Vec4::new(d.x, d.y, d.z, 0.0)
    }

    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Perspective divide: returns NDC position (xyz / w).
    pub fn perspective_divide(self) -> Vec3 {
        let inv_w = 1.0 / self.w;
        Vec3::new(self.x * inv_w, self.y * inv_w, self.z * inv_w)
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, s: f32) -> Vec4 {
        Vec4::new(self.x * s, self.y * s, self.z * s, self.w * s)
    }
}
