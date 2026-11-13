use super::distribution::SurfaceInstance;
use crate::math::{Mat4, Quat, Vec3};
use crate::render::mesh::Mesh;
use crate::render::shader::FragmentShader;

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
    pub orbit: Option<OrbitParams>,
    /// Self-rotation angular speed in radians/second (about the local Y axis).
    pub spin_speed: f64,
    pub shader: Box<dyn FragmentShader>,
    /// Shader used for the surface props (usually a plain Lambert: the planet's own grid/bump
    /// shaders are tuned for the sphere's UVs and look like noise on small boxes/spheres).
    pub prop_shader: Box<dyn FragmentShader>,
    /// Fractional height of the tallest terrain feature above the base sphere (0 for perfect
    /// spheres, the deformation amplitude for noise-displaced planets). Collision and camera
    /// clearance use `radius * (1 + surface_bulge)` so the ship never sinks into a hill.
    pub surface_bulge: f32,

    /// The planet's up-to-4 exclusive surface model types (empty for the sun / un-themed bodies).
    pub prop_meshes: Vec<Mesh>,
    /// Where each surface-model instance sits, in local unit-sphere space (see
    /// `distribution::SurfaceInstance`). Regenerated in place by `regenerate_surface` when the
    /// placement seed changes.
    pub surface_instances: Vec<SurfaceInstance>,

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

    pub fn collision_radius(&self) -> f32 {
        self.radius * (1.0 + self.surface_bulge)
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::translation(self.position) * self.rotation.to_mat4() * Mat4::scaling(self.radius)
    }

    /// World-space model matrix for one surface instance. Composing onto `model_matrix()` (which
    /// already carries the planet's position/rotation/radius) is what makes the instance orbit,
    /// spin and sit at the right scale automatically, with no extra per-frame recomputation.
    pub fn instance_model_matrix_with(&self, body_model: &Mat4, inst: &SurfaceInstance) -> Mat4 {
        *body_model
            * Mat4::translation(inst.local_pos)
            * inst.local_rotation.to_mat4()
            * Mat4::scaling(inst.scale)
    }

    /// Combined rotation (planet spin + instance alignment/yaw), for correct normal transforms.
    pub fn instance_rotation(&self, inst: &SurfaceInstance) -> Quat {
        self.rotation * inst.local_rotation
    }

    /// LOD distance thresholds scale with the body's own radius, so a small and a huge placeholder
    /// both switch detail at a visually similar point. `mesh_lods` goes coarsest (index 0) to
    /// finest; this returns `mesh_lods.len() - 1` thresholds in descending distance order.
    pub fn lod_thresholds(&self) -> Vec<f32> {
        let n = self.mesh_lods.len();
        if n <= 1 {
            return Vec::new();
        }
        (1..n).map(|i| self.radius * (i as f32) * 16.0).collect()
    }
}
