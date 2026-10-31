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
        GravityConfig { g_constant: 1200.0, max_accel: 45.0, min_distance: 8.0 }
    }
}

pub fn apply_gravity(ship: &mut Ship, bodies: &[&CelestialBody], cfg: &GravityConfig, dt: f32) {
    for body in bodies {
        let to_body = body.position - ship.position;
        let dist = to_body.length().max(cfg.min_distance);
        let accel_mag = (cfg.g_constant * body.mass / (dist * dist)).min(cfg.max_accel);
        ship.velocity += to_body.normalized() * (accel_mag * dt);
    }
}
