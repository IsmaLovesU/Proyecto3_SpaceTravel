use super::mesh::Mesh;
use crate::math::{Quat, Vec2, Vec3};
use raylib::color::Color;

/// Axis-aligned box centered at the origin, flat-shaded (each face has its own 4 vertices so
/// normals are per-face, not averaged).
pub fn box_mesh(half_x: f32, half_y: f32, half_z: f32) -> Mesh {
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)), // +X
        (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, -1.0)), // -X
        (Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)), // +Y
        (Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(1.0, 0.0, 0.0)), // -Y
        (Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)), // +Z
        (Vec3::new(0.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)), // -Z
    ];
    let half = Vec3::new(half_x, half_y, half_z);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for (normal, u_axis, v_axis) in faces {
        let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
        let base = positions.len() as u32;
        for (u, v) in corners {
            let unit = normal + u_axis * u + v_axis * v;
            let p = Vec3::new(unit.x * half.x, unit.y * half.y, unit.z * half.z);
            positions.push(p);
            normals.push(normal);
            uvs.push(Vec2::new((u + 1.0) * 0.5, (v + 1.0) * 0.5));
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh { positions, normals, uvs, colors: Vec::new(), indices }
}

/// Cylinder with its base circle centered at the origin (y = 0), extending up to y = height, with
/// flat top/bottom caps and smooth (radially normaled) sides.
pub fn cylinder_mesh(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let segments = segments.max(3);

    // Sides: two rings of vertices (bottom, top), smooth radial normals.
    let side_base = positions.len() as u32;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = t * std::f32::consts::TAU;
        let (x, z) = (angle.cos() * radius, angle.sin() * radius);
        let n = Vec3::new(angle.cos(), 0.0, angle.sin());
        positions.push(Vec3::new(x, 0.0, z));
        normals.push(n);
        uvs.push(Vec2::new(t, 0.0));
        positions.push(Vec3::new(x, height, z));
        normals.push(n);
        uvs.push(Vec2::new(t, 1.0));
    }
    for i in 0..segments {
        let b0 = side_base + i * 2;
        let t0 = b0 + 1;
        let b1 = side_base + (i + 1) * 2;
        let t1 = b1 + 1;
        indices.extend_from_slice(&[b0, b1, t0, t0, b1, t1]);
    }

    append_cap(&mut positions, &mut normals, &mut uvs, &mut indices, radius, 0.0, segments, false);
    append_cap(&mut positions, &mut normals, &mut uvs, &mut indices, radius, height, segments, true);

    Mesh { positions, normals, uvs, colors: Vec::new(), indices }
}

/// Cone with its base circle centered at the origin (y = 0) and apex at y = height.
pub fn cone_mesh(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let segments = segments.max(3);

    let apex = Vec3::new(0.0, height, 0.0);
    let slope = (radius * radius + height * height).sqrt().max(1e-5);
    let base = positions.len() as u32;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = t * std::f32::consts::TAU;
        let (x, z) = (angle.cos() * radius, angle.sin() * radius);
        // Side normal: blend of radial direction and the cone's upward slope.
        let n = Vec3::new(angle.cos() * height / slope, radius / slope, angle.sin() * height / slope).normalized();
        positions.push(Vec3::new(x, 0.0, z));
        normals.push(n);
        uvs.push(Vec2::new(t, 0.0));
        positions.push(apex);
        normals.push(n);
        uvs.push(Vec2::new(t, 1.0));
    }
    for i in 0..segments {
        let b0 = base + i * 2;
        let a0 = b0 + 1;
        let b1 = base + (i + 1) * 2;
        indices.extend_from_slice(&[b0, b1, a0]);
    }

    append_cap(&mut positions, &mut normals, &mut uvs, &mut indices, radius, 0.0, segments, false);

    Mesh { positions, normals, uvs, colors: Vec::new(), indices }
}

#[allow(clippy::too_many_arguments)]
fn append_cap(
    positions: &mut Vec<Vec3>,
    normals: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    indices: &mut Vec<u32>,
    radius: f32,
    y: f32,
    segments: u32,
    facing_up: bool,
) {
    let normal = if facing_up { Vec3::UP } else { -Vec3::UP };
    let center_idx = positions.len() as u32;
    positions.push(Vec3::new(0.0, y, 0.0));
    normals.push(normal);
    uvs.push(Vec2::new(0.5, 0.5));

    let ring_base = positions.len() as u32;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = t * std::f32::consts::TAU;
        positions.push(Vec3::new(angle.cos() * radius, y, angle.sin() * radius));
        normals.push(normal);
        uvs.push(Vec2::new(0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5));
    }
    for i in 0..segments {
        let a = ring_base + i;
        let b = ring_base + i + 1;
        if facing_up {
            indices.extend_from_slice(&[center_idx, a, b]);
        } else {
            indices.extend_from_slice(&[center_idx, b, a]);
        }
    }
}

/// Appends `src` into `dst`, transformed by a uniform `scale` + `rotation` + `translate` (applied
/// in that order), tinted with a single flat `color`. Used to build a composite prop mesh (e.g. a
/// mushroom = a cone/sphere cap + a cylinder stem) out of several primitive pieces in one `Mesh`.
pub fn append_transformed(dst: &mut Mesh, src: &Mesh, translate: Vec3, rotation: Quat, scale: f32, color: Color) {
    append_transformed_fn(dst, src, translate, rotation, scale, |_| color);
}

/// Same as `append_transformed`, but the color is computed per source vertex (in the primitive's
/// own *local, pre-transform* space) — e.g. banding a sphere by local Y to get a Poké Ball's
/// red/white/black stripes out of a single icosphere.
pub fn append_transformed_fn(
    dst: &mut Mesh,
    src: &Mesh,
    translate: Vec3,
    rotation: Quat,
    scale: f32,
    color_fn: impl Fn(Vec3) -> Color,
) {
    let base = dst.positions.len() as u32;
    for p in &src.positions {
        dst.positions.push(rotation.rotate_vec3(*p * scale) + translate);
        dst.colors.push(color_fn(*p));
    }
    for n in &src.normals {
        dst.normals.push(rotation.rotate_vec3(*n).normalized());
    }
    dst.uvs.extend_from_slice(&src.uvs);
    dst.indices.extend(src.indices.iter().map(|i| i + base));
}
