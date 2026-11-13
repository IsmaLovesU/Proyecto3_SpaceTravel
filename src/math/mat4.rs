use super::vec3::Vec3;
use super::vec4::Vec4;
use std::ops::Mul;

/// Row-major 4x4 matrix: m[row][col]. Points are transformed as `M * v` (v as a column vector).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn translation(t: Vec3) -> Mat4 {
        let mut m = Mat4::IDENTITY;
        m.m[0][3] = t.x;
        m.m[1][3] = t.y;
        m.m[2][3] = t.z;
        m
    }

    pub fn scaling(s: f32) -> Mat4 {
        Mat4::scaling_nonuniform(Vec3::new(s, s, s))
    }

    pub fn scaling_nonuniform(s: Vec3) -> Mat4 {
        let mut m = Mat4::IDENTITY;
        m.m[0][0] = s.x;
        m.m[1][1] = s.y;
        m.m[2][2] = s.z;
        m
    }

    /// Right-handed perspective projection. Maps view-space depth to NDC z in [0, 1]
    /// (near -> 0, far -> 1), matching the framebuffer's depth-buffer convention.
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fovy_radians * 0.5).tan();
        let mut m = [[0.0f32; 4]; 4];
        m[0][0] = f / aspect;
        m[1][1] = f;
        m[2][2] = far / (near - far);
        m[2][3] = (far * near) / (near - far);
        m[3][2] = -1.0;
        Mat4 { m }
    }

    /// Right-handed view matrix (camera looks down -Z in view space).
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = (target - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);
        Mat4 {
            m: [
                [s.x, s.y, s.z, -s.dot(eye)],
                [u.x, u.y, u.z, -u.dot(eye)],
                [-f.x, -f.y, -f.z, f.dot(eye)],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn mul_vec4(self, v: Vec4) -> Vec4 {
        let a = [v.x, v.y, v.z, v.w];
        let mut out = [0.0f32; 4];
        for i in 0..4 {
            out[i] = self.m[i][0] * a[0] + self.m[i][1] * a[1] + self.m[i][2] * a[2] + self.m[i][3] * a[3];
        }
        Vec4::new(out[0], out[1], out[2], out[3])
    }

    /// Transforms a point (implicit w = 1).
    pub fn mul_point(self, p: Vec3) -> Vec3 {
        self.mul_vec4(Vec4::from_point(p)).xyz()
    }

}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut out = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[i][k] * rhs.m[k][j];
                }
                out[i][j] = sum;
            }
        }
        Mat4 { m: out }
    }
}
