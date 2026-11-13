use crate::math::{Vec2, Vec3};
use raylib::color::Color;
use std::collections::HashMap;

/// Plain triangle mesh: positions/normals/uvs/colors are parallel arrays, `indices` are triangle
/// triplets. Per-vertex color lets a single mesh carry multiple flat colors (e.g. a mushroom prop
/// with a red cap and a white stem) without extra draw calls.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub colors: Vec<Color>,
    pub indices: Vec<u32>,
}

/// Generates a unit icosphere (radius 1, centered at origin) with `subdivisions` recursive
/// subdivision passes. Normals equal the normalized position (perfect sphere). UVs use a simple
/// equirectangular (lon/lat) mapping.
pub fn icosphere(subdivisions: u32) -> Mesh {
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut positions: Vec<Vec3> = vec![
        Vec3::new(-1.0, t, 0.0),
        Vec3::new(1.0, t, 0.0),
        Vec3::new(-1.0, -t, 0.0),
        Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t),
        Vec3::new(0.0, 1.0, t),
        Vec3::new(0.0, -1.0, -t),
        Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0),
        Vec3::new(t, 0.0, 1.0),
        Vec3::new(-t, 0.0, -1.0),
        Vec3::new(-t, 0.0, 1.0),
    ]
    .into_iter()
    .map(|p| p.normalized())
    .collect();

    let mut indices: Vec<u32> = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11,
        1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7, 1, 8,
        3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9,
        4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
    ];

    for _ in 0..subdivisions {
        let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();
        let mut new_indices = Vec::with_capacity(indices.len() * 4);

        let mut midpoint = |a: u32, b: u32, positions: &mut Vec<Vec3>| -> u32 {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&idx) = midpoint_cache.get(&key) {
                return idx;
            }
            let mid = ((positions[a as usize] + positions[b as usize]) * 0.5).normalized();
            positions.push(mid);
            let idx = (positions.len() - 1) as u32;
            midpoint_cache.insert(key, idx);
            idx
        };

        for tri in indices.chunks_exact(3) {
            let (a, b, c) = (tri[0], tri[1], tri[2]);
            let ab = midpoint(a, b, &mut positions);
            let bc = midpoint(b, c, &mut positions);
            let ca = midpoint(c, a, &mut positions);
            new_indices.extend_from_slice(&[a, ab, ca, b, bc, ab, c, ca, bc, ab, bc, ca]);
        }

        indices = new_indices;
    }

    let normals: Vec<Vec3> = positions.clone();
    let uvs: Vec<Vec2> = positions
        .iter()
        .map(|p| {
            let u = 0.5 + p.z.atan2(p.x) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - p.y.asin() / std::f32::consts::PI;
            Vec2::new(u, v)
        })
        .collect();

    Mesh { positions, normals, uvs, colors: Vec::new(), indices }
}

/// Precomputes a set of icospheres at increasing subdivision levels, from coarsest (index 0) to
/// finest (index `levels - 1`), used for distance-based level of detail.
pub fn icosphere_lod_levels(levels: u32) -> Vec<Mesh> {
    (0..levels).map(icosphere).collect()
}

/// Recomputes smooth per-vertex normals from scratch: accumulates each triangle's face normal
/// into its three vertices, then normalizes. Shared by noise deformation (after displacing
/// vertices) and mesh subdivision (after Laplacian smoothing) — both need normals recomputed from
/// new positions, independent of whatever normals the mesh had before.
pub fn recompute_smooth_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let face_normal = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }
    for n in &mut normals {
        *n = n.normalized();
    }
    normals
}
