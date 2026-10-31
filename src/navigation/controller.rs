use crate::math::{Quat, Vec3};
use crate::scene::{CelestialBody, Ship};
use raylib::prelude::*;

pub struct ControllerConfig {
    pub thrust_accel: f32,
    pub boost_multiplier: f32,
    pub max_speed: f32,
    pub local_mode_max_speed: f32,
    pub ambient_damping_per_sec: f32, // fraction of velocity remaining after 1s with no thrust/brake
    pub brake_damping_per_sec: f32,   // same, but while the brake key is held
    pub turn_speed_radians: f32,      // pitch/yaw rate at full mouse delta reference
    pub roll_speed_radians: f32,
    pub mouse_sensitivity: f32,
    pub local_mode_distance: f32, // distance-to-surface under which "local mode" kicks in
}

impl Default for ControllerConfig {
    fn default() -> Self {
        ControllerConfig {
            thrust_accel: 60.0,
            boost_multiplier: 2.5,
            max_speed: 220.0,
            local_mode_max_speed: 40.0,
            ambient_damping_per_sec: 0.35,
            brake_damping_per_sec: 0.02,
            turn_speed_radians: 2.2,
            roll_speed_radians: 1.8,
            mouse_sensitivity: 0.0025,
            local_mode_distance: 35.0,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ControllerOutput {
    pub local_mode: bool,
    pub nearest_surface_distance: f32,
    pub speed: f32,
}

pub fn update_ship(
    rl: &mut RaylibHandle,
    ship: &mut Ship,
    bodies: &[&CelestialBody],
    cfg: &ControllerConfig,
    dt: f32,
) -> ControllerOutput {
    let nearest_surface_distance = bodies
        .iter()
        .map(|b| ship.position.distance(b.position) - b.radius)
        .fold(f32::INFINITY, f32::min);
    let local_mode = nearest_surface_distance < cfg.local_mode_distance;

    // --- Orientation: full local-space rotation (6DOF), avoids gimbal lock via quaternions ---
    let mouse_delta = rl.get_mouse_delta();
    let yaw_input = -mouse_delta.x * cfg.mouse_sensitivity;
    let pitch_input = -mouse_delta.y * cfg.mouse_sensitivity;
    let mut roll_input = 0.0f32;
    if rl.is_key_down(KeyboardKey::KEY_A) {
        roll_input += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        roll_input -= 1.0;
    }

    let delta_rot = Quat::from_axis_angle(Vec3::RIGHT, pitch_input * cfg.turn_speed_radians)
        * Quat::from_axis_angle(Vec3::UP, yaw_input * cfg.turn_speed_radians)
        * Quat::from_axis_angle(Vec3::FORWARD, roll_input * cfg.roll_speed_radians * dt);
    ship.orientation = (ship.orientation * delta_rot).normalized();

    // --- Thrust: progressive acceleration, not an instant velocity set ---
    let mut thrust_input = 0.0f32;
    if rl.is_key_down(KeyboardKey::KEY_W) {
        thrust_input += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        thrust_input -= 1.0;
    }
    let boosting = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
    let braking = rl.is_key_down(KeyboardKey::KEY_SPACE);

    if thrust_input != 0.0 {
        let boost = if boosting { cfg.boost_multiplier } else { 1.0 };
        ship.velocity += ship.forward() * (thrust_input * cfg.thrust_accel * boost * dt);
    }

    // Progressive, frame-rate-independent braking/damping (velocity *= coeff^dt).
    let damping_coeff = if braking { cfg.brake_damping_per_sec } else { cfg.ambient_damping_per_sec };
    ship.velocity = ship.velocity * damping_coeff.powf(dt);

    let max_speed = if local_mode { cfg.local_mode_max_speed } else { cfg.max_speed };
    let speed = ship.velocity.length();
    if speed > max_speed {
        ship.velocity = ship.velocity * (max_speed / speed);
    }

    ship.position += ship.velocity * dt;

    ControllerOutput { local_mode, nearest_surface_distance, speed: ship.velocity.length() }
}
