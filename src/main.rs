mod math;
mod navigation;
mod perf;
mod render;
mod scene;

use math::{Quat, Vec3};
use navigation::{collision, controller, gravity};
use perf::{frustum, lod, stats::FrameStats};
use raylib::prelude::*;
use render::{camera::Camera, pipeline, shader::LambertShader, sky::Sky, Framebuffer};
use scene::{CelestialBody, SolarSystem};

const WINDOW_W: i32 = 1280;
const WINDOW_H: i32 = 720;

/// Props are drawn only while the camera is within this many planet radii; farther than that they
/// are sub-pixel and only cost triangles.
const PROP_VISIBLE_RADII: f32 = 22.0;
/// Minimum projected size (in pixels of diameter) for an individual prop to be worth drawing.
const PROP_MIN_PIXELS: f32 = 3.0;

fn main() {
    let (mut rl, thread) = raylib::init().size(WINDOW_W, WINDOW_H).title("Space Travel").build();
    rl.set_target_fps(60);
    rl.disable_cursor();

    let image = Image::gen_image_color(WINDOW_W, WINDOW_H, Color::new(3, 3, 10, 255));
    let mut texture = rl.load_texture_from_image(&thread, &image).expect("no se pudo crear la textura de presentacion");

    let mut framebuffer = Framebuffer::new(WINDOW_W, WINDOW_H);
    let mut rgba = Vec::with_capacity((WINDOW_W * WINDOW_H * 4) as usize);
    let mut solar_system = SolarSystem::new();
    let sky = Sky::new(2200, 7);
    let mut camera = Camera::new(WINDOW_W as f32 / WINDOW_H as f32);

    let controller_cfg = controller::ControllerConfig::default();
    let gravity_cfg = gravity::GravityConfig::default();
    let ship_shader = LambertShader { ambient: 0.42 };

    // Chase-cam tuning (ship-local frame). Distance stretches with speed and the FOV opens up
    // while boosting, which is what sells acceleration in a static-camera game.
    let cam_back = 11.0f32;
    let cam_up = 3.2f32;
    let base_fov = 62.0f32.to_radians();
    let boost_fov_extra = 9.0f32.to_radians();
    let mut visual_bank = 0.0f32;
    // 0..1 strength of the light-speed visual effect (ramps in/out around the boost key).
    let mut warp = 0.0f32;

    // Settle the camera behind the ship on the first frame instead of lerping in from the origin.
    {
        let ship = &solar_system.ship;
        camera.position = ship.position + ship.up() * cam_up - ship.forward() * cam_back;
        camera.orientation = ship.orientation;
        camera.fov_y_radians = base_fov;
    }

    let mut stats = FrameStats::default();
    let mut show_hud = true;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.05);
        solar_system.sim_time += dt as f64;
        solar_system.update_bodies();

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            solar_system.regenerate_surfaces(solar_system.surface_seed.wrapping_add(1));
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            show_hud = !show_hud;
        }

        // Built from the individual fields (not `solar_system.bodies()`) so the borrow checker
        // sees it as disjoint from the `&mut solar_system.ship` borrows right below.
        let body_refs: Vec<&CelestialBody> =
            std::iter::once(&solar_system.sun).chain(solar_system.planets.iter()).collect();

        let ctrl_out = controller::update_ship(&mut rl, &mut solar_system.ship, &body_refs, &controller_cfg, dt);
        gravity::apply_gravity(&mut solar_system.ship, &body_refs, &gravity_cfg, dt);
        collision::resolve_collisions(&mut solar_system.ship, &body_refs);

        // --- Chase camera: smoothed position + orientation, speed-based distance, bank into turns ---
        let ship = &solar_system.ship;
        let speed_frac = (ctrl_out.speed / controller_cfg.max_speed).clamp(0.0, 1.0);
        let target_bank = -ctrl_out.yaw_rate * 0.28;
        visual_bank += (target_bank - visual_bank) * (1.0 - (-8.0 * dt).exp());
        let ship_visual_orientation = ship.orientation * Quat::from_axis_angle(Vec3::FORWARD, visual_bank);

        let desired_cam_pos = ship.position + ship.up() * cam_up - ship.forward() * (cam_back * (1.0 + 0.45 * speed_frac));
        camera.position = camera.position.lerp(desired_cam_pos, 1.0 - (-10.0 * dt).exp());
        let cam_target_orientation = ship.orientation * Quat::from_axis_angle(Vec3::FORWARD, visual_bank * 0.4);
        camera.orientation = camera.orientation.slerp(cam_target_orientation, 1.0 - (-14.0 * dt).exp());
        let warp_target = if ctrl_out.boosting { 1.0 } else { 0.0 };
        warp += (warp_target - warp) * (1.0 - (-(if ctrl_out.boosting { 3.0 } else { 5.0 }) * dt).exp());
        let target_fov = base_fov + if ctrl_out.boosting { boost_fov_extra } else { 0.0 } + speed_frac * 3.0f32.to_radians();
        camera.fov_y_radians += (target_fov - camera.fov_y_radians) * (1.0 - (-5.0 * dt).exp());

        // Keep the camera itself out of planets (the ship collides, but the camera trails behind
        // it and could otherwise dip below the surface when flying nose-up from a landing).
        for body in &body_refs {
            let clearance = body.collision_radius() + 1.5;
            let offset = camera.position - body.position;
            let d = offset.length();
            if d < clearance && d > 1e-4 {
                camera.position = body.position + offset * (clearance / d);
            }
        }

        let view_proj = camera.view_proj();
        let planes = frustum::extract_planes(&view_proj);
        let focal_px = (WINDOW_H as f32 * 0.5) / (camera.fov_y_radians * 0.5).tan();

        stats.reset();
        stats.bodies_total = body_refs.len() as u32;
        framebuffer.clear(Color::new(3, 3, 10 + (warp * 18.0) as u8, 255));
        sky.draw_stars(&mut framebuffer, &view_proj, camera.position, solar_system.sim_time, warp);
        Sky::draw_warp_vignette(&mut framebuffer, warp);
        let sun_pos = solar_system.sun.position;
        Sky::draw_sun_glow(&mut framebuffer, &view_proj, sun_pos, solar_system.sun.radius, camera.up(), solar_system.sim_time);

        for body in solar_system.bodies() {
            let dist = camera.position.distance(body.position);
            if !frustum::sphere_in_frustum(&planes, body.position, body.collision_radius()) {
                stats.bodies_culled += 1;
                continue;
            }
            let thresholds = body.lod_thresholds();
            let lod_idx = lod::choose_lod(dist, &thresholds, body.mesh_lods.len());
            let mesh = &body.mesh_lods[lod_idx];
            let light_dir = if body.orbit.is_some() { (body.position - sun_pos).normalized() } else { Vec3::FORWARD };
            let body_model = body.model_matrix();

            pipeline::draw_mesh(
                &mut framebuffer,
                mesh,
                &body_model,
                body.rotation,
                &view_proj,
                camera.position,
                body.shader.as_ref(),
                light_dir,
                solar_system.sim_time,
                &mut stats,
            );

            if body.prop_meshes.is_empty() || dist > body.radius * PROP_VISIBLE_RADII {
                continue;
            }
            stats.instances_total += body.surface_instances.len() as u32;
            for inst in &body.surface_instances {
                let world_pos = body_model.mul_point(inst.local_pos);
                let bounding_radius = body.radius * inst.scale * 0.45;

                // Horizon cull: a prop on the far side of the planet can't be seen, no matter
                // what the frustum says -- this alone removes about half of all instances.
                let surface_normal = (world_pos - body.position).normalized();
                let to_camera = camera.position - world_pos;
                if surface_normal.dot(to_camera) < -bounding_radius {
                    stats.instances_culled += 1;
                    continue;
                }
                // Size cull: skip props that would cover only a couple of pixels.
                let prop_dist = to_camera.length().max(1e-3);
                if (2.0 * bounding_radius / prop_dist) * focal_px < PROP_MIN_PIXELS {
                    stats.instances_culled += 1;
                    continue;
                }
                if !frustum::sphere_in_frustum(&planes, world_pos, bounding_radius) {
                    stats.instances_culled += 1;
                    continue;
                }
                pipeline::draw_mesh(
                    &mut framebuffer,
                    &body.prop_meshes[inst.prop_type],
                    &body.instance_model_matrix_with(&body_model, inst),
                    body.instance_rotation(inst),
                    &view_proj,
                    camera.position,
                    body.prop_shader.as_ref(),
                    light_dir,
                    solar_system.sim_time,
                    &mut stats,
                );
            }
        }

        let ship = &solar_system.ship;
        let ship_light_dir = (ship.position - sun_pos).normalized();
        let ship_model = math::Mat4::translation(ship.position)
            * ship_visual_orientation.to_mat4()
            * math::Mat4::scaling(ship.mesh_scale);
        pipeline::draw_mesh(
            &mut framebuffer,
            &ship.mesh,
            &ship_model,
            ship_visual_orientation,
            &view_proj,
            camera.position,
            &ship_shader,
            ship_light_dir,
            solar_system.sim_time,
            &mut stats,
        );

        framebuffer.write_rgba_into(&mut rgba);
        texture.update_texture(&rgba).expect("fallo al actualizar la textura");

        let fps = rl.get_fps();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &texture,
            Rectangle::new(0.0, 0.0, WINDOW_W as f32, WINDOW_H as f32),
            Rectangle::new(0.0, 0.0, WINDOW_W as f32, WINDOW_H as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        // --- Planet name tags: 2D labels anchored just above each body's projected top ---
        if show_hud {
            for body in solar_system.bodies() {
                // Skip bodies hidden behind another one (e.g. a planet on the far side of the sun).
                let to_body = body.position - camera.position;
                let body_dist = to_body.length();
                let ray = to_body * (1.0 / body_dist.max(1e-3));
                let occluded = solar_system.bodies().any(|other| {
                    if std::ptr::eq(other, body) {
                        return false;
                    }
                    let rel = other.position - camera.position;
                    let t = rel.dot(ray);
                    if t <= 0.0 || t >= body_dist {
                        return false;
                    }
                    let perp = (rel - ray * t).length();
                    perp < other.collision_radius()
                });
                if occluded {
                    continue;
                }
                let anchor = body.position + camera.up() * (body.collision_radius() * 1.12);
                let clip = view_proj.mul_vec4(math::Vec4::from_point(anchor));
                if clip.w <= 0.0 {
                    continue;
                }
                let nx = clip.x / clip.w;
                let ny = clip.y / clip.w;
                if nx < -1.05 || nx > 1.05 || ny < -1.05 || ny > 1.05 {
                    continue;
                }
                let sx = ((nx * 0.5 + 0.5) * WINDOW_W as f32) as i32;
                let sy = ((1.0 - (ny * 0.5 + 0.5)) * WINDOW_H as f32) as i32;
                let dist = camera.position.distance(body.position) - body.collision_radius();
                let label = format!("{}  {:.0} u", body.name, dist.max(0.0));
                let font = if dist < 120.0 { 20 } else { 16 };
                let tw = d.measure_text(&label, font);
                let (tx, ty) = (sx - tw / 2, sy - font - 10);
                d.draw_rectangle(tx - 6, ty - 3, tw + 12, font + 6, Color::new(0, 0, 0, 140));
                d.draw_text(&label, tx, ty, font, Color::new(255, 240, 200, 255));
                d.draw_line(sx, ty + font + 3, sx, sy, Color::new(255, 240, 200, 170));
            }
        }

        if show_hud {
            let nearest = ctrl_out.nearest_body_name.as_deref().unwrap_or("-");
            let mode = if ctrl_out.local_mode { "LOCAL" } else { "crucero" };
            let hud = format!(
                "FPS {}   |   velocidad {:>5.1}{}   |   modo {}\nCerca de: {}  ({:.0} u a la superficie)",
                fps,
                ctrl_out.speed,
                if ctrl_out.boosting { " BOOST" } else { "" },
                mode,
                nearest,
                ctrl_out.nearest_surface_distance.max(0.0),
            );
            d.draw_text(&hud, 16, 14, 20, Color::new(190, 255, 200, 255));

            let perf = format!(
                "cuerpos {}/{}  props {}/{}  tris {} env / {} back / {} rast  |  semilla {}",
                stats.bodies_total - stats.bodies_culled,
                stats.bodies_total,
                stats.instances_total - stats.instances_culled,
                stats.instances_total,
                stats.triangles_submitted,
                stats.triangles_backface_culled,
                stats.triangles_rasterized,
                solar_system.surface_seed,
            );
            d.draw_text(&perf, 16, 64, 16, Color::new(140, 170, 160, 255));
            d.draw_text(
                "W/S empuje  |  mouse mirar  |  A/D roll  |  Shift boost  |  Space frenar  |  R nueva semilla  |  H ocultar HUD",
                16,
                WINDOW_H - 26,
                16,
                Color::new(200, 200, 210, 255),
            );
        }
    }
}
