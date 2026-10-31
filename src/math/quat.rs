use super::mat4::Mat4;
use super::vec3::Vec3;
use std::ops::Mul;

/// Unit quaternion used for ship/planet orientation (avoids gimbal lock).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn from_axis_angle(axis: Vec3, angle_radians: f32) -> Quat {
        let a = axis.normalized();
        let half = angle_radians * 0.5;
        let s = half.sin();
        Quat { x: a.x * s, y: a.y * s, z: a.z * s, w: half.cos() }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn normalized(self) -> Quat {
        let len = self.length_squared().sqrt();
        if len <= f32::EPSILON {
            Quat::IDENTITY
        } else {
            let inv = 1.0 / len;
            Quat { x: self.x * inv, y: self.y * inv, z: self.z * inv, w: self.w * inv }
        }
    }

    pub fn conjugate(self) -> Quat {
        Quat { x: -self.x, y: -self.y, z: -self.z, w: self.w }
    }

    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let qv = Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + t * self.w + qv.cross(t)
    }

    pub fn to_mat4(self) -> Mat4 {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        Mat4 {
            m: [
                [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy), 0.0],
                [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx), 0.0],
                [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy), 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn slerp(self, rhs: Quat, t: f32) -> Quat {
        let mut b = rhs;
        let mut dot = self.x * b.x + self.y * b.y + self.z * b.z + self.w * b.w;
        if dot < 0.0 {
            b = Quat { x: -b.x, y: -b.y, z: -b.z, w: -b.w };
            dot = -dot;
        }
        if dot > 0.9995 {
            let r = Quat {
                x: self.x + (b.x - self.x) * t,
                y: self.y + (b.y - self.y) * t,
                z: self.z + (b.z - self.z) * t,
                w: self.w + (b.w - self.w) * t,
            };
            return r.normalized();
        }
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        let s0 = (theta_0 - theta).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        Quat {
            x: self.x * s0 + b.x * s1,
            y: self.y * s0 + b.y * s1,
            z: self.z * s0 + b.z * s1,
            w: self.w * s0 + b.w * s1,
        }
    }
}

impl Mul for Quat {
    type Output = Quat;
    /// Hamilton product: applying `self * rhs` rotates by `rhs` first, then `self`.
    fn mul(self, rhs: Quat) -> Quat {
        Quat {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}
