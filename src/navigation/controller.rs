use crate::math::{Quat, Vec3};
use crate::scene::{CelestialBody, Ship};
use raylib::prelude::*;

pub struct ControllerConfig {
    pub thrust_accel: f32,
    pub boost_multiplier: f32,
    pub max_speed: f32,
    pub local_mode_max_speed: f32,
    /// Fraction of velocity remaining after 1 s with no thrust key held. Very small on purpose:
    /// the ship only moves while W/S is pressed and comes to rest right after releasing.
    pub ambient_damping_per_sec: f32,
    pub brake_damping_per_sec: f32,   // same, but while the brake key is held
    /// Maximum pitch/yaw rate (radians/second) the mouse can command, regardless of how fast it
    /// is moved -- keeps a flick of the wrist from snapping the view around.
    pub max_turn_rate: f32,
    pub roll_rate: f32,
    /// Radians of commanded turn per pixel of mouse travel.
    pub mouse_sensitivity: f32,
    /// How quickly the actual angular velocity catches up with the commanded one (1/s). Higher is
    /// snappier, lower is heavier/smoother. This is what makes the turning feel "stable": the
    /// orientation never reacts to a raw per-frame mouse delta, only to a filtered rate.
    pub turn_responsiveness: f32,
    /// How fast the velocity vector bends toward the nose (1/s), so the ship actually flies where
    /// it is pointing instead of drifting sideways forever. Stronger while thrusting.
    pub heading_alignment_thrusting: f32,
    pub heading_alignment_coasting: f32,
    pub local_mode_distance: f32, // distance-to-surface under which "local mode" kicks in
}

impl Default for ControllerConfig {
    fn default() -> Self {
        ControllerConfig {
            thrust_accel: 55.0,
            boost_multiplier: 2.6,
            max_speed: 240.0,
            local_mode_max_speed: 45.0,
            ambient_damping_per_sec: 0.004,
            brake_damping_per_sec: 0.03,
            max_turn_rate: 2.4,
            roll_rate: 2.0,
            mouse_sensitivity: 0.0022,
            turn_responsiveness: 9.0,
            heading_alignment_thrusting: 2.8,
            heading_alignment_coasting: 0.9,
            local_mode_distance: 60.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct ControllerOutput {
    pub local_mode: bool,
    pub nearest_surface_distance: f32,
    pub nearest_body_name: Option<String>,
    pub speed: f32,
    pub boosting: bool,
    /// Commanded (filtered) yaw rate, for the visual banking of ship/camera.
    pub yaw_rate: f32,
}

pub fn update_ship(
    rl: &mut RaylibHandle,
    ship: &mut Ship,
    bodies: &[&CelestialBody],
    cfg: &ControllerConfig,
    dt: f32,
) -> ControllerOutput {
    let mut nearest_surface_distance = f32::INFINITY;
    let mut nearest_body_name: Option<String> = None;
    for b in bodies {
        let d = ship.position.distance(b.position) - b.collision_radius();
        if d < nearest_surface_distance {
            nearest_surface_distance = d;
            nearest_body_name = Some(b.name.clone());
        }
    }
    let local_mode = nearest_surface_distance < cfg.local_mode_distance;

    // --- Orientation: full local-space rotation (6DOF), avoids gimbal lock via quaternions ---
    // Mouse travel is converted to a *target* angular rate, clamped, and then the real angular
    // velocity is low-pass filtered toward it. The result is frame-rate independent and immune to
    // single-frame mouse spikes, which is what made the old direct-delta version feel twitchy.
    let mouse_delta = rl.get_mouse_delta();
    let dt_safe = dt.max(1.0 / 240.0);
    let target_yaw = (-mouse_delta.x * cfg.mouse_sensitivity / dt_safe).clamp(-cfg.max_turn_rate, cfg.max_turn_rate);
    let target_pitch =
        (-mouse_delta.y * cfg.mouse_sensitivity / dt_safe).clamp(-cfg.max_turn_rate, cfg.max_turn_rate);

    let mut roll_input = 0.0f32;
    if rl.is_key_down(KeyboardKey::KEY_A) {
        roll_input += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        roll_input -= 1.0;
    }
    let target_roll = roll_input * cfg.roll_rate;

    let blend = 1.0 - (-cfg.turn_responsiveness * dt).exp();
    let target_rates = Vec3::new(target_pitch, target_yaw, target_roll);
    ship.angular_velocity = ship.angular_velocity.lerp(target_rates, blend);

    let rates = ship.angular_velocity;
    let delta_rot = Quat::from_axis_angle(Vec3::RIGHT, rates.x * dt)
        * Quat::from_axis_angle(Vec3::UP, rates.y * dt)
        * Quat::from_axis_angle(Vec3::FORWARD, rates.z * dt);
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

    // Bend the velocity toward the nose (arcade-style handling). Keeps the magnitude, only turns
    // the direction, so braking/acceleration still behave as before.
    let speed = ship.velocity.length();
    if speed > 0.05 {
        let forward = ship.forward();
        let along = if ship.velocity.dot(forward) >= 0.0 { 1.0 } else { -1.0 };
        let desired = forward * (speed * along);
        let rate = if thrust_input != 0.0 { cfg.heading_alignment_thrusting } else { cfg.heading_alignment_coasting };
        let t = 1.0 - (-rate * dt).exp();
        let bent = ship.velocity.lerp(desired, t);
        let bent_len = bent.length();
        if bent_len > 1e-5 {
            ship.velocity = bent * (speed / bent_len);
        }
    }

    // Progressive, frame-rate-independent braking/damping (velocity *= coeff^dt).
    let damping_coeff = if braking { cfg.brake_damping_per_sec } else { cfg.ambient_damping_per_sec };
    ship.velocity = ship.velocity * damping_coeff.powf(dt);

    // Fully stop once the ship is crawling so it never drifts forever at a sub-visible speed.
    if thrust_input == 0.0 && ship.velocity.length() < 0.6 {
        ship.velocity = Vec3::ZERO;
    }

    let max_speed = if local_mode { cfg.local_mode_max_speed } else { cfg.max_speed };
    let speed = ship.velocity.length();
    if speed > max_speed {
        // Ease into the cap instead of clamping hard, so entering local mode at high speed slows
        // the ship over a fraction of a second rather than in a single frame.
        let eased = speed + (max_speed - speed) * (1.0 - (-6.0 * dt).exp());
        ship.velocity = ship.velocity * (eased / speed);
    }

    ship.position += ship.velocity * dt;

    ControllerOutput {
        local_mode,
        nearest_surface_distance,
        nearest_body_name,
        speed: ship.velocity.length(),
        boosting: boosting && thrust_input > 0.0,
        yaw_rate: rates.y,
    }
}
