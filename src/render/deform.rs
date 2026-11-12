use super::mesh::{recompute_smooth_normals, Mesh};
use crate::math::{fbm, Vec3};

/// Displaces every vertex of `mesh` along its own radial direction (for a unit-sphere mesh, the
/// position *is* the outward normal) by `height_fn(original_position) * amplitude`, then
/// recomputes normals from the deformed surface. This is a distinct technique from the icosphere
/// subdivision used for LOD: it reshapes an *existing* sphere rather than generating new
/// geometry.
pub fn deform_by_height(mesh: &mut Mesh, height_fn: impl Fn(Vec3) -> f32, amplitude: f32) {
    for p in &mut mesh.positions {
        let dir = p.normalized();
        let h = height_fn(dir);
        *p = dir * (1.0 + h * amplitude);
    }
    mesh.normals = recompute_smooth_normals(&mesh.positions, &mesh.indices);
}

/// Mictlan Celestial's terrain height field: fixed noise parameters for this one planet, reused
/// both to deform its mesh (see `deform_by_height`) and to keep surface props
/// (`scene::distribution::scatter`) sitting on that same terrain instead of floating/sinking.
pub fn mictlan_terrain_height(p: Vec3) -> f32 {
    fbm(p * 2.4, 4, 7.0)
}
