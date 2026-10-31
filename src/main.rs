mod math;
mod navigation;
mod perf;
mod render;
mod scene;

use math::Vec3;
use navigation::{collision, controller, gravity};
use perf::{frustum, lod, stats::FrameStats};
use raylib::prelude::*;
use render::{camera::Camera, pipeline, shader::LambertShader, Framebuffer};
use scene::{CelestialBody, SolarSystem};

const WINDOW_W: i32 = 1280;
const WINDOW_H: i32 = 720;

fn main() {
    let (mut rl, thread) = raylib::init().size(WINDOW_W, WINDOW_H).title("Space Travel - fase preliminar").build();
    rl.set_target_fps(60);
    rl.disable_cursor();

    let image = Image::gen_image_color(WINDOW_W, WINDOW_H, Color::new(5, 5, 15, 255));
    let mut texture = rl.load_texture_from_image(&thread, &image).expect("no se pudo crear la textura de presentacion");

    let mut framebuffer = Framebuffer::new(WINDOW_W, WINDOW_H);
    let mut solar_system = SolarSystem::new();
    let mut camera = Camera::new(WINDOW_W as f32 / WINDOW_H as f32);

    let controller_cfg = controller::ControllerConfig::default();
    let gravity_cfg = gravity::GravityConfig::default();
    let ship_shader = LambertShader { ambient: 0.2 };

    // Chase-cam offsets, in the ship's own local frame.
    let cam_back = 14.0f32;
    let cam_up = 4.0f32;
    let cam_smooth_per_sec = 6.0f32;

    let mut stats = FrameStats::default();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        solar_system.sim_time += dt as f64;
        solar_system.update_bodies();

        // Built from the individual fields (not `solar_system.bodies()`) so the borrow checker
        // sees it as disjoint from the `&mut solar_system.ship` borrows right below.
        let body_refs: Vec<&CelestialBody> =
            std::iter::once(&solar_system.sun).chain(solar_system.planets.iter()).collect();

        let ctrl_out = controller::update_ship(&mut rl, &mut solar_system.ship, &body_refs, &controller_cfg, dt);
        gravity::apply_gravity(&mut solar_system.ship, &body_refs, &gravity_cfg, dt);
        collision::resolve_collisions(&mut solar_system.ship, &body_refs);

        let ship = &solar_system.ship;
        let desired_cam_pos = ship.position + ship.up() * cam_up - ship.forward() * cam_back;
        let smooth_t = (cam_smooth_per_sec * dt).min(1.0);
        camera.position = camera.position.lerp(desired_cam_pos, smooth_t);
        camera.orientation = ship.orientation;

        let view_proj = camera.view_proj();
        let planes = frustum::extract_planes(&view_proj);

        stats.reset();
        stats.bodies_total = body_refs.len() as u32;
        framebuffer.clear(Color::new(5, 5, 15, 255));

        let sun_pos = solar_system.sun.position;
        for body in solar_system.bodies() {
            let dist = camera.position.distance(body.position);
            if !frustum::sphere_in_frustum(&planes, body.position, body.radius) {
                stats.bodies_culled += 1;
                continue;
            }
            let thresholds = body.lod_thresholds();
            let lod_idx = lod::choose_lod(dist, &thresholds, body.mesh_lods.len());
            let mesh = &body.mesh_lods[lod_idx];
            let light_dir = if body.orbit.is_some() { (body.position - sun_pos).normalized() } else { Vec3::FORWARD };

            pipeline::draw_mesh(
                &mut framebuffer,
                mesh,
                &body.model_matrix(),
                body.rotation,
                &view_proj,
                camera.position,
                body.color,
                body.shader.as_ref(),
                light_dir,
                solar_system.sim_time,
                &mut stats,
            );
        }

        let ship_light_dir = (solar_system.ship.position - sun_pos).normalized();
        pipeline::draw_mesh(
            &mut framebuffer,
            &solar_system.ship.mesh,
            &solar_system.ship.model_matrix(),
            solar_system.ship.orientation,
            &view_proj,
            camera.position,
            Color::new(205, 205, 215, 255),
            &ship_shader,
            ship_light_dir,
            solar_system.sim_time,
            &mut stats,
        );

        texture.update_texture(&framebuffer.to_rgba_bytes()).expect("fallo al actualizar la textura");

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

        let hud = format!(
            "FPS: {}\nVelocidad: {:.1}\nModo local: {}\nCuerpos: {}/{} visibles\nTriangulos: {} enviados | {} back-face | {} rasterizados",
            fps,
            ctrl_out.speed,
            if ctrl_out.local_mode { "SI" } else { "no" },
            stats.bodies_total - stats.bodies_culled,
            stats.bodies_total,
            stats.triangles_submitted,
            stats.triangles_backface_culled,
            stats.triangles_rasterized,
        );
        d.draw_text(&hud, 16, 16, 20, Color::LIME);
        d.draw_text(
            "WASD mover | Mouse mirar | A/D roll | Shift boost | Space frenar",
            16,
            WINDOW_H - 28,
            18,
            Color::RAYWHITE,
        );
    }
}
