use super::vec3::Vec3;

/// Deterministic hash -> pseudo-random value in [0, 1), based on a position.
fn hash(p: Vec3) -> f32 {
    let h = p.dot(Vec3::new(127.1, 311.7, 74.7));
    let s = h.sin() * 43758.5453;
    s - s.floor()
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// 3D value noise: trilinear interpolation of a hashed value at each surrounding lattice point,
/// eased with smoothstep. Returns a value in [0, 1].
fn value_noise(p: Vec3) -> f32 {
    let i = Vec3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = p - i;
    let u = Vec3::new(smoothstep(f.x), smoothstep(f.y), smoothstep(f.z));

    let corner = |dx: f32, dy: f32, dz: f32| hash(i + Vec3::new(dx, dy, dz));

    let x00 = corner(0.0, 0.0, 0.0) + (corner(1.0, 0.0, 0.0) - corner(0.0, 0.0, 0.0)) * u.x;
    let x10 = corner(0.0, 1.0, 0.0) + (corner(1.0, 1.0, 0.0) - corner(0.0, 1.0, 0.0)) * u.x;
    let x01 = corner(0.0, 0.0, 1.0) + (corner(1.0, 0.0, 1.0) - corner(0.0, 0.0, 1.0)) * u.x;
    let x11 = corner(0.0, 1.0, 1.0) + (corner(1.0, 1.0, 1.0) - corner(0.0, 1.0, 1.0)) * u.x;

    let y0 = x00 + (x10 - x00) * u.y;
    let y1 = x01 + (x11 - x01) * u.y;

    y0 + (y1 - y0) * u.z
}

/// Fractal Brownian motion: several octaves of value noise summed at increasing frequency and
/// decreasing amplitude, normalized to roughly [-1, 1]. Used both for deforming a planet's
/// geometry and for generating a procedural normal map, with different frequency/amplitude.
pub fn fbm(p: Vec3, octaves: u32, seed: f32) -> f32 {
    let shifted = p + Vec3::splat(seed * 17.0);
    let mut total = 0.0;
    let mut amplitude = 0.5;
    let mut max_amplitude = 0.0;
    let mut freq = 1.0;
    for _ in 0..octaves {
        total += (value_noise(shifted * freq) * 2.0 - 1.0) * amplitude;
        max_amplitude += amplitude;
        amplitude *= 0.5;
        freq *= 2.0;
    }
    if max_amplitude > 0.0 {
        total / max_amplitude
    } else {
        0.0
    }
}
