use crate::math::{Mat4, Quat, Vec3};
use crate::render::mesh::Mesh;

pub struct Ship {
    pub position: Vec3,
    pub orientation: Quat,
    pub velocity: Vec3,
    pub mesh: Mesh,
    pub mesh_scale: f32,
    /// Approximate bounding-sphere radius (world units, after `mesh_scale`) used for collisions.
    pub collision_radius: f32,
}

impl Ship {
    pub fn forward(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::FORWARD)
    }

    pub fn right(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::RIGHT)
    }

    pub fn up(&self) -> Vec3 {
        self.orientation.rotate_vec3(Vec3::UP)
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::translation(self.position) * self.orientation.to_mat4() * Mat4::scaling(self.mesh_scale)
    }
}

/// Small hand-built wedge used only if `assets/models/ship.obj` is missing, so the project still
/// runs (and is flyable) before the final modeled ship is dropped in.
pub fn placeholder_wedge_mesh() -> Mesh {
    let positions = vec![
        Vec3::new(0.0, 0.0, -1.0),  // nose
        Vec3::new(-0.6, -0.2, 0.8), // rear-left-bottom
        Vec3::new(0.6, -0.2, 0.8),  // rear-right-bottom
        Vec3::new(0.0, 0.5, 0.8),   // rear-top
    ];
    let faces = [[0u32, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]];
    let mut out_positions = Vec::new();
    let mut out_normals = Vec::new();
    let mut out_uvs = Vec::new();
    let mut indices = Vec::new();
    for face in faces {
        let (a, b, c) = (positions[face[0] as usize], positions[face[1] as usize], positions[face[2] as usize]);
        let n = (b - a).cross(c - a).normalized();
        for p in [a, b, c] {
            out_positions.push(p);
            out_normals.push(n);
            out_uvs.push(crate::math::Vec2::ZERO);
            indices.push((out_positions.len() - 1) as u32);
        }
    }
    Mesh { positions: out_positions, normals: out_normals, uvs: out_uvs, indices }
}
