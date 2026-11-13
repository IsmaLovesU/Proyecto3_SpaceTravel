use crate::scene::{CelestialBody, Ship};

pub struct GravityConfig {
    pub g_constant: f32,
    pub max_accel: f32,
    /// Minimum distance used in the inverse-square falloff, so getting very close to a body
    /// doesn't fling the ship away with an absurd acceleration spike.
    pub min_distance: f32,
}

impl Default for GravityConfig {
    fn default() -> Self {
        // Tuned to be clearly noticeable when coasting near a planet (the ship visibly curves
        // toward it) without yanking the nose around while the player is trying to line up a
        // close fly-by. Massive bodies still pull from far away, small ones only up close.
        GravityConfig { g_constant: 320.0, max_accel: 9.0, min_distance: 10.0 }
    }
}

pub fn apply_gravity(ship: &mut Ship, bodies: &[&CelestialBody], cfg: &GravityConfig, dt: f32) {
    // Gravity bends the trajectory while flying, but a parked ship stays parked: scale the pull
    // by how fast the ship is already moving (full strength above a few units/second).
    let motion_factor = (ship.velocity.length() / 6.0).clamp(0.0, 1.0);
    if motion_factor <= 0.0 {
        return;
    }
    for body in bodies {
        let to_body = body.position - ship.position;
        let dist = to_body.length().max(cfg.min_distance);
        let accel_mag = (cfg.g_constant * body.mass / (dist * dist)).min(cfg.max_accel);
        ship.velocity += to_body.normalized() * (accel_mag * motion_factor * dt);
    }
}
