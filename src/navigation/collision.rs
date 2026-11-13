use crate::scene::{CelestialBody, Ship};

/// Simple sphere-vs-sphere collision: pushes the ship out to the surface and cancels the
/// velocity component pointing into the body, so it slides/stops instead of clipping through.
/// Uses `collision_radius()` (visual radius + terrain bulge) so noise-deformed planets don't let
/// the ship sink into their hills.
pub fn resolve_collisions(ship: &mut Ship, bodies: &[&CelestialBody]) {
    for body in bodies {
        let offset = ship.position - body.position;
        let dist = offset.length();
        let min_dist = body.collision_radius() + ship.collision_radius;
        if dist < min_dist {
            let normal = if dist > 1e-5 { offset * (1.0 / dist) } else { crate::math::Vec3::UP };
            ship.position = body.position + normal * min_dist;
            let into_surface = ship.velocity.dot(normal);
            if into_surface < 0.0 {
                // Remove the inward component and bleed off a bit of the tangential speed too,
                // so a scrape along the surface feels like contact rather than frictionless ice.
                ship.velocity -= normal * into_surface;
                ship.velocity = ship.velocity * 0.85;
            }
        }
    }
}
