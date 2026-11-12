use crate::math::{Quat, Vec3};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// One placed surface object, defined in the host planet's *local unit-sphere* space (before the
/// planet's own model matrix is applied). `local_pos` sits exactly on the unit sphere, so
/// multiplying by `body.model_matrix()` automatically scales it to the planet's radius, orbits it,
/// and spins it along with the planet — no extra per-frame work needed for correct placement.
#[derive(Clone, Copy)]
pub struct SurfaceInstance {
    pub local_pos: Vec3,
    /// Aligns the prop's local "up" to the surface normal at `local_pos`, plus a random yaw.
    pub local_rotation: Quat,
    /// Extra per-instance scale variation (prop meshes are already sized relative to a unit
    /// sphere; this just adds natural variety, not a base unit conversion).
    pub scale: f32,
    /// Index into the planet's `prop_meshes`.
    pub prop_type: usize,
}

/// How many instances to scatter for each of a planet's (up to 4) prop types.
pub struct ScatterSpec {
    pub counts: Vec<u32>,
    /// Minimum angular separation (radians) enforced between any two placed instances, so props
    /// don't overlap/clump.
    pub min_separation: f32,
    pub scale_range: (f32, f32),
    /// For a planet whose mesh is deformed by `render::deform::deform_by_height` (same function +
    /// amplitude here), offsets each instance's radius to match -- otherwise props would float
    /// above valleys or sink into hills on a non-spherical surface. `None` for ordinary
    /// perfectly-spherical planets.
    pub surface_height: Option<(fn(Vec3) -> f32, f32)>,
}

/// Rejection-sampling blue-noise scatter over the unit sphere: for each prop type, repeatedly
/// picks a uniformly random point on the sphere (Marsaglia method) and keeps it only if it's far
/// enough (angularly) from every instance already placed. Reproducible: the same `seed` always
/// yields the same positions, satisfying the "misma semilla -> mismas ubicaciones" requirement.
pub fn scatter(seed: u64, spec: &ScatterSpec) -> Vec<SurfaceInstance> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut placed: Vec<Vec3> = Vec::new();
    let mut instances = Vec::new();

    for (prop_type, &count) in spec.counts.iter().enumerate() {
        let mut placed_for_type = 0u32;
        let mut attempts = 0u32;
        let max_attempts = count * 60 + 60;

        while placed_for_type < count && attempts < max_attempts {
            attempts += 1;
            let p = random_point_on_sphere(&mut rng);

            let far_enough = placed.iter().all(|&q| p.dot(q).clamp(-1.0, 1.0).acos() >= spec.min_separation);
            if !far_enough {
                continue;
            }

            let yaw = Quat::from_axis_angle(p, rng.random_range(0.0f32..std::f32::consts::TAU));
            let align = Quat::from_to_rotation(Vec3::UP, p);
            let scale = rng.random_range(spec.scale_range.0..spec.scale_range.1);

            let local_pos = match spec.surface_height {
                Some((height_fn, amplitude)) => p * (1.0 + height_fn(p) * amplitude),
                None => p,
            };

            instances.push(SurfaceInstance { local_pos, local_rotation: yaw * align, scale, prop_type });
            placed.push(p);
            placed_for_type += 1;
        }
    }

    instances
}

fn random_point_on_sphere(rng: &mut StdRng) -> Vec3 {
    // Marsaglia (1972): reject points outside the unit disc, then project onto the sphere.
    loop {
        let x1 = rng.random_range(-1.0f32..1.0f32);
        let x2 = rng.random_range(-1.0f32..1.0f32);
        let d2 = x1 * x1 + x2 * x2;
        if d2 >= 1.0 {
            continue;
        }
        let s = (1.0 - d2).sqrt();
        return Vec3::new(2.0 * x1 * s, 2.0 * x2 * s, 1.0 - 2.0 * d2);
    }
}
