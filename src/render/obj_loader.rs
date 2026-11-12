use super::mesh::Mesh;
use crate::math::{Vec2, Vec3};
use raylib::color::Color;
use std::collections::HashMap;
use std::fs;
use std::io;

/// A single `f` face-corner reference: 1-based (or negative/relative) indices into the file's
/// v/vt/vn pools. uv/normal are optional, matching the `v`, `v/vt`, `v//vn`, `v/vt/vn` forms.
#[derive(Clone, Copy)]
struct Corner {
    pos: i32,
    uv: Option<i32>,
    norm: Option<i32>,
}

/// Minimal Wavefront .obj parser: reads positions/normals/uvs/faces ourselves (no raylib model
/// loading, per the course's rules). Supports triangles and n-gons (fan-triangulated), and
/// computes flat face normals when the file has none.
pub fn load_obj(path: &str) -> io::Result<Mesh> {
    let text = fs::read_to_string(path)?;

    let mut raw_pos: Vec<Vec3> = Vec::new();
    let mut raw_uv: Vec<Vec2> = Vec::new();
    let mut raw_norm: Vec<Vec3> = Vec::new();
    let mut faces: Vec<Vec<Corner>> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        let tag = match it.next() {
            Some(t) => t,
            None => continue,
        };
        match tag {
            "v" => {
                let x: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                raw_pos.push(Vec3::new(x, y, z));
            }
            "vt" => {
                let u: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let v: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                raw_uv.push(Vec2::new(u, v));
            }
            "vn" => {
                let x: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                raw_norm.push(Vec3::new(x, y, z));
            }
            "f" => {
                let corners: Vec<Corner> = it.map(parse_corner).collect();
                if corners.len() >= 3 {
                    faces.push(corners);
                }
            }
            _ => {}
        }
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut cache: HashMap<(usize, i64, i64), u32> = HashMap::new();

    for face in &faces {
        let face_normal = if raw_norm.is_empty() {
            let p0 = raw_pos[resolve_index(face[0].pos, raw_pos.len())];
            let p1 = raw_pos[resolve_index(face[1].pos, raw_pos.len())];
            let p2 = raw_pos[resolve_index(face[2].pos, raw_pos.len())];
            Some((p1 - p0).cross(p2 - p0).normalized())
        } else {
            None
        };

        let mut resolved = Vec::with_capacity(face.len());
        for corner in face {
            resolved.push(emit_corner(
                *corner,
                &raw_pos,
                &raw_uv,
                &raw_norm,
                face_normal,
                &mut cache,
                &mut positions,
                &mut uvs,
                &mut normals,
            ));
        }

        for i in 1..resolved.len() - 1 {
            indices.extend_from_slice(&[resolved[0], resolved[i], resolved[i + 1]]);
        }
    }

    let vertex_count = positions.len();
    Ok(Mesh { positions, normals, uvs, colors: vec![Color::new(205, 205, 215, 255); vertex_count], indices })
}

fn parse_corner(token: &str) -> Corner {
    let mut parts = token.split('/');
    let pos = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let uv = parts.next().and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
    let norm = parts.next().and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
    Corner { pos, uv, norm }
}

/// Resolves a 1-based (or negative, relative-to-end) obj index into a 0-based array index.
fn resolve_index(idx: i32, len: usize) -> usize {
    if idx > 0 {
        (idx - 1) as usize
    } else {
        (len as i32 + idx) as usize
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_corner(
    corner: Corner,
    raw_pos: &[Vec3],
    raw_uv: &[Vec2],
    raw_norm: &[Vec3],
    face_normal: Option<Vec3>,
    cache: &mut HashMap<(usize, i64, i64), u32>,
    positions: &mut Vec<Vec3>,
    uvs: &mut Vec<Vec2>,
    normals: &mut Vec<Vec3>,
) -> u32 {
    let pos_idx = resolve_index(corner.pos, raw_pos.len());
    let uv_idx = corner.uv.map(|u| resolve_index(u, raw_uv.len()));
    let norm_idx = corner.norm.map(|n| resolve_index(n, raw_norm.len()));

    let key = (pos_idx, uv_idx.map(|u| u as i64).unwrap_or(-1), norm_idx.map(|n| n as i64).unwrap_or(-1));
    if let Some(&existing) = cache.get(&key) {
        return existing;
    }

    positions.push(raw_pos[pos_idx]);
    uvs.push(uv_idx.map(|u| raw_uv[u]).unwrap_or(Vec2::ZERO));
    normals.push(norm_idx.map(|n| raw_norm[n]).unwrap_or_else(|| face_normal.unwrap_or(Vec3::UP)));

    let idx = (positions.len() - 1) as u32;
    cache.insert(key, idx);
    idx
}
