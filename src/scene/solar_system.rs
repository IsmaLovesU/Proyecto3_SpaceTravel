use super::body::{CelestialBody, OrbitParams};
use super::distribution::{scatter, ScatterSpec};
use super::props;
use super::ship::{self, Ship};
use crate::math::{Quat, Vec3};
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

const LOD_LEVELS: u32 = 4;
const SHIP_OBJ_PATH: &str = "assets/models/ship.obj";
const DEFAULT_SURFACE_SEED: u64 = 1;

/// One planet's full theme: visual identity, shader, and its 4 exclusive surface-model types with
/// how many of each to scatter.
struct PlanetTheme {
    name: &'static str,
    radius: f32,
    orbit_radius: f64,
    orbit_speed: f64,
    spin_speed: f64,
    color: Color,
    shader: Box<dyn FragmentShader>,
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
            mesh_lods: tinted_lods(Color::new(255, 200, 60, 255)),
            radius: 25.0,
            mass: 4000.0,
            orbit: None,
            spin_speed: 0.05,
            shader: Box::new(EmissiveCoronaShader::default()),
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
        // Procedural subdivision + Laplacian smoothing pass, applied once here -- rounds off the
        // loaded ship's low-poly silhouette before it's ever rasterized.
        let ship_mesh = subdivide_mesh(&ship_mesh, 1, 0.5);

        let ship = Ship {
            position: Vec3::new(0.0, 15.0, 340.0),
            orientation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            mesh: ship_mesh,
            mesh_scale: 1.0,
            collision_radius: 1.2,
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
                scale_range: (0.85, 1.15),
                surface_height: theme.terrain_height,
            };
            planet.surface_instances = scatter(seed, &spec);
        }
    }
}

fn tinted_lods(color: Color) -> Vec<Mesh> {
    icosphere_lod_levels(LOD_LEVELS).into_iter().map(|m| m.tint(color)).collect()
}

fn build_planet(theme: PlanetTheme, phase: f64, seed: u64) -> CelestialBody {
    let spec = ScatterSpec {
        counts: theme.prop_counts.to_vec(),
        min_separation: theme.min_separation,
        scale_range: (0.85, 1.15),
        surface_height: theme.terrain_height,
    };
    let surface_instances = scatter(seed, &spec);
    let prop_meshes: Vec<Mesh> = theme.prop_builders.iter().map(|f| f()).collect();

    let mut mesh_lods = tinted_lods(theme.color);
    if let Some((height_fn, amplitude)) = theme.terrain_height {
        for lod in &mut mesh_lods {
            deform_by_height(lod, height_fn, amplitude);
        }
    }

    CelestialBody {
        name: theme.name.to_string(),
        mesh_lods,
        radius: theme.radius,
        mass: theme.radius * 40.0,
        orbit: Some(OrbitParams { radius: theme.orbit_radius, speed: theme.orbit_speed, phase }),
        spin_speed: theme.spin_speed,
        shader: theme.shader,
        prop_meshes,
        surface_instances,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    }
}

/// The 5 themes, in a fixed order matching `SolarSystem.planets`. A free function (not a const)
/// because `Box<dyn FragmentShader>` needs to be constructed fresh each call (e.g. when `R`
/// re-scatters surfaces, we rebuild the theme list but reuse the planet's already-built shader).
fn planet_themes() -> Vec<PlanetTheme> {
    vec![
        // 1. Mario Galaxy -- "Reino Fungi Orbital"
        PlanetTheme {
            name: "Reino Fungi Orbital",
            radius: 4.0,
            orbit_radius: 70.0,
            orbit_speed: 0.35,
            spin_speed: 0.4,
            color: Color::new(90, 195, 110, 255),
            shader: Box::new(LambertShader { ambient: 0.3 }),
            prop_builders: [props::mario_mushroom, props::mario_question_block, props::mario_pipe, props::mario_star],
            prop_counts: [6, 8, 5, 14],
            min_separation: 0.22,
            terrain_height: None,
        },
        // 2. Guatemala futurista -- "Tikal Prime"
        PlanetTheme {
            name: "Tikal Prime",
            radius: 5.5,
            orbit_radius: 110.0,
            orbit_speed: 0.22,
            spin_speed: 0.5,
            color: Color::new(55, 70, 60, 255),
            shader: Box::new(JadeCircuitShader::default()),
            prop_builders: [
                props::guatemala_pyramid,
                props::guatemala_stela,
                props::guatemala_quetzal,
                props::guatemala_textile_panel,
            ],
            prop_counts: [4, 7, 5, 6],
            min_separation: 0.26,
            terrain_height: None,
        },
        // 3. Pokémon -- "Ruta Orbital"
        PlanetTheme {
            name: "Ruta Orbital",
            radius: 3.2,
            orbit_radius: 150.0,
            orbit_speed: 0.18,
            spin_speed: 0.6,
            color: Color::new(120, 200, 110, 255),
            // Procedural tangent-space normal map (own noise, no image file) gives the sphere a
            // bumpy grass/dirt texture -- the "mapeo de normales" requirement.
            shader: Box::new(BumpMappedShader {
                ambient: 0.28,
                normal_map: NormalMap::generate(64, 64, 10.0, 3.0, 3.0),
            }),
            prop_builders: [
                props::pokemon_pokeball,
                props::pokemon_berry_tree,
                props::pokemon_evolution_stone,
                props::pokemon_center,
            ],
            prop_counts: [8, 7, 6, 3],
            min_separation: 0.24,
            terrain_height: None,
        },
        // 4. Pac-Man -- "Laberinto Neón"
        PlanetTheme {
            name: "Laberinto Neon",
            radius: 6.0,
            orbit_radius: 200.0,
            orbit_speed: 0.12,
            spin_speed: 0.3,
            color: Color::new(8, 8, 25, 255),
            shader: Box::new(NeonGridShader::default()),
            prop_builders: [props::pacman_wall, props::pacman_dot, props::pacman_power_pellet, props::pacman_ghost],
            prop_counts: [22, 30, 6, 5],
            min_separation: 0.14,
            terrain_height: None,
        },
        // 5. Día de Muertos -- "Mictlán Celestial"
        PlanetTheme {
            name: "Mictlan Celestial",
            radius: 4.6,
            orbit_radius: 260.0,
            orbit_speed: 0.08,
            spin_speed: 0.45,
            color: Color::new(215, 130, 55, 255),
            shader: Box::new(LambertShader { ambient: 0.3 }),
            prop_builders: [props::calavera_skull, props::alebrije, props::ofrenda_velas, props::cempasuchil_arch],
            prop_counts: [4, 6, 5, 10],
            min_separation: 0.22,
            // Noise-deformed terrain (mountainous/uneven, not a perfect sphere) -- props use the
            // same height function via `surface_height` so they sit on it correctly.
            terrain_height: Some((mictlan_terrain_height, 0.12)),
        },
    ]
}
