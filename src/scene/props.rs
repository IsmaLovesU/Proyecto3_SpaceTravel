//! The 20 exclusive surface-model types (4 per planet), built procedurally from primitives
//! (`render::primitives`) and the icosphere generator (`render::mesh::icosphere`) — no external
//! model files. Each function returns one finished, tinted `Mesh` ready to be scattered over a
//! planet by `scene::distribution::scatter`.

use crate::math::{Quat, Vec3};
use crate::render::mesh::{icosphere, Mesh};
use crate::render::primitives::{append_transformed, append_transformed_fn, box_mesh, cone_mesh, cylinder_mesh};
use raylib::color::Color;

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::new(r, g, b, 255)
}

fn empty() -> Mesh {
    Mesh::default()
}

// ---------------------------------------------------------------------------------------------
// Mario Galaxy — "Reino Fungi Orbital"
// ---------------------------------------------------------------------------------------------

/// Giant mushroom: red cap on a white stem.
pub fn mario_mushroom() -> Mesh {
    let mut m = empty();
    let stem = cylinder_mesh(0.035, 0.12, 8);
    append_transformed(&mut m, &stem, Vec3::ZERO, Quat::IDENTITY, 1.0, col(245, 240, 230));
    let cap = icosphere(1);
    append_transformed(&mut m, &cap, Vec3::new(0.0, 0.12, 0.0), Quat::IDENTITY, 0.09, col(215, 40, 40));
    m
}

/// Floating "?" block.
pub fn mario_question_block() -> Mesh {
    let mut m = empty();
    let block = box_mesh(0.05, 0.05, 0.05);
    append_transformed(&mut m, &block, Vec3::new(0.0, 0.11, 0.0), Quat::IDENTITY, 1.0, col(235, 180, 40));
    m
}

/// Green warp pipe sticking out of the surface.
pub fn mario_pipe() -> Mesh {
    let mut m = empty();
    let stem = cylinder_mesh(0.045, 0.14, 10);
    append_transformed(&mut m, &stem, Vec3::ZERO, Quat::IDENTITY, 1.0, col(40, 160, 70));
    let lip = cylinder_mesh(0.06, 0.025, 10);
    append_transformed(&mut m, &lip, Vec3::new(0.0, 0.14, 0.0), Quat::IDENTITY, 1.0, col(30, 125, 55));
    m
}

/// Power star: a small gold bipyramid (two cones base-to-base).
pub fn mario_star() -> Mesh {
    let mut m = empty();
    let half = cone_mesh(0.05, 0.06, 5);
    append_transformed(&mut m, &half, Vec3::new(0.0, 0.09, 0.0), Quat::IDENTITY, 1.0, col(250, 210, 40));
    append_transformed(
        &mut m,
        &half,
        Vec3::new(0.0, 0.09, 0.0),
        Quat::from_axis_angle(Vec3::RIGHT, std::f32::consts::PI),
        1.0,
        col(250, 210, 40),
    );
    m
}

// ---------------------------------------------------------------------------------------------
// Guatemala futurista — "Tikal Prime"
// ---------------------------------------------------------------------------------------------

/// Stepped pyramid with a holographic projector antenna on top.
pub fn guatemala_pyramid() -> Mesh {
    let mut m = empty();
    let stone = col(95, 100, 80);
    append_transformed(&mut m, &box_mesh(0.14, 0.03, 0.14), Vec3::new(0.0, 0.03, 0.0), Quat::IDENTITY, 1.0, stone);
    append_transformed(&mut m, &box_mesh(0.10, 0.03, 0.10), Vec3::new(0.0, 0.09, 0.0), Quat::IDENTITY, 1.0, stone);
    append_transformed(&mut m, &box_mesh(0.06, 0.03, 0.06), Vec3::new(0.0, 0.15, 0.0), Quat::IDENTITY, 1.0, stone);
    let antenna = cylinder_mesh(0.008, 0.08, 6);
    append_transformed(&mut m, &antenna, Vec3::new(0.0, 0.18, 0.0), Quat::IDENTITY, 1.0, col(60, 220, 190));
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.26, 0.0), Quat::IDENTITY, 0.02, col(80, 240, 210));
    m
}

/// Jade stela with glowing carved glyphs (the grid shader does the "glow" part).
pub fn guatemala_stela() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &box_mesh(0.03, 0.15, 0.015), Vec3::new(0.0, 0.15, 0.0), Quat::IDENTITY, 1.0, col(40, 130, 100));
    append_transformed(&mut m, &box_mesh(0.035, 0.02, 0.02), Vec3::new(0.0, 0.31, 0.0), Quat::IDENTITY, 1.0, col(50, 150, 115));
    m
}

/// Small mechanical quetzal perched on the surface.
pub fn guatemala_quetzal() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.05, 0.0), Quat::IDENTITY, 0.035, col(30, 160, 110));
    append_transformed(
        &mut m,
        &box_mesh(0.01, 0.01, 0.08),
        Vec3::new(0.0, 0.08, -0.06),
        Quat::from_axis_angle(Vec3::RIGHT, -0.5),
        1.0,
        col(20, 190, 140),
    );
    append_transformed(
        &mut m,
        &cone_mesh(0.012, 0.025, 4),
        Vec3::new(0.0, 0.055, 0.035),
        Quat::from_axis_angle(Vec3::RIGHT, std::f32::consts::FRAC_PI_2),
        1.0,
        col(230, 190, 60),
    );
    m
}

/// A small solar panel decorated with a huipil-style striped pattern.
pub fn guatemala_textile_panel() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &box_mesh(0.1, 0.01, 0.07), Vec3::new(0.0, 0.015, 0.0), Quat::IDENTITY, 1.0, col(30, 35, 40));
    let stripe_colors = [col(200, 40, 60), col(230, 180, 40), col(120, 50, 160)];
    for (i, c) in stripe_colors.iter().enumerate() {
        let x = -0.07 + i as f32 * 0.07;
        append_transformed(&mut m, &box_mesh(0.02, 0.003, 0.06), Vec3::new(x, 0.028, 0.0), Quat::IDENTITY, 1.0, *c);
    }
    m
}

// ---------------------------------------------------------------------------------------------
// Pokémon — "Ruta Orbital"
// ---------------------------------------------------------------------------------------------

/// Half-buried Poké Ball (red/white/black bands from one icosphere, tinted per-vertex by local Y).
pub fn pokemon_pokeball() -> Mesh {
    let mut m = empty();
    append_transformed_fn(&mut m, &icosphere(2), Vec3::ZERO, Quat::IDENTITY, 0.06, |p| {
        if p.y > 0.12 {
            col(210, 35, 35)
        } else if p.y > -0.12 {
            col(25, 25, 25)
        } else {
            col(240, 240, 235)
        }
    });
    m
}

/// Stylized berry tree: green canopy with a few red berries, on a brown trunk.
pub fn pokemon_berry_tree() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &cylinder_mesh(0.02, 0.1, 6), Vec3::ZERO, Quat::IDENTITY, 1.0, col(110, 70, 40));
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.14, 0.0), Quat::IDENTITY, 0.06, col(70, 170, 70));
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::TAU / 4.0;
        let offset = Vec3::new(angle.cos() * 0.05, 0.13, angle.sin() * 0.05);
        append_transformed(&mut m, &icosphere(0), offset, Quat::IDENTITY, 0.015, col(220, 40, 60));
    }
    m
}

/// Floating evolution stone (an unsubdivided icosphere looks naturally faceted, like a gem).
pub fn pokemon_evolution_stone() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(0), Vec3::new(0.0, 0.08, 0.0), Quat::IDENTITY, 0.045, col(240, 150, 30));
    m
}

/// Tiny Pokémon Center: white dome body, red conical roof.
pub fn pokemon_center() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &box_mesh(0.09, 0.06, 0.09), Vec3::new(0.0, 0.06, 0.0), Quat::IDENTITY, 1.0, col(245, 245, 240));
    append_transformed(&mut m, &cone_mesh(0.11, 0.05, 4), Vec3::new(0.0, 0.12, 0.0), Quat::IDENTITY, 1.0, col(210, 40, 40));
    m
}

// ---------------------------------------------------------------------------------------------
// Pac-Man — "Laberinto Neón"
// ---------------------------------------------------------------------------------------------

/// Maze wall segment (the neon grid shader lights up its edges).
pub fn pacman_wall() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &box_mesh(0.09, 0.03, 0.015), Vec3::new(0.0, 0.03, 0.0), Quat::IDENTITY, 1.0, col(15, 30, 110));
    m
}

/// Small Pac-Dot.
pub fn pacman_dot() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(0), Vec3::new(0.0, 0.02, 0.0), Quat::IDENTITY, 0.012, col(250, 240, 200));
    m
}

/// Larger power pellet.
pub fn pacman_power_pellet() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.035, 0.0), Quat::IDENTITY, 0.03, col(255, 235, 150));
    m
}

/// Decorative ghost: capsule body + rounded head + a dark eye band.
pub fn pacman_ghost() -> Mesh {
    let mut m = empty();
    let ghost_color = col(255, 150, 40);
    append_transformed(&mut m, &cylinder_mesh(0.045, 0.07, 8), Vec3::ZERO, Quat::IDENTITY, 1.0, ghost_color);
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.07, 0.0), Quat::IDENTITY, 0.045, ghost_color);
    append_transformed(&mut m, &box_mesh(0.04, 0.012, 0.005), Vec3::new(0.0, 0.075, 0.043), Quat::IDENTITY, 1.0, col(20, 20, 25));
    m
}

// ---------------------------------------------------------------------------------------------
// Día de Muertos — "Mictlán Celestial"
// ---------------------------------------------------------------------------------------------

/// Monumental sugar skull: white base, black eye sockets, a colorful forehead accent.
pub fn calavera_skull() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(2), Vec3::new(0.0, 0.06, 0.0), Quat::IDENTITY, 0.06, col(245, 240, 225));
    for side in [-1.0f32, 1.0] {
        append_transformed(&mut m, &icosphere(0), Vec3::new(side * 0.025, 0.065, 0.05), Quat::IDENTITY, 0.012, col(20, 20, 20));
    }
    append_transformed(&mut m, &icosphere(0), Vec3::new(0.0, 0.11, 0.045), Quat::IDENTITY, 0.015, col(220, 30, 140));
    m
}

/// Colorful alebrije spirit-animal statue: body, two wings, a tail.
pub fn alebrije() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &icosphere(1), Vec3::new(0.0, 0.055, 0.0), Quat::IDENTITY, 0.05, col(40, 170, 170));
    for side in [-1.0f32, 1.0] {
        append_transformed(
            &mut m,
            &box_mesh(0.045, 0.008, 0.02),
            Vec3::new(side * 0.06, 0.08, 0.0),
            Quat::from_axis_angle(Vec3::FORWARD, side * 0.6),
            1.0,
            col(240, 140, 30),
        );
    }
    append_transformed(
        &mut m,
        &cone_mesh(0.02, 0.07, 5),
        Vec3::new(0.0, 0.05, 0.045),
        Quat::from_axis_angle(Vec3::RIGHT, -2.0),
        1.0,
        col(220, 60, 160),
    );
    m
}

/// Offering altar with two lit candles.
pub fn ofrenda_velas() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &box_mesh(0.08, 0.015, 0.05), Vec3::new(0.0, 0.015, 0.0), Quat::IDENTITY, 1.0, col(120, 70, 40));
    for side in [-1.0f32, 1.0] {
        let base = Vec3::new(side * 0.035, 0.03, 0.0);
        append_transformed(&mut m, &cylinder_mesh(0.008, 0.05, 6), base, Quat::IDENTITY, 1.0, col(250, 245, 230));
        append_transformed(&mut m, &icosphere(0), base + Vec3::new(0.0, 0.055, 0.0), Quat::IDENTITY, 0.012, col(255, 170, 40));
    }
    m
}

/// A clustered marigold (cempasúchil) bouquet on a green stem.
pub fn cempasuchil_arch() -> Mesh {
    let mut m = empty();
    append_transformed(&mut m, &cylinder_mesh(0.01, 0.12, 6), Vec3::ZERO, Quat::IDENTITY, 1.0, col(60, 130, 50));
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        let offset = Vec3::new(angle.cos() * 0.035, 0.12 + (i % 2) as f32 * 0.015, angle.sin() * 0.035);
        append_transformed(&mut m, &icosphere(0), offset, Quat::IDENTITY, 0.025, col(240, 140, 20));
    }
    m
}
