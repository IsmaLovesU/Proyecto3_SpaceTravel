use crate::math::{Mat4, Vec3};

pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Plane {
    fn normalize(self) -> Plane {
        let len = self.normal.length();
        if len <= f32::EPSILON {
            self
        } else {
            Plane { normal: self.normal * (1.0 / len), d: self.d / len }
        }
    }

    #[inline]
    fn signed_distance(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.d
    }
}

/// Extracts the 6 frustum planes (left, right, bottom, top, near, far) from a combined
/// view-projection matrix (Gribb/Hartmann method), with inward-facing normals.
pub fn extract_planes(view_proj: &Mat4) -> [Plane; 6] {
    let m = view_proj.m;
    let row = |i: usize| Vec3::new(m[i][0], m[i][1], m[i][2]);
    let w = row(3);
    let wd = m[3][3];

    let build = |r: Vec3, rd: f32, sign: f32| -> Plane {
        Plane { normal: Vec3::new(w.x + sign * r.x, w.y + sign * r.y, w.z + sign * r.z), d: wd + sign * rd }
            .normalize()
    };

    // Near is `row2` alone (not row2+row3): our projection maps near -> NDC z = 0, so the
    // inside condition is simply clip.z >= 0, unlike the +-w conditions used for the other sides.
    let near = Plane { normal: row(2), d: m[2][3] }.normalize();

    [
        build(row(0), m[0][3], 1.0),  // left
        build(row(0), m[0][3], -1.0), // right
        build(row(1), m[1][3], 1.0),  // bottom
        build(row(1), m[1][3], -1.0), // top
        near,
        build(row(2), m[2][3], -1.0), // far
    ]
}

/// Sphere-vs-frustum test used to cull whole celestial bodies before transforming/rasterizing
/// their meshes.
pub fn sphere_in_frustum(planes: &[Plane; 6], center: Vec3, radius: f32) -> bool {
    planes.iter().all(|p| p.signed_distance(center) >= -radius)
}
