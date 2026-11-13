//! Procedural styling for the player's ship. The base hull comes from `assets/models/ship.obj`
//! (Blender); this module repaints it per vertex and bolts on extra procedural parts (engines,
//! cockpit dome, winglets, dorsal fin, lights) built from the same primitives as the planet props,
//! so the ship reads as a hero object instead of a flat grey mesh.
//!
//! Ship-local frame (from the .obj): nose at z ≈ -3.1, tail at z ≈ +2.2, wings span x ≈ ±2.35
//! around z ∈ [-0.3, 0.7], main hull box x ∈ ±1, y ∈ [-0.17, 1.0], z ∈ [-1, 1].

use crate::math::{Quat, Vec3};
use crate::render::mesh::{icosphere, Mesh};
use crate::render::primitives::{append_transformed, box_mesh, cone_mesh, cylinder_mesh};
use raylib::color::Color;
use std::f32::consts::FRAC_PI_2;

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::new(r, g, b, 255)
}

const HULL: Color = Color { r: 228, g: 231, b: 240, a: 255 };
const HULL_DARK: Color = Color { r: 70, g: 76, b: 92, a: 255 };
const ACCENT: Color = Color { r: 232, g: 92, b: 40, a: 255 };
const GLASS: Color = Color { r: 55, g: 120, b: 200, a: 255 };
const METAL: Color = Color { r: 58, g: 62, b: 78, a: 255 };
const ENGINE_GLOW: Color = Color { r: 90, g: 235, b: 255, a: 255 };

/// Repaints the loaded hull per vertex: light hull with a dark underside, orange accent band and
/// nose tip, orange wing leading edges, and a blue-glass canopy area on top of the cabin. Applied
/// *before* subdivision so the colors interpolate smoothly across the new vertices.
pub fn paint_hull(mesh: &mut Mesh) {
    mesh.colors = mesh
        .positions
        .iter()
        .map(|p| {
            let is_wing = p.x.abs() > 1.1;
            if p.y < 0.12 {
                HULL_DARK // belly / skids
            } else if p.z < -2.4 {
                ACCENT // nose tip
            } else if is_wing && p.z < 0.1 {
                ACCENT // leading edges of the wings
            } else if is_wing {
                HULL
            } else if p.y > 0.85 && p.z > -1.05 && p.z < -0.2 {
                GLASS // canopy area on the cabin roof
            } else if (p.z - 0.45).abs() < 0.14 {
                ACCENT // racing band around the hull
            } else {
                HULL
            }
        })
        .collect();
}

/// Appends the procedural detail parts onto the (already subdivided) hull.
pub fn add_details(ship: &mut Mesh) {
    let along_z = Quat::from_axis_angle(Vec3::RIGHT, FRAC_PI_2); // cylinder axis +Y -> +Z (rear)

    // --- Twin engines on the tail, with glowing exhaust cores and a thin nozzle ring ---
    for side in [-1.0f32, 1.0] {
        let base = Vec3::new(side * 0.68, 0.42, 1.25);
        append_transformed(ship, &cylinder_mesh(0.3, 1.05, 14), base, along_z, 1.0, METAL);
        append_transformed(ship, &cylinder_mesh(0.34, 0.1, 14), base + Vec3::new(0.0, 0.0, 2.2 - 1.25 - 0.05), along_z, 1.0, ACCENT);
        append_transformed(ship, &cylinder_mesh(0.22, 0.16, 12), base + Vec3::new(0.0, 0.0, 1.0), along_z, 1.0, ENGINE_GLOW);
        append_transformed(ship, &icosphere(2), base + Vec3::new(0.0, 0.0, 1.18), Quat::IDENTITY, 0.2, col(200, 250, 255));
        // Pylon joining the engine to the hull.
        append_transformed(ship, &box_mesh(0.22, 0.08, 0.35), Vec3::new(side * 0.6, 0.45, 1.55), Quat::IDENTITY, 1.0, HULL_DARK);
    }

    // --- Cockpit dome ---
    append_transformed(ship, &icosphere(3), Vec3::new(0.0, 0.92, -0.62), Quat::IDENTITY, 0.5, GLASS);
    append_transformed(ship, &cylinder_mesh(0.5, 0.06, 16), Vec3::new(0.0, 0.9, -0.62), Quat::IDENTITY, 1.0, METAL);

    // --- Dorsal fin + sensor mast ---
    append_transformed(ship, &box_mesh(0.045, 0.38, 0.5), Vec3::new(0.0, 1.3, 0.75), Quat::from_axis_angle(Vec3::RIGHT, -0.35), 1.0, ACCENT);
    append_transformed(ship, &cylinder_mesh(0.025, 0.45, 6), Vec3::new(0.0, 1.0, 0.1), Quat::IDENTITY, 1.0, METAL);
    append_transformed(ship, &icosphere(1), Vec3::new(0.0, 1.5, 0.1), Quat::IDENTITY, 0.07, col(255, 80, 80));

    // --- Winglets and navigation lights at the wing tips (red port, green starboard) ---
    for side in [-1.0f32, 1.0] {
        append_transformed(
            ship,
            &box_mesh(0.04, 0.32, 0.42),
            Vec3::new(side * 2.25, 0.78, 0.2),
            Quat::from_axis_angle(Vec3::FORWARD, -side * 0.25),
            1.0,
            ACCENT,
        );
        let light = if side < 0.0 { col(255, 60, 60) } else { col(70, 255, 110) };
        append_transformed(ship, &icosphere(1), Vec3::new(side * 2.3, 0.52, -0.2), Quat::IDENTITY, 0.09, light);
    }

    // --- Nose: headlight + small sensor cone ---
    append_transformed(ship, &icosphere(1), Vec3::new(0.0, 0.41, -3.12), Quat::IDENTITY, 0.13, col(255, 240, 160));
    append_transformed(
        ship,
        &cone_mesh(0.08, 0.3, 8),
        Vec3::new(0.0, 0.41, -3.1),
        Quat::from_axis_angle(Vec3::RIGHT, -FRAC_PI_2),
        1.0,
        METAL,
    );

    // --- Side intakes on the cabin flanks and belly skids ---
    for side in [-1.0f32, 1.0] {
        append_transformed(ship, &cylinder_mesh(0.16, 0.9, 10), Vec3::new(side * 1.08, 0.55, -1.0), along_z, 1.0, HULL_DARK);
        append_transformed(ship, &cylinder_mesh(0.1, 0.05, 10), Vec3::new(side * 1.08, 0.55, -1.03), along_z, 1.0, ENGINE_GLOW);
        append_transformed(ship, &box_mesh(0.08, 0.06, 0.6), Vec3::new(side * 0.55, -0.2, 0.0), Quat::IDENTITY, 1.0, METAL);
    }
}
