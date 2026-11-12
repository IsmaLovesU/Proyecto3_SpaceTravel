use super::framebuffer::Framebuffer;
use super::mesh::Mesh;
use super::shader::{FragmentShader, ShadingContext};
use crate::math::{Mat4, Quat, Vec3, Vec4};
use crate::perf::stats::FrameStats;

#[inline]
fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0)
}

/// Transforms `mesh` by `model`/`rotation` and rasterizes it into `fb` with a per-pixel depth
/// test and a pluggable fragment shader. `rotation` is passed separately from `model` so normals
/// can be rotated without needing a general inverse-transpose (every body in this phase only
/// uses uniform scale, so rotating the normal directly is exact).
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

        if c0.w <= 1e-5 && c1.w <= 1e-5 && c2.w <= 1e-5 {
            continue; // fully behind the near plane
        }
        let (invw0, invw1, invw2) = (1.0 / c0.w.max(1e-5), 1.0 / c1.w.max(1e-5), 1.0 / c2.w.max(1e-5));

        let ndc0 = Vec3::new(c0.x * invw0, c0.y * invw0, c0.z * invw0);
        let ndc1 = Vec3::new(c1.x * invw1, c1.y * invw1, c1.z * invw1);
        let ndc2 = Vec3::new(c2.x * invw2, c2.y * invw2, c2.z * invw2);

        let to_screen = |n: Vec3| -> (f32, f32) {
            ((n.x * 0.5 + 0.5) * fb.width as f32, (1.0 - (n.y * 0.5 + 0.5)) * fb.height as f32)
        };
        let s0 = to_screen(ndc0);
        let s1 = to_screen(ndc1);
        let s2 = to_screen(ndc2);

        let area = edge(s0, s1, s2);
        if area.abs() < 1e-6 {
            continue; // degenerate triangle
        }

        let min_x = s0.0.min(s1.0).min(s2.0).floor().max(0.0) as i32;
        let max_x = s0.0.max(s1.0).max(s2.0).ceil().min(fb.width as f32 - 1.0) as i32;
        let min_y = s0.1.min(s1.1).min(s2.1).floor().max(0.0) as i32;
        let max_y = s0.1.max(s1.1).max(s2.1).ceil().min(fb.height as f32 - 1.0) as i32;
        if min_x > max_x || min_y > max_y {
            continue;
        }

        let n0 = rotation.rotate_vec3(mesh.normals[i0]);
        let n1 = rotation.rotate_vec3(mesh.normals[i1]);
        let n2 = rotation.rotate_vec3(mesh.normals[i2]);
        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];
        let col0 = mesh.colors[i0];
        let col1 = mesh.colors[i1];
        let col2 = mesh.colors[i2];

        stats.triangles_rasterized += 1;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = (x as f32 + 0.5, y as f32 + 0.5);
                let w0e = edge(s1, s2, p);
                let w1e = edge(s2, s0, p);
                let w2e = edge(s0, s1, p);
                let inside = (w0e >= 0.0 && w1e >= 0.0 && w2e >= 0.0) || (w0e <= 0.0 && w1e <= 0.0 && w2e <= 0.0);
                if !inside {
                    continue;
                }
                let (b0, b1, b2) = (w0e / area, w1e / area, w2e / area);

                let depth = b0 * ndc0.z + b1 * ndc1.z + b2 * ndc2.z;
                if !fb.depth_test(x, y, depth) {
                    continue;
                }

                let pw = b0 * invw0 + b1 * invw1 + b2 * invw2;
                let persp = |a: f32, b: f32, c: f32| (a * b0 * invw0 + b * b1 * invw1 + c * b2 * invw2) / pw;

                let world_pos = Vec3::new(
                    persp(w0.x, w1.x, w2.x),
                    persp(w0.y, w1.y, w2.y),
                    persp(w0.z, w1.z, w2.z),
                );
                let normal = Vec3::new(
                    persp(n0.x, n1.x, n2.x),
                    persp(n0.y, n1.y, n2.y),
                    persp(n0.z, n1.z, n2.z),
                )
                .normalized();
                let uv = crate::math::Vec2::new(persp(uv0.x, uv1.x, uv2.x), persp(uv0.y, uv1.y, uv2.y));
                let base_color = raylib::color::Color::new(
                    persp(col0.r as f32, col1.r as f32, col2.r as f32).clamp(0.0, 255.0) as u8,
                    persp(col0.g as f32, col1.g as f32, col2.g as f32).clamp(0.0, 255.0) as u8,
                    persp(col0.b as f32, col1.b as f32, col2.b as f32).clamp(0.0, 255.0) as u8,
                    persp(col0.a as f32, col1.a as f32, col2.a as f32).clamp(0.0, 255.0) as u8,
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
                fb.write_pixel(x, y, depth, color);
            }
        }
    }
}
