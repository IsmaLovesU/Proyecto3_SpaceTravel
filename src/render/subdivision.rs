use super::mesh::{recompute_smooth_normals, Mesh};
use crate::math::{Vec2, Vec3};
use raylib::color::Color;
use std::collections::HashMap;

/// Generic procedural geometry subdivision: works on *any* input mesh (unlike `icosphere()`,
/// which only ever builds a sphere from scratch). Each pass splits every triangle into 4 at its
/// edge midpoints, then nudges vertices toward the average of their new neighbors (a Laplacian
/// smoothing pass) to round off sharp low-poly edges. Used to smooth the loaded ship mesh before
/// rasterizing it -- this is the "etapa propia de generación o subdivisión procedural de
/// geometría" requirement, distinct from the noise-based planet deformation in `deform.rs`.
pub fn subdivide_mesh(mesh: &Mesh, levels: u32, smoothing: f32) -> Mesh {
    let mut current = mesh.clone();
    for _ in 0..levels {
        current = subdivide_once(&current, smoothing);
    }
    current
}

fn subdivide_once(mesh: &Mesh, smoothing: f32) -> Mesh {
    let mut positions = mesh.positions.clone();
    let mut uvs = mesh.uvs.clone();
    let mut colors = mesh.colors.clone();
    let mut indices = Vec::with_capacity(mesh.indices.len() * 4);
    let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();

    let mut midpoint = |a: u32, b: u32, positions: &mut Vec<Vec3>, uvs: &mut Vec<Vec2>, colors: &mut Vec<Color>| -> u32 {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = midpoint_cache.get(&key) {
            return idx;
        }
        let pos = (positions[a as usize] + positions[b as usize]) * 0.5;
        let uv = (uvs[a as usize] + uvs[b as usize]) * 0.5;
        let color = lerp_color(colors[a as usize], colors[b as usize], 0.5);
        positions.push(pos);
        uvs.push(uv);
        colors.push(color);
        let idx = (positions.len() - 1) as u32;
        midpoint_cache.insert(key, idx);
        idx
    };

    for tri in mesh.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let ab = midpoint(a, b, &mut positions, &mut uvs, &mut colors);
        let bc = midpoint(b, c, &mut positions, &mut uvs, &mut colors);
        let ca = midpoint(c, a, &mut positions, &mut uvs, &mut colors);
        indices.extend_from_slice(&[a, ab, ca, b, bc, ab, c, ca, bc, ab, bc, ca]);
    }

    if smoothing > 0.0 {
        let mut neighbor_sum = vec![Vec3::ZERO; positions.len()];
        let mut neighbor_count = vec![0u32; positions.len()];
        for tri in indices.chunks_exact(3) {
            for &(i, j) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                neighbor_sum[i as usize] += positions[j as usize];
                neighbor_count[i as usize] += 1;
                neighbor_sum[j as usize] += positions[i as usize];
                neighbor_count[j as usize] += 1;
            }
        }
        for i in 0..positions.len() {
            if neighbor_count[i] > 0 {
                let avg = neighbor_sum[i] / neighbor_count[i] as f32;
                positions[i] = positions[i].lerp(avg, smoothing);
            }
        }
    }

    let normals = recompute_smooth_normals(&positions, &indices);
    Mesh { positions, normals, uvs, colors, indices }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color { r: lerp(a.r, b.r), g: lerp(a.g, b.g), b: lerp(a.b, b.b), a: lerp(a.a, b.a) }
}
