//! The 20 exclusive surface-model types (4 per planet), built procedurally from primitives
//! (`render::primitives`) and the icosphere generator (`render::mesh::icosphere`) — no external
//! model files. Each function returns one finished, tinted `Mesh` ready to be scattered over a
//! planet by `scene::distribution::scatter`.
//!
//! Coordinates are in the host planet's *unit-sphere* space (y = outward from the surface), so a
//! prop 0.4 tall stands 0.4 × planet-radius world units above the ground. Sizes are deliberately
//! generous: the whole point of the props is to be readable from a fly-by, not just up close.

use crate::math::{Quat, Vec3};
use crate::render::mesh::{icosphere, Mesh};
use crate::render::primitives::{append_transformed, append_transformed_fn, box_mesh, cone_mesh, cylinder_mesh};
use raylib::color::Color;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::new(r, g, b, 255)
}

fn empty() -> Mesh {
    Mesh::default()
}

fn put(m: &mut Mesh, src: &Mesh, at: Vec3, scale: f32, color: Color) {
    append_transformed(m, src, at, Quat::IDENTITY, scale, color);
}

fn put_rot(m: &mut Mesh, src: &Mesh, at: Vec3, rot: Quat, scale: f32, color: Color) {
    append_transformed(m, src, at, rot, scale, color);
}

// ---------------------------------------------------------------------------------------------
// Mario Galaxy — "Reino Fungi Orbital"
// ---------------------------------------------------------------------------------------------

/// Giant mushroom: cream stem with eyes, big red cap with white spots.
pub fn mario_mushroom() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.065, 0.2, 10), Vec3::ZERO, 1.0, col(245, 238, 225));
    for side in [-1.0f32, 1.0] {
        put(&mut m, &box_mesh(0.012, 0.025, 0.006), Vec3::new(side * 0.025, 0.11, 0.063), 1.0, col(25, 25, 30));
    }
    let cap_center = Vec3::new(0.0, 0.2, 0.0);
    put(&mut m, &icosphere(2), cap_center, 0.17, col(220, 40, 40));
    let spot = icosphere(1);
    for i in 0..7 {
        let a = i as f32 * TAU / 7.0 + 0.4;
        let elev: f32 = if i % 2 == 0 { 0.55 } else { 0.95 };
        let dir = Vec3::new(a.cos() * (1.0 - elev * elev).sqrt(), elev, a.sin() * (1.0 - elev * elev).sqrt());
        put(&mut m, &spot, cap_center + dir * 0.165, 0.04, col(250, 248, 240));
    }
    m
}

/// Floating "?" block with dark question marks on every side.
pub fn mario_question_block() -> Mesh {
    let mut m = empty();
    let c = Vec3::new(0.0, 0.26, 0.0);
    put(&mut m, &box_mesh(0.1, 0.1, 0.1), c, 1.0, col(240, 185, 40));
    let mark = box_mesh(0.03, 0.045, 0.006);
    let dot = box_mesh(0.018, 0.015, 0.006);
    for (dx, dz, rot) in [(0.0, 1.0, 0.0), (0.0, -1.0, PI), (1.0, 0.0, FRAC_PI_2), (-1.0, 0.0, -FRAC_PI_2)] {
        let r = Quat::from_axis_angle(Vec3::UP, rot);
        put_rot(&mut m, &mark, c + Vec3::new(dx * 0.1, 0.02, dz * 0.1), r, 1.0, col(120, 70, 20));
        put_rot(&mut m, &dot, c + Vec3::new(dx * 0.1, -0.055, dz * 0.1), r, 1.0, col(120, 70, 20));
    }
    // Rivets on the corners.
    let rivet = icosphere(0);
    for sx in [-1.0f32, 1.0] {
        for sz in [-1.0f32, 1.0] {
            put(&mut m, &rivet, c + Vec3::new(sx * 0.085, 0.1, sz * 0.085), 0.012, col(150, 100, 30));
        }
    }
    m
}

/// Green warp pipe with a piranha plant peeking out.
pub fn mario_pipe() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.085, 0.22, 12), Vec3::ZERO, 1.0, col(40, 165, 70));
    put(&mut m, &cylinder_mesh(0.105, 0.055, 12), Vec3::new(0.0, 0.22, 0.0), 1.0, col(30, 130, 55));
    put(&mut m, &cylinder_mesh(0.07, 0.012, 12), Vec3::new(0.0, 0.275, 0.0), 1.0, col(10, 25, 15));
    // Piranha plant: green stalk, red spotted head, white lips.
    put(&mut m, &cylinder_mesh(0.018, 0.14, 6), Vec3::new(0.0, 0.28, 0.0), 1.0, col(60, 180, 70));
    let head = Vec3::new(0.0, 0.45, 0.0);
    put(&mut m, &icosphere(1), head, 0.07, col(225, 45, 45));
    for i in 0..4 {
        let a = i as f32 * TAU / 4.0 + 0.7;
        put(&mut m, &icosphere(0), head + Vec3::new(a.cos() * 0.055, 0.03, a.sin() * 0.055), 0.018, col(250, 250, 240));
    }
    put(&mut m, &box_mesh(0.065, 0.012, 0.03), head + Vec3::new(0.0, -0.03, 0.045), 1.0, col(250, 250, 245));
    m
}

/// Power star: five gold points around a core, floating above the ground, with two eyes.
pub fn mario_star() -> Mesh {
    let mut m = empty();
    let gold = col(252, 212, 45);
    let c = Vec3::new(0.0, 0.3, 0.0);
    put(&mut m, &icosphere(1), c, 0.065, gold);
    let point = cone_mesh(0.045, 0.13, 6);
    for i in 0..5 {
        let a = i as f32 * TAU / 5.0;
        // Cone axis is +Y; rotate it around the forward axis so the points fan out in a vertical
        // plane (the random per-instance yaw turns them to face different directions).
        let r = Quat::from_axis_angle(Vec3::FORWARD, a);
        let dir = r.rotate_vec3(Vec3::UP);
        put_rot(&mut m, &point, c + dir * 0.035, r, 1.0, gold);
    }
    for side in [-1.0f32, 1.0] {
        put(&mut m, &box_mesh(0.01, 0.022, 0.006), c + Vec3::new(side * 0.022, 0.005, 0.062), 1.0, col(25, 25, 30));
    }
    m
}

// ---------------------------------------------------------------------------------------------
// Guatemala futurista — "Tikal Prime"
// ---------------------------------------------------------------------------------------------

/// Stepped Tikal-style pyramid with a temple, glowing jade edge lights and a hologram antenna.
pub fn guatemala_pyramid() -> Mesh {
    let mut m = empty();
    let stone = col(130, 125, 105);
    let stone_dark = col(100, 98, 85);
    let jade = col(60, 230, 190);
    let tiers = [(0.24, 0.04), (0.19, 0.04), (0.14, 0.04), (0.09, 0.04)];
    let mut y = 0.0;
    for (i, (half, h)) in tiers.iter().enumerate() {
        y += h;
        put(&mut m, &box_mesh(*half, *h, *half), Vec3::new(0.0, y, 0.0), 1.0, if i % 2 == 0 { stone } else { stone_dark });
        // Glowing jade strip along the front edge of every tier.
        put(&mut m, &box_mesh(*half, 0.006, 0.006), Vec3::new(0.0, y + h, half + 0.002), 1.0, jade);
        y += h;
    }
    // Front staircase: a tilted slab from the ground to the temple platform.
    let stair_len = 0.32;
    put_rot(
        &mut m,
        &box_mesh(0.05, stair_len * 0.5, 0.012),
        Vec3::new(0.0, 0.16, 0.19),
        Quat::from_axis_angle(Vec3::RIGHT, -0.75),
        1.0,
        col(150, 145, 125),
    );
    // Temple on top + jade lintel.
    put(&mut m, &box_mesh(0.06, 0.06, 0.06), Vec3::new(0.0, y + 0.06, 0.0), 1.0, stone);
    put(&mut m, &box_mesh(0.065, 0.012, 0.065), Vec3::new(0.0, y + 0.125, 0.0), 1.0, jade);
    put(&mut m, &box_mesh(0.02, 0.035, 0.006), Vec3::new(0.0, y + 0.04, 0.062), 1.0, col(15, 30, 25));
    // Hologram antenna + projected orb.
    put(&mut m, &cylinder_mesh(0.012, 0.16, 6), Vec3::new(0.0, y + 0.13, 0.0), 1.0, col(70, 80, 90));
    put(&mut m, &icosphere(1), Vec3::new(0.0, y + 0.34, 0.0), 0.05, col(120, 255, 225));
    m
}

/// Jade stela on a plinth, with carved glyph blocks (some lit) and a lighter capstone.
pub fn guatemala_stela() -> Mesh {
    let mut m = empty();
    put(&mut m, &box_mesh(0.1, 0.02, 0.07), Vec3::new(0.0, 0.02, 0.0), 1.0, col(90, 95, 80));
    put(&mut m, &box_mesh(0.065, 0.26, 0.03), Vec3::new(0.0, 0.3, 0.0), 1.0, col(40, 135, 105));
    put(&mut m, &box_mesh(0.075, 0.03, 0.04), Vec3::new(0.0, 0.59, 0.0), 1.0, col(60, 165, 130));
    let glyph = box_mesh(0.02, 0.02, 0.01);
    for row in 0..5 {
        for colm in 0..2 {
            let lit = (row + colm) % 2 == 0;
            let x = -0.028 + colm as f32 * 0.056;
            let yy = 0.12 + row as f32 * 0.09;
            put(&mut m, &glyph, Vec3::new(x, yy, 0.03), 1.0, if lit { col(110, 255, 220) } else { col(25, 70, 60) });
        }
    }
    m
}

/// Mechanical quetzal perched on a tech mast: emerald body, red breast, long tail feathers.
pub fn guatemala_quetzal() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.014, 0.26, 6), Vec3::ZERO, 1.0, col(70, 75, 85));
    put(&mut m, &box_mesh(0.07, 0.008, 0.012), Vec3::new(0.0, 0.26, 0.0), 1.0, col(90, 95, 105));
    let body = Vec3::new(0.0, 0.32, 0.0);
    put(&mut m, &icosphere(1), body, 0.06, col(30, 170, 115));
    put(&mut m, &icosphere(1), body + Vec3::new(0.0, -0.01, 0.04), 0.04, col(215, 50, 60)); // breast
    let head = body + Vec3::new(0.0, 0.065, 0.03);
    put(&mut m, &icosphere(1), head, 0.038, col(40, 200, 140));
    put_rot(
        &mut m,
        &cone_mesh(0.014, 0.035, 5),
        head + Vec3::new(0.0, 0.0, 0.035),
        Quat::from_axis_angle(Vec3::RIGHT, FRAC_PI_2),
        1.0,
        col(240, 200, 60),
    );
    // Wings (slightly raised) and two long tail feathers hanging behind the mast.
    for side in [-1.0f32, 1.0] {
        put_rot(
            &mut m,
            &box_mesh(0.07, 0.008, 0.035),
            body + Vec3::new(side * 0.07, 0.02, -0.01),
            Quat::from_axis_angle(Vec3::FORWARD, side * 0.5),
            1.0,
            col(25, 150, 105),
        );
        put_rot(
            &mut m,
            &box_mesh(0.008, 0.17, 0.005),
            body + Vec3::new(side * 0.018, -0.16, -0.05),
            Quat::from_axis_angle(Vec3::RIGHT, 0.25),
            1.0,
            if side > 0.0 { col(40, 220, 170) } else { col(20, 180, 130) },
        );
    }
    m
}

/// Solar panel billboard on a mast, painted with a huipil-style striped and diamond pattern.
pub fn guatemala_textile_panel() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.016, 0.3, 6), Vec3::ZERO, 1.0, col(70, 75, 85));
    let center = Vec3::new(0.0, 0.33, 0.0);
    put(&mut m, &box_mesh(0.18, 0.12, 0.012), center, 1.0, col(25, 30, 45));
    let stripes = [
        col(205, 40, 70),
        col(240, 185, 40),
        col(130, 60, 170),
        col(40, 190, 160),
        col(240, 120, 40),
        col(205, 40, 70),
        col(240, 185, 40),
    ];
    for (i, c) in stripes.iter().enumerate() {
        let yy = -0.09 + i as f32 * 0.03;
        put(&mut m, &box_mesh(0.17, 0.011, 0.004), center + Vec3::new(0.0, yy, 0.014), 1.0, *c);
    }
    // Diamond motifs (rotated squares) over the stripes.
    let diamond = box_mesh(0.03, 0.03, 0.004);
    for i in 0..3 {
        let x = -0.1 + i as f32 * 0.1;
        put_rot(
            &mut m,
            &diamond,
            center + Vec3::new(x, 0.0, 0.02),
            Quat::from_axis_angle(Vec3::FORWARD, PI / 4.0),
            1.0,
            col(250, 245, 230),
        );
    }
    // Teal status light along the top edge.
    put(&mut m, &box_mesh(0.18, 0.006, 0.006), center + Vec3::new(0.0, 0.126, 0.0), 1.0, col(60, 230, 190));
    m
}

// ---------------------------------------------------------------------------------------------
// Pokémon — "Ruta Orbital"
// ---------------------------------------------------------------------------------------------

/// Big Poké Ball resting on the ground (red/black/white bands + a white button).
pub fn pokemon_pokeball() -> Mesh {
    let mut m = empty();
    let c = Vec3::new(0.0, 0.1, 0.0);
    append_transformed_fn(&mut m, &icosphere(3), c, Quat::IDENTITY, 0.13, |p| {
        if p.y > 0.09 {
            col(215, 35, 35)
        } else if p.y > -0.09 {
            col(25, 25, 28)
        } else {
            col(242, 242, 238)
        }
    });
    put(&mut m, &icosphere(1), c + Vec3::new(0.0, 0.0, 0.118), 0.04, col(25, 25, 28));
    put(&mut m, &icosphere(1), c + Vec3::new(0.0, 0.0, 0.13), 0.028, col(250, 250, 248));
    m
}

/// Berry tree: thick trunk, round layered canopy, red and blue berries.
pub fn pokemon_berry_tree() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.04, 0.2, 7), Vec3::ZERO, 1.0, col(115, 75, 42));
    let canopy = Vec3::new(0.0, 0.3, 0.0);
    put(&mut m, &icosphere(2), canopy, 0.14, col(70, 175, 70));
    for i in 0..3 {
        let a = i as f32 * TAU / 3.0;
        put(&mut m, &icosphere(1), canopy + Vec3::new(a.cos() * 0.1, -0.04, a.sin() * 0.1), 0.09, col(55, 150, 60));
    }
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0 + 0.3;
        let elev: f32 = if i % 2 == 0 { 0.2 } else { 0.55 };
        let dir = Vec3::new(a.cos() * (1.0 - elev * elev).sqrt(), elev, a.sin() * (1.0 - elev * elev).sqrt());
        let berry = if i % 3 == 0 { col(70, 110, 230) } else { col(225, 45, 65) };
        put(&mut m, &icosphere(0), canopy + dir * 0.135, 0.03, berry);
    }
    m
}

/// Evolution stone: a faceted fire-stone floating above a stone pedestal, with orbiting shards.
pub fn pokemon_evolution_stone() -> Mesh {
    let mut m = empty();
    put(&mut m, &cylinder_mesh(0.07, 0.05, 8), Vec3::ZERO, 1.0, col(120, 120, 125));
    put(&mut m, &cylinder_mesh(0.05, 0.03, 8), Vec3::new(0.0, 0.05, 0.0), 1.0, col(140, 140, 145));
    let c = Vec3::new(0.0, 0.22, 0.0);
    put(&mut m, &icosphere(0), c, 0.09, col(245, 150, 30));
    put(&mut m, &icosphere(0), c, 0.05, col(255, 220, 90)); // inner glow core peeks through facets
    for i in 0..3 {
        let a = i as f32 * TAU / 3.0;
        put(&mut m, &icosphere(0), c + Vec3::new(a.cos() * 0.13, 0.02 + (i as f32) * 0.02, a.sin() * 0.13), 0.025, col(250, 180, 60));
    }
    m
}

/// Pokémon Center: cream building, two-tier red roof, door, windows and a "P" sign.
pub fn pokemon_center() -> Mesh {
    let mut m = empty();
    put(&mut m, &box_mesh(0.17, 0.11, 0.14), Vec3::new(0.0, 0.11, 0.0), 1.0, col(248, 245, 235));
    put(&mut m, &box_mesh(0.19, 0.03, 0.16), Vec3::new(0.0, 0.25, 0.0), 1.0, col(215, 45, 45));
    put(&mut m, &box_mesh(0.12, 0.035, 0.1), Vec3::new(0.0, 0.315, 0.0), 1.0, col(215, 45, 45));
    put(&mut m, &box_mesh(0.045, 0.055, 0.006), Vec3::new(0.0, 0.055, 0.142), 1.0, col(60, 140, 220)); // glass door
    for side in [-1.0f32, 1.0] {
        put(&mut m, &box_mesh(0.035, 0.03, 0.006), Vec3::new(side * 0.1, 0.13, 0.142), 1.0, col(120, 190, 240));
    }
    // Roof sign with a pink "P".
    put(&mut m, &box_mesh(0.05, 0.05, 0.01), Vec3::new(0.0, 0.36, 0.1), 1.0, col(250, 250, 250));
    put(&mut m, &box_mesh(0.008, 0.035, 0.004), Vec3::new(-0.015, 0.36, 0.112), 1.0, col(240, 90, 160));
    put(&mut m, &box_mesh(0.02, 0.008, 0.004), Vec3::new(-0.003, 0.385, 0.112), 1.0, col(240, 90, 160));
    put(&mut m, &box_mesh(0.02, 0.008, 0.004), Vec3::new(-0.003, 0.36, 0.112), 1.0, col(240, 90, 160));
    put(&mut m, &box_mesh(0.008, 0.015, 0.004), Vec3::new(0.012, 0.372, 0.112), 1.0, col(240, 90, 160));
    m
}

// ---------------------------------------------------------------------------------------------
// Pac-Man — "Laberinto Neón"
// ---------------------------------------------------------------------------------------------

/// L-shaped maze wall segment with a bright neon rim on top.
pub fn pacman_wall() -> Mesh {
    let mut m = empty();
    let wall = col(18, 35, 125);
    let rim = col(80, 150, 255);
    put(&mut m, &box_mesh(0.2, 0.05, 0.035), Vec3::new(0.0, 0.05, 0.0), 1.0, wall);
    put(&mut m, &box_mesh(0.035, 0.05, 0.12), Vec3::new(0.165, 0.05, 0.085), 1.0, wall);
    put(&mut m, &box_mesh(0.2, 0.007, 0.035), Vec3::new(0.0, 0.105, 0.0), 1.0, rim);
    put(&mut m, &box_mesh(0.035, 0.007, 0.12), Vec3::new(0.165, 0.105, 0.085), 1.0, rim);
    m
}

/// Pac-Dot.
pub fn pacman_dot() -> Mesh {
    let mut m = empty();
    put(&mut m, &icosphere(1), Vec3::new(0.0, 0.04, 0.0), 0.035, col(252, 240, 200));
    m
}

/// Power pellet (large glowing dot).
pub fn pacman_power_pellet() -> Mesh {
    let mut m = empty();
    put(&mut m, &icosphere(2), Vec3::new(0.0, 0.08, 0.0), 0.07, col(255, 240, 160));
    m
}

/// Ghost: rounded head, cylindrical body, wavy skirt, big eyes with blue pupils.
pub fn pacman_ghost() -> Mesh {
    let mut m = empty();
    let ghost = col(255, 150, 45);
    put(&mut m, &cylinder_mesh(0.09, 0.13, 12), Vec3::new(0.0, 0.02, 0.0), 1.0, ghost);
    put(&mut m, &icosphere(2), Vec3::new(0.0, 0.15, 0.0), 0.09, ghost);
    for i in 0..6 {
        let a = i as f32 * TAU / 6.0;
        put(&mut m, &icosphere(1), Vec3::new(a.cos() * 0.065, 0.02, a.sin() * 0.065), 0.03, ghost);
    }
    for side in [-1.0f32, 1.0] {
        put(&mut m, &icosphere(1), Vec3::new(side * 0.038, 0.17, 0.07), 0.032, col(250, 250, 250));
        put(&mut m, &icosphere(0), Vec3::new(side * 0.03, 0.17, 0.096), 0.015, col(40, 60, 220));
    }
    m
}

// ---------------------------------------------------------------------------------------------
// Día de Muertos — "Mictlán Celestial"
// ---------------------------------------------------------------------------------------------

/// Monumental sugar skull: cranium + jaw, black sockets ringed with petals, teeth, forehead flower.
pub fn calavera_skull() -> Mesh {
    let mut m = empty();
    let bone = col(246, 242, 228);
    let c = Vec3::new(0.0, 0.14, 0.0);
    put(&mut m, &icosphere(2), c, 0.14, bone);
    put(&mut m, &box_mesh(0.085, 0.045, 0.075), Vec3::new(0.0, 0.045, 0.02), 1.0, bone);
    for side in [-1.0f32, 1.0] {
        let eye = c + Vec3::new(side * 0.055, 0.02, 0.115);
        put(&mut m, &icosphere(1), eye, 0.04, col(20, 18, 22));
        for i in 0..6 {
            let a = i as f32 * TAU / 6.0;
            let petal = if i % 2 == 0 { col(245, 140, 30) } else { col(225, 50, 150) };
            put(&mut m, &icosphere(0), eye + Vec3::new(a.cos() * 0.05, a.sin() * 0.05, 0.005), 0.014, petal);
        }
    }
    put_rot(
        &mut m,
        &cone_mesh(0.014, 0.03, 4),
        c + Vec3::new(0.0, -0.03, 0.13),
        Quat::from_axis_angle(Vec3::RIGHT, PI),
        1.0,
        col(20, 18, 22),
    );
    for i in 0..6 {
        let x = -0.0375 + i as f32 * 0.015;
        put(&mut m, &box_mesh(0.006, 0.014, 0.006), Vec3::new(x, 0.075, 0.1), 1.0, if i % 2 == 0 { bone } else { col(40, 40, 45) });
    }
    put(&mut m, &icosphere(1), c + Vec3::new(0.0, 0.11, 0.075), 0.035, col(230, 40, 150));
    put(&mut m, &icosphere(0), c + Vec3::new(0.0, 0.11, 0.105), 0.014, col(255, 220, 60));
    m
}

/// Alebrije: colorful spirit creature with horns, wings, legs, tail and painted spots.
pub fn alebrije() -> Mesh {
    let mut m = empty();
    let body_c = Vec3::new(0.0, 0.16, 0.0);
    put(&mut m, &icosphere(2), body_c, 0.1, col(40, 175, 175));
    for i in 0..6 {
        let a = i as f32 * TAU / 6.0 + 0.5;
        let elev: f32 = if i % 2 == 0 { 0.3 } else { -0.1 };
        let dir = Vec3::new(a.cos() * (1.0 - elev * elev).sqrt(), elev, a.sin() * (1.0 - elev * elev).sqrt());
        put(&mut m, &icosphere(0), body_c + dir * 0.097, 0.022, col(250, 210, 50));
    }
    let head = Vec3::new(0.0, 0.25, 0.1);
    put(&mut m, &icosphere(1), head, 0.06, col(150, 60, 190));
    for side in [-1.0f32, 1.0] {
        put_rot(
            &mut m,
            &cone_mesh(0.016, 0.08, 5),
            head + Vec3::new(side * 0.035, 0.04, -0.01),
            Quat::from_axis_angle(Vec3::FORWARD, -side * 0.5),
            1.0,
            col(255, 200, 40),
        );
        put(&mut m, &icosphere(0), head + Vec3::new(side * 0.03, 0.01, 0.05), 0.014, col(255, 255, 255));
        put(&mut m, &cylinder_mesh(0.018, 0.1, 6), Vec3::new(side * 0.06, 0.0, 0.05), 1.0, col(240, 140, 30));
        put(&mut m, &cylinder_mesh(0.018, 0.1, 6), Vec3::new(side * 0.06, 0.0, -0.05), 1.0, col(240, 140, 30));
        put_rot(
            &mut m,
            &box_mesh(0.11, 0.012, 0.05),
            body_c + Vec3::new(side * 0.12, 0.07, -0.02),
            Quat::from_axis_angle(Vec3::FORWARD, side * 0.7),
            1.0,
            col(225, 60, 160),
        );
    }
    put_rot(
        &mut m,
        &cone_mesh(0.03, 0.14, 6),
        body_c + Vec3::new(0.0, -0.02, -0.09),
        Quat::from_axis_angle(Vec3::RIGHT, -2.2),
        1.0,
        col(250, 120, 40),
    );
    m
}

/// Ofrenda: three-tier altar with candles, marigolds, a framed photo and pan de muerto.
pub fn ofrenda_velas() -> Mesh {
    let mut m = empty();
    let purple = col(120, 45, 150);
    let orange = col(240, 125, 35);
    put(&mut m, &box_mesh(0.2, 0.035, 0.12), Vec3::new(0.0, 0.035, 0.0), 1.0, purple);
    put(&mut m, &box_mesh(0.15, 0.035, 0.085), Vec3::new(0.0, 0.105, -0.02), 1.0, orange);
    put(&mut m, &box_mesh(0.1, 0.035, 0.055), Vec3::new(0.0, 0.175, -0.04), 1.0, purple);
    // Candles on the lowest tier.
    for (x, z) in [(-0.16f32, 0.07f32), (0.16, 0.07), (-0.16, -0.06), (0.16, -0.06)] {
        let base = Vec3::new(x, 0.07, z);
        put(&mut m, &cylinder_mesh(0.016, 0.09, 7), base, 1.0, col(250, 246, 232));
        put(&mut m, &icosphere(1), base + Vec3::new(0.0, 0.105, 0.0), 0.02, col(255, 190, 50));
    }
    // Marigolds along the middle tier.
    for i in 0..5 {
        let x = -0.12 + i as f32 * 0.06;
        put(&mut m, &icosphere(1), Vec3::new(x, 0.165, 0.055), 0.03, if i % 2 == 0 { col(245, 140, 20) } else { col(255, 200, 40) });
    }
    // Framed photo + pan de muerto on top.
    put(&mut m, &box_mesh(0.045, 0.055, 0.008), Vec3::new(0.0, 0.265, -0.04), 1.0, col(220, 180, 60));
    put(&mut m, &box_mesh(0.032, 0.04, 0.004), Vec3::new(0.0, 0.265, -0.03), 1.0, col(90, 70, 60));
    put(&mut m, &icosphere(1), Vec3::new(0.07, 0.235, -0.03), 0.028, col(190, 130, 70));
    put(&mut m, &icosphere(1), Vec3::new(-0.07, 0.235, -0.03), 0.028, col(190, 130, 70));
    m
}

/// Cempasúchil arch: two wooden posts joined by an arc of marigolds, with flowers at the base.
pub fn cempasuchil_arch() -> Mesh {
    let mut m = empty();
    let wood = col(95, 60, 35);
    for side in [-1.0f32, 1.0] {
        put(&mut m, &cylinder_mesh(0.022, 0.3, 7), Vec3::new(side * 0.2, 0.0, 0.0), 1.0, wood);
        for i in 0..3 {
            let a = i as f32 * TAU / 3.0;
            put(&mut m, &icosphere(1), Vec3::new(side * 0.2 + a.cos() * 0.04, 0.03, a.sin() * 0.04), 0.03, col(245, 140, 20));
        }
    }
    let flower = icosphere(1);
    let n = 11;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let a = PI * (1.0 - t);
        let p = Vec3::new(a.cos() * 0.2, 0.3 + a.sin() * 0.2, 0.0);
        let c = match i % 3 {
            0 => col(245, 135, 20),
            1 => col(255, 200, 40),
            _ => col(235, 100, 25),
        };
        put(&mut m, &flower, p, 0.04, c);
    }
    // Hanging papel picado flags under the arch.
    for i in 0..4 {
        let x = -0.12 + i as f32 * 0.08;
        let c = if i % 2 == 0 { col(225, 50, 150) } else { col(60, 190, 200) };
        put(&mut m, &box_mesh(0.025, 0.03, 0.003), Vec3::new(x, 0.42 - (x * x) * 1.6, 0.0), 1.0, c);
    }
    m
}
