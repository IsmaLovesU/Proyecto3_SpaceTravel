use super::framebuffer::Framebuffer;
use super::mesh::Mesh;
use super::shader::{FragmentShader, ShadingContext};
use crate::math::{Mat4, Quat, Vec2, Vec3, Vec4};
use crate::perf::stats::FrameStats;
use raylib::color::Color;

/// One vertex after the vertex stage: clip-space position plus every attribute the fragment
/// stage needs. Kept as a plain copyable struct so near-plane clipping can interpolate whole
/// vertices with a single `lerp`.
#[derive(Clone, Copy)]
struct ClipVertex {
    clip: Vec4,
    world: Vec3,
    normal: Vec3,
    uv: Vec2,
    color: Vec4,
}

impl ClipVertex {
    #[inline]
    fn lerp(a: &ClipVertex, b: &ClipVertex, t: f32) -> ClipVertex {
        ClipVertex {
            clip: Vec4::new(
                a.clip.x + (b.clip.x - a.clip.x) * t,
                a.clip.y + (b.clip.y - a.clip.y) * t,
                a.clip.z + (b.clip.z - a.clip.z) * t,
                a.clip.w + (b.clip.w - a.clip.w) * t,
            ),
            world: a.world.lerp(b.world, t),
            normal: a.normal.lerp(b.normal, t),
            uv: Vec2::new(a.uv.x + (b.uv.x - a.uv.x) * t, a.uv.y + (b.uv.y - a.uv.y) * t),
            color: Vec4::new(
                a.color.x + (b.color.x - a.color.x) * t,
                a.color.y + (b.color.y - a.color.y) * t,
                a.color.z + (b.color.z - a.color.z) * t,
                a.color.w + (b.color.w - a.color.w) * t,
            ),
        }
    }
}

#[inline]
fn color_to_vec4(c: Color) -> Vec4 {
    Vec4::new(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
}

/// Clips a triangle against the near plane (our projection maps the near plane to clip.z = 0,
/// so "inside" is simply `clip.z >= 0`). Returns 0, 3 or 4 vertices (a quad when exactly one
/// vertex is behind the plane), which the caller fan-triangulates. Without this, a triangle with
/// a vertex behind the camera would get a bogus perspective divide and smear a huge garbage
/// triangle across the screen whenever the ship flies close to a planet.
fn clip_near(input: &[ClipVertex; 3], out: &mut Vec<ClipVertex>) {
    out.clear();
    for i in 0..3 {
        let a = &input[i];
        let b = &input[(i + 1) % 3];
        let a_in = a.clip.z >= 0.0;
        let b_in = b.clip.z >= 0.0;
        if a_in {
            out.push(*a);
        }
        if a_in != b_in {
            let t = a.clip.z / (a.clip.z - b.clip.z);
            out.push(ClipVertex::lerp(a, b, t));
        }
    }
}

/// Transforms `mesh` by `model`/`rotation` and rasterizes it into `fb` with a per-pixel depth
/// test and a pluggable fragment shader. `rotation` is passed separately from `model` so normals
/// can be rotated without needing a general inverse-transpose (every object only uses uniform
/// scale, so rotating the normal directly is exact).
#[allow(clippy::too_many_arguments)]
pub fn draw_mesh(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    model: &Mat4,
    rotation: Quat,
    view_proj: &Mat4,
    camera_pos: Vec3,
    shader: &dyn FragmentShader,
    light_dir: Vec3,
    time: f64,
    stats: &mut FrameStats,
) {
    let mut clipped: Vec<ClipVertex> = Vec::with_capacity(4);

    for tri in mesh.indices.chunks_exact(3) {
        stats.triangles_submitted += 1;
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);

        let w0 = model.mul_point(mesh.positions[i0]);
        let w1 = model.mul_point(mesh.positions[i1]);
        let w2 = model.mul_point(mesh.positions[i2]);

        // Backface cull in world space: skip triangles facing away from the camera.
        let face_normal = (w1 - w0).cross(w2 - w0);
        if face_normal.dot(camera_pos - w0) <= 0.0 {
            stats.triangles_backface_culled += 1;
            continue;
        }

        let c0 = view_proj.mul_vec4(Vec4::from_point(w0));
        let c1 = view_proj.mul_vec4(Vec4::from_point(w1));
        let c2 = view_proj.mul_vec4(Vec4::from_point(w2));

        // Trivial reject: all three vertices outside the same clip plane.
        if (c0.z < 0.0 && c1.z < 0.0 && c2.z < 0.0)
            || (c0.x > c0.w && c1.x > c1.w && c2.x > c2.w)
            || (c0.x < -c0.w && c1.x < -c1.w && c2.x < -c2.w)
            || (c0.y > c0.w && c1.y > c1.w && c2.y > c2.w)
            || (c0.y < -c0.w && c1.y < -c1.w && c2.y < -c2.w)
        {
            continue;
        }

        let build = |ci: Vec4, wi: Vec3, idx: usize| ClipVertex {
            clip: ci,
            world: wi,
            normal: rotation.rotate_vec3(mesh.normals[idx]),
            uv: mesh.uvs[idx],
            color: color_to_vec4(mesh.colors[idx]),
        };
        let verts = [build(c0, w0, i0), build(c1, w1, i1), build(c2, w2, i2)];

        let all_in_front = c0.z >= 0.0 && c1.z >= 0.0 && c2.z >= 0.0;
        if all_in_front {
            rasterize_triangle(fb, &verts[0], &verts[1], &verts[2], camera_pos, shader, light_dir, time, stats);
        } else {
            clip_near(&verts, &mut clipped);
            if clipped.len() >= 3 {
                for k in 1..clipped.len() - 1 {
                    rasterize_triangle(
                        fb,
                        &clipped[0],
                        &clipped[k],
                        &clipped[k + 1],
                        camera_pos,
                        shader,
                        light_dir,
                        time,
                        stats,
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn rasterize_triangle(
    fb: &mut Framebuffer,
    v0: &ClipVertex,
    v1: &ClipVertex,
    v2: &ClipVertex,
    camera_pos: Vec3,
    shader: &dyn FragmentShader,
    light_dir: Vec3,
    time: f64,
    stats: &mut FrameStats,
) {
    let (invw0, invw1, invw2) = (1.0 / v0.clip.w, 1.0 / v1.clip.w, 1.0 / v2.clip.w);
    let ndc0 = Vec3::new(v0.clip.x * invw0, v0.clip.y * invw0, v0.clip.z * invw0);
    let ndc1 = Vec3::new(v1.clip.x * invw1, v1.clip.y * invw1, v1.clip.z * invw1);
    let ndc2 = Vec3::new(v2.clip.x * invw2, v2.clip.y * invw2, v2.clip.z * invw2);

    let (fw, fh) = (fb.width as f32, fb.height as f32);
    let to_screen = |n: Vec3| -> (f32, f32) { ((n.x * 0.5 + 0.5) * fw, (1.0 - (n.y * 0.5 + 0.5)) * fh) };
    let s0 = to_screen(ndc0);
    let s1 = to_screen(ndc1);
    let s2 = to_screen(ndc2);

    // Same orientation convention as the edge functions below (edge(s0, s1, p) evaluated at s2).
    let mut area = (s2.0 - s0.0) * (s1.1 - s0.1) - (s2.1 - s0.1) * (s1.0 - s0.0);
    if area.abs() < 1e-6 {
        return; // degenerate
    }
    // Normalize the winding so every edge function is >= 0 inside the triangle.
    let sign = if area < 0.0 { -1.0 } else { 1.0 };
    area *= sign;
    let inv_area = 1.0 / area;

    let min_x = s0.0.min(s1.0).min(s2.0).floor().max(0.0) as i32;
    let max_x = s0.0.max(s1.0).max(s2.0).ceil().min(fw - 1.0) as i32;
    let min_y = s0.1.min(s1.1).min(s2.1).floor().max(0.0) as i32;
    let max_y = s0.1.max(s1.1).max(s2.1).ceil().min(fh - 1.0) as i32;
    if min_x > max_x || min_y > max_y {
        return;
    }
    stats.triangles_rasterized += 1;

    // Edge functions e_i(p) = A_i * p.x + B_i * p.y + C_i, evaluated incrementally: one add per
    // pixel in x and one per row in y, instead of re-evaluating the full expression per pixel.
    let edge_coeffs = |a: (f32, f32), b: (f32, f32)| -> (f32, f32, f32) {
        let aa = (b.1 - a.1) * sign;
        let bb = -(b.0 - a.0) * sign;
        let cc = -(aa * a.0 + bb * a.1);
        (aa, bb, cc)
    };
    let (a0, b0c, c0c) = edge_coeffs(s1, s2);
    let (a1, b1c, c1c) = edge_coeffs(s2, s0);
    let (a2, b2c, c2c) = edge_coeffs(s0, s1);

    let start_x = min_x as f32 + 0.5;
    let start_y = min_y as f32 + 0.5;
    let mut row0 = a0 * start_x + b0c * start_y + c0c;
    let mut row1 = a1 * start_x + b1c * start_y + c1c;
    let mut row2 = a2 * start_x + b2c * start_y + c2c;

    let width = fb.width as usize;
    let col0 = v0.color;
    let col1 = v1.color;
    let col2 = v2.color;

    for y in min_y..=max_y {
        let mut e0 = row0;
        let mut e1 = row1;
        let mut e2 = row2;
        let row_base = y as usize * width;

        for x in min_x..=max_x {
            if e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0 {
                let (b0, b1, b2) = (e0 * inv_area, e1 * inv_area, e2 * inv_area);
                let depth = b0 * ndc0.z + b1 * ndc1.z + b2 * ndc2.z;
                let idx = row_base + x as usize;
                if depth >= 0.0 && depth < fb.depth[idx] {
                    // Perspective-correct weights, normalized once and reused for every attribute.
                    let l0 = b0 * invw0;
                    let l1 = b1 * invw1;
                    let l2 = b2 * invw2;
                    let inv_sum = 1.0 / (l0 + l1 + l2);
                    let (l0, l1, l2) = (l0 * inv_sum, l1 * inv_sum, l2 * inv_sum);

                    let world_pos = v0.world * l0 + v1.world * l1 + v2.world * l2;
                    let normal = (v0.normal * l0 + v1.normal * l1 + v2.normal * l2).normalized();
                    let uv = Vec2::new(
                        v0.uv.x * l0 + v1.uv.x * l1 + v2.uv.x * l2,
                        v0.uv.y * l0 + v1.uv.y * l1 + v2.uv.y * l2,
                    );
                    let base_color = Color::new(
                        (col0.x * l0 + col1.x * l1 + col2.x * l2).clamp(0.0, 255.0) as u8,
                        (col0.y * l0 + col1.y * l1 + col2.y * l2).clamp(0.0, 255.0) as u8,
                        (col0.z * l0 + col1.z * l1 + col2.z * l2).clamp(0.0, 255.0) as u8,
                        (col0.w * l0 + col1.w * l1 + col2.w * l2).clamp(0.0, 255.0) as u8,
                    );

                    let ctx = ShadingContext {
                        world_pos,
                        normal,
                        uv,
                        base_color,
                        light_dir,
                        view_dir: camera_pos - world_pos,
                        time,
                    };
                    let color = shader.shade(&ctx);
                    fb.depth[idx] = depth;
                    fb.color[idx] = color;
                }
            }
            e0 += a0;
            e1 += a1;
            e2 += a2;
        }
        row0 += b0c;
        row1 += b1c;
        row2 += b2c;
    }
}
