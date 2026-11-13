use super::body::{CelestialBody, OrbitParams};
use super::distribution::{scatter, ScatterSpec};
use super::props;
use super::ship::{self, Ship};
use super::ship_style;
use crate::math::{fbm, Quat, Vec3};
use crate::render::deform::{deform_by_height, mictlan_terrain_height};
use crate::render::mesh::{icosphere_lod_levels, Mesh};
use crate::render::normal_map::NormalMap;
use crate::render::obj_loader::load_obj;
use crate::render::shader::{
    BumpMappedShader, EmissiveCoronaShader, FragmentShader, JadeCircuitShader, LambertShader, NeonGridShader,
};
use crate::render::subdivision::subdivide_mesh;
use raylib::color::Color;

pub struct SolarSystem {
    pub sun: CelestialBody,
    pub planets: Vec<CelestialBody>,
    pub ship: Ship,
    pub sim_time: f64,
    pub surface_seed: u64,
}

/// 5 LOD tiers: 20 / 80 / 320 / 1280 / 5120 triangles. The finest tier only kicks in close to a
/// planet, where the bigger radii would otherwise show a visibly faceted horizon.
const LOD_LEVELS: u32 = 5;
const SHIP_OBJ_PATH: &str = "assets/models/ship.obj";
const DEFAULT_SURFACE_SEED: u64 = 1;
/// Per-instance size variation applied on top of the props' own (already generous) dimensions.
const PROP_SCALE_RANGE: (f32, f32) = (0.55, 0.75);

/// One planet's full theme: visual identity, shader, and its 4 exclusive surface-model types with
/// how many of each to scatter.
struct PlanetTheme {
    name: &'static str,
    radius: f32,
    orbit_radius: f64,
    orbit_speed: f64,
    spin_speed: f64,
    /// Per-vertex surface coloring (direction on the unit sphere -> color). Lets each planet
    /// carry biomes/patches baked into the mesh with zero per-frame cost.
    surface_color: fn(Vec3) -> Color,
    shader: Box<dyn FragmentShader>,
    prop_shader: Box<dyn FragmentShader>,
    prop_builders: [fn() -> Mesh; 4],
    prop_counts: [u32; 4],
    min_separation: f32,
    /// When set, the planet's sphere is displaced by this noise height function + amplitude
    /// (`render::deform::deform_by_height`), and surface props are offset to match so they sit on
    /// the deformed terrain instead of floating/sinking.
    terrain_height: Option<(fn(Vec3) -> f32, f32)>,
}

impl SolarSystem {
    pub fn new() -> Self {
        let sun = CelestialBody {
            name: "Sol".to_string(),
            mesh_lods: colored_lods(sun_surface_color, None),
            radius: 34.0,
            mass: 2600.0,
            orbit: None,
            spin_speed: 0.03,
            shader: Box::new(EmissiveCoronaShader::default()),
            prop_shader: Box::new(LambertShader::default()),
            surface_bulge: 0.0,
            prop_meshes: Vec::new(),
            surface_instances: Vec::new(),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        };

        let themes = planet_themes();
        let planets = themes
            .into_iter()
            .enumerate()
            .map(|(i, theme)| build_planet(theme, i as f64 * 1.3, DEFAULT_SURFACE_SEED))
            .collect();

        let ship_mesh = match load_obj(SHIP_OBJ_PATH) {
            Ok(mesh) => mesh,
            Err(_) => {
                eprintln!(
                    "WARN: no se encontro '{}', usando nave de respaldo (placeholder). Coloca el .obj ahi para usar la nave real.",
                    SHIP_OBJ_PATH
                );
                ship::placeholder_wedge_mesh()
            }
        };
        // Repaint the hull, then the procedural subdivision + Laplacian smoothing pass (rounds off
        // the low-poly silhouette), then bolt on the procedural detail parts (engines, canopy,
        // fins, lights) -- see `scene::ship_style`.
        let mut ship_mesh = ship_mesh;
        ship_style::paint_hull(&mut ship_mesh);
        let mut ship_mesh = subdivide_mesh(&ship_mesh, 1, 0.5);
        ship_style::add_details(&mut ship_mesh);

        let ship = Ship {
            // Start just outside the innermost orbit, looking in toward the sun, so the first
            // thing on screen is the sun with planets strung out around it.
            position: Vec3::new(-70.0, 22.0, 165.0),
            orientation: Quat::from_to_rotation(Vec3::FORWARD, Vec3::new(70.0, -14.0, -165.0).normalized()),
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mesh: ship_mesh,
            mesh_scale: 1.0,
            collision_radius: 2.4,
        };

        SolarSystem { sun, planets, ship, sim_time: 0.0, surface_seed: DEFAULT_SURFACE_SEED }
    }

    pub fn update_bodies(&mut self) {
        self.sun.update(self.sim_time);
        for p in &mut self.planets {
            p.update(self.sim_time);
        }
    }

    pub fn bodies(&self) -> impl Iterator<Item = &CelestialBody> {
        std::iter::once(&self.sun).chain(self.planets.iter())
    }

    /// Re-scatters every planet's surface instances with a new seed, in place (same prop meshes,
    /// just new positions) — this is what the "R" key drives at runtime, to demo that the
    /// distribution is reproducible and seed-driven, not hand-placed.
    pub fn regenerate_surfaces(&mut self, seed: u64) {
        self.surface_seed = seed;
        let themes = planet_themes();
        for (planet, theme) in self.planets.iter_mut().zip(themes.into_iter()) {
            let spec = ScatterSpec {
                counts: theme.prop_counts.to_vec(),
                min_separation: theme.min_separation,
                scale_range: PROP_SCALE_RANGE,
                surface_height: theme.terrain_height,
            };
            planet.surface_instances = scatter(seed, &spec);
        }
    }
}

/// Builds the LOD chain for a body, colors every vertex with `surface_color` (evaluated on the
/// unit-sphere direction, so all LOD tiers agree) and optionally displaces it with a terrain
/// height function. Color is evaluated *after* deformation so a height-aware palette (Mictlán's
/// valleys vs. peaks) can read the final shape.
fn colored_lods(surface_color: fn(Vec3) -> Color, terrain: Option<(fn(Vec3) -> f32, f32)>) -> Vec<Mesh> {
    icosphere_lod_levels(LOD_LEVELS)
        .into_iter()
        .map(|mut m| {
            if let Some((height_fn, amplitude)) = terrain {
                deform_by_height(&mut m, height_fn, amplitude);
            }
            m.colors = m.positions.iter().map(|p| surface_color(p.normalized())).collect();
            m
        })
        .collect()
}

fn build_planet(theme: PlanetTheme, phase: f64, seed: u64) -> CelestialBody {
    let spec = ScatterSpec {
        counts: theme.prop_counts.to_vec(),
        min_separation: theme.min_separation,
        scale_range: PROP_SCALE_RANGE,
        surface_height: theme.terrain_height,
    };
    let surface_instances = scatter(seed, &spec);
    let prop_meshes: Vec<Mesh> = theme.prop_builders.iter().map(|f| f()).collect();
    let mesh_lods = colored_lods(theme.surface_color, theme.terrain_height);

    CelestialBody {
        name: theme.name.to_string(),
        mesh_lods,
        radius: theme.radius,
        mass: theme.radius * 22.0,
        orbit: Some(OrbitParams { radius: theme.orbit_radius, speed: theme.orbit_speed, phase }),
        spin_speed: theme.spin_speed,
        shader: theme.shader,
        prop_shader: theme.prop_shader,
        surface_bulge: theme.terrain_height.map(|(_, amp)| amp).unwrap_or(0.0),
        prop_meshes,
        surface_instances,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    }
}

// ---------------------------------------------------------------------------------------------
// Per-planet surface palettes (baked per vertex, read through the planet's shader at runtime)
// ---------------------------------------------------------------------------------------------

fn c(r: u8, g: u8, b: u8) -> Color {
    Color::new(r, g, b, 255)
}

fn sun_surface_color(p: Vec3) -> Color {
    // Hot yellow-white with slightly darker "granulation" patches.
    let n = fbm(p * 4.0, 3, 21.0);
    if n > 0.25 { c(255, 235, 140) } else if n < -0.3 { c(250, 165, 40) } else { c(255, 205, 70) }
}

/// Mario Galaxy: bright cartoon grass with brown dirt patches and little blue ponds.
fn mario_surface_color(p: Vec3) -> Color {
    let n = fbm(p * 3.0, 3, 11.0);
    let n2 = fbm(p * 7.0, 2, 12.0);
    if n < -0.42 {
        c(70, 140, 235)
    } else if n < -0.22 {
        c(150, 105, 60)
    } else if n2 > 0.3 {
        c(140, 220, 120)
    } else {
        c(95, 200, 110)
    }
}

/// Tikal Prime: dark jade stone with lighter weathered plates (the circuit grid glows on top).
fn guatemala_surface_color(p: Vec3) -> Color {
    let n = fbm(p * 3.5, 3, 31.0);
    if n > 0.22 { c(85, 110, 90) } else if n < -0.3 { c(35, 50, 45) } else { c(55, 75, 62) }
}

/// Ruta Orbital: grassland with winding dirt routes, lakes and darker forest patches.
fn pokemon_surface_color(p: Vec3) -> Color {
    let n = fbm(p * 2.6, 4, 41.0);
    let route = fbm(p * 5.0, 2, 42.0);
    if n < -0.38 {
        c(55, 125, 220)
    } else if n < -0.3 {
        c(225, 205, 140) // beach rim around lakes
    } else if route.abs() < 0.05 {
        c(200, 170, 110) // dirt route
    } else if n > 0.4 {
        c(50, 130, 70) // forest
    } else {
        c(125, 205, 110)
    }
}

/// Laberinto Neón: near-black arcade base with slightly lighter "circuit" plates.
fn pacman_surface_color(p: Vec3) -> Color {
    let n = fbm(p * 4.0, 2, 51.0);
    if n > 0.3 { c(16, 18, 48) } else { c(8, 8, 26) }
}

/// Mictlán Celestial: height-coded -- deep violet valleys, ochre slopes, marigold-colored peaks.
fn mictlan_surface_color(p: Vec3) -> Color {
    let h = mictlan_terrain_height(p);
    let speck = fbm(p * 9.0, 2, 61.0);
    if h < -0.28 {
        c(75, 30, 95)
    } else if h < -0.05 {
        c(120, 55, 95)
    } else if h < 0.3 {
        if speck > 0.35 { c(225, 150, 60) } else { c(200, 115, 50) }
    } else {
        c(245, 175, 45)
    }
}

/// The 5 themes, in a fixed order matching `SolarSystem.planets`. A free function (not a const)
/// because `Box<dyn FragmentShader>` needs to be constructed fresh each call (e.g. when `R`
/// re-scatters surfaces, we rebuild the theme list but reuse the planet's already-built shader).
///
/// Orbits are widely spaced and slow (periods of 2-8 minutes) so each planet reads as its own
/// destination and the motion is appreciable without planets sliding away mid-approach.
fn planet_themes() -> Vec<PlanetTheme> {
    vec![
        // 1. Mario Galaxy -- "Reino Fungi Orbital"
        PlanetTheme {
            name: "Reino Fungi Orbital",
            radius: 9.0,
            orbit_radius: 130.0,
            orbit_speed: 0.045,
            spin_speed: 0.22,
            surface_color: mario_surface_color,
            shader: Box::new(LambertShader { ambient: 0.32 }),
            prop_shader: Box::new(LambertShader { ambient: 0.35 }),
            prop_builders: [props::mario_mushroom, props::mario_question_block, props::mario_pipe, props::mario_star],
            prop_counts: [12, 12, 9, 16],
            min_separation: 0.24,
            terrain_height: None,
        },
        // 2. Guatemala futurista -- "Tikal Prime"
        PlanetTheme {
            name: "Tikal Prime",
            radius: 11.0,
            orbit_radius: 215.0,
            orbit_speed: 0.032,
            spin_speed: 0.26,
            surface_color: guatemala_surface_color,
            shader: Box::new(JadeCircuitShader::default()),
            prop_shader: Box::new(LambertShader { ambient: 0.35 }),
            prop_builders: [
                props::guatemala_pyramid,
                props::guatemala_stela,
                props::guatemala_quetzal,
                props::guatemala_textile_panel,
            ],
            prop_counts: [8, 12, 9, 10],
            min_separation: 0.24,
            terrain_height: None,
        },
        // 3. Pokémon -- "Ruta Orbital"
        PlanetTheme {
            name: "Ruta Orbital",
            radius: 8.5,
            orbit_radius: 305.0,
            orbit_speed: 0.024,
            spin_speed: 0.3,
            surface_color: pokemon_surface_color,
            // Procedural tangent-space normal map (own noise, no image file) gives the sphere a
            // bumpy grass/dirt texture -- the "mapeo de normales" requirement.
            shader: Box::new(BumpMappedShader {
                ambient: 0.3,
                normal_map: NormalMap::generate(64, 64, 10.0, 3.0, 3.0),
            }),
            prop_shader: Box::new(LambertShader { ambient: 0.35 }),
            prop_builders: [
                props::pokemon_pokeball,
                props::pokemon_berry_tree,
                props::pokemon_evolution_stone,
                props::pokemon_center,
            ],
            prop_counts: [14, 14, 9, 6],
            min_separation: 0.22,
            terrain_height: None,
        },
        // 4. Pac-Man -- "Laberinto Neón"
        PlanetTheme {
            name: "Laberinto Neon",
            radius: 12.0,
            orbit_radius: 405.0,
            orbit_speed: 0.018,
            spin_speed: 0.18,
            surface_color: pacman_surface_color,
            shader: Box::new(NeonGridShader::default()),
            // Props keep a coarser neon grid so the maze walls get glowing brick edges while
            // dots/ghosts stay mostly flat-lit.
            prop_shader: Box::new(NeonGridShader {
                ambient: 0.5,
                grid_frequency: 3.0,
                line_width: 0.08,
                glow_color: (60.0, 140.0, 255.0),
                pulse_speed: 2.4,
            }),
            prop_builders: [props::pacman_wall, props::pacman_dot, props::pacman_power_pellet, props::pacman_ghost],
            prop_counts: [40, 50, 12, 9],
            min_separation: 0.15,
            terrain_height: None,
        },
        // 5. Día de Muertos -- "Mictlán Celestial"
        PlanetTheme {
            name: "Mictlan Celestial",
            radius: 10.0,
            orbit_radius: 515.0,
            orbit_speed: 0.013,
            spin_speed: 0.24,
            surface_color: mictlan_surface_color,
            shader: Box::new(LambertShader { ambient: 0.3 }),
            prop_shader: Box::new(LambertShader { ambient: 0.38 }),
            prop_builders: [props::calavera_skull, props::alebrije, props::ofrenda_velas, props::cempasuchil_arch],
            prop_counts: [9, 10, 10, 14],
            min_separation: 0.24,
            // Noise-deformed terrain (mountainous/uneven, not a perfect sphere) -- props use the
            // same height function via `surface_height` so they sit on it correctly.
            terrain_height: Some((mictlan_terrain_height, 0.12)),
        },
    ]
}
