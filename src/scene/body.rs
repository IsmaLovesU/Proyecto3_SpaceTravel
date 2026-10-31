use crate::math::{Mat4, Quat, Vec3};
use crate::render::mesh::Mesh;
use crate::render::shader::FragmentShader;
use raylib::color::Color;

/// None for the sun (fixed at the origin).
pub struct OrbitParams {
    pub radius: f64,
    /// Angular speed in radians/second.
    pub speed: f64,
    pub phase: f64,
}

/// A sun/planet placeholder: a procedurally generated sphere with a distinct size, color, orbit
/// and spin. Orbit/rotation angles depend on accumulated simulation time, not frame count.
pub struct CelestialBody {
    pub name: String,
    pub mesh_lods: Vec<Mesh>,
    pub radius: f32,
    /// Used only for the gravity model; independent from the visual radius so it can be tuned.
    pub mass: f32,
    pub color: Color,
    pub orbit: Option<OrbitParams>,
    /// Self-rotation angular speed in radians/second (about the local Y axis).
    pub spin_speed: f64,
    pub shader: Box<dyn FragmentShader>,

    pub position: Vec3,
    pub rotation: Quat,
}

impl CelestialBody {
    pub fn update(&mut self, sim_time: f64) {
        if let Some(orbit) = &self.orbit {
            let angle = orbit.phase + orbit.speed * sim_time;
            self.position = Vec3::new((angle.cos() * orbit.radius) as f32, 0.0, (angle.sin() * orbit.radius) as f32);
        }
        let spin_angle = (self.spin_speed * sim_time) as f32;
        self.rotation = Quat::from_axis_angle(Vec3::UP, spin_angle);
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::translation(self.position) * self.rotation.to_mat4() * Mat4::scaling(self.radius)
    }

    /// LOD distance thresholds scale with the body's own radius, so a small and a huge placeholder
    /// both switch detail at a visually similar point. `mesh_lods` goes coarsest (index 0) to
    /// finest; this returns `mesh_lods.len() - 1` thresholds in descending distance order.
    pub fn lod_thresholds(&self) -> Vec<f32> {
        let n = self.mesh_lods.len();
        if n <= 1 {
            return Vec::new();
        }
        (1..n).map(|i| self.radius * (i as f32) * 15.0).collect()
    }
}
