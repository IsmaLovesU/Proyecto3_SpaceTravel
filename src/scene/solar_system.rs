use super::body::{CelestialBody, OrbitParams};
use super::ship::{self, Ship};
use crate::math::{Quat, Vec3};
use crate::render::mesh::icosphere_lod_levels;
use crate::render::obj_loader::load_obj;
use crate::render::shader::{EmissiveCoronaShader, LambertShader};
use raylib::color::Color;

pub struct SolarSystem {
    pub sun: CelestialBody,
    pub planets: Vec<CelestialBody>,
    pub ship: Ship,
    pub sim_time: f64,
}

const LOD_LEVELS: u32 = 4;
const SHIP_OBJ_PATH: &str = "assets/models/ship.obj";

impl SolarSystem {
    pub fn new() -> Self {
        let sun = CelestialBody {
            name: "Sol".to_string(),
            mesh_lods: icosphere_lod_levels(LOD_LEVELS),
            radius: 25.0,
            mass: 4000.0,
            color: Color::new(255, 200, 60, 255),
            orbit: None,
            spin_speed: 0.05,
            shader: Box::new(EmissiveCoronaShader::default()),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        };

        // Placeholders: color/size/orbit only differ enough to tell them apart for now. Real
        // themes (surface types, custom shaders) come in a later pass, per planet.
        let placeholder_specs: [(&str, f32, f64, f64, Color); 5] = [
            ("Planeta 1", 4.0, 70.0, 0.35, Color::new(90, 170, 110, 255)),
            ("Planeta 2", 5.5, 110.0, 0.22, Color::new(190, 140, 90, 255)),
            ("Planeta 3", 3.2, 150.0, 0.18, Color::new(150, 210, 230, 255)),
            ("Planeta 4", 6.0, 200.0, 0.12, Color::new(120, 120, 140, 255)),
            ("Planeta 5", 4.6, 260.0, 0.08, Color::new(200, 90, 70, 255)),
        ];

        let planets = placeholder_specs
            .into_iter()
            .enumerate()
            .map(|(i, (name, radius, orbit_radius, orbit_speed, color))| CelestialBody {
                name: name.to_string(),
                mesh_lods: icosphere_lod_levels(LOD_LEVELS),
                radius,
                mass: radius * 40.0,
                color,
                orbit: Some(OrbitParams { radius: orbit_radius, speed: orbit_speed, phase: i as f64 * 1.3 }),
                spin_speed: 0.4 + i as f64 * 0.1,
                shader: Box::new(LambertShader::default()),
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            })
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

        let ship = Ship {
            position: Vec3::new(0.0, 15.0, 340.0),
            orientation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            mesh: ship_mesh,
            mesh_scale: 1.0,
            collision_radius: 1.2,
        };

        SolarSystem { sun, planets, ship, sim_time: 0.0 }
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
}
