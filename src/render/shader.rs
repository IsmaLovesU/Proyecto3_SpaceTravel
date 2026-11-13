use super::normal_map::NormalMap;
use crate::math::{Vec2, Vec3};
use raylib::color::Color;

/// Everything a fragment shader needs to compute the final pixel color. This is the seam where
/// future per-planet shaders (atmosphere, crystal, lava, toon...) plug in without touching the
/// rasterizer itself.
pub struct ShadingContext {
    /// Not read by the current shaders, but part of the stable shader interface (position-based
    /// effects such as world-space noise plug in here without touching the rasterizer).
    #[allow(dead_code)]
    pub world_pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub base_color: Color,
    pub light_dir: Vec3,
    pub view_dir: Vec3,
    pub time: f64,
}

pub trait FragmentShader: Send + Sync {
    fn shade(&self, ctx: &ShadingContext) -> Color;
}

fn scale_color(c: Color, f: f32) -> Color {
    let f = f.max(0.0);
    Color {
        r: (c.r as f32 * f).clamp(0.0, 255.0) as u8,
        g: (c.g as f32 * f).clamp(0.0, 255.0) as u8,
        b: (c.b as f32 * f).clamp(0.0, 255.0) as u8,
        a: c.a,
    }
}

/// Default placeholder shading: simple Lambert diffuse + a flat ambient term. Every body uses
/// this until it gets a themed shader later.
pub struct LambertShader {
    pub ambient: f32,
}

impl Default for LambertShader {
    fn default() -> Self {
        LambertShader { ambient: 0.15 }
    }
}

impl FragmentShader for LambertShader {
    fn shade(&self, ctx: &ShadingContext) -> Color {
        let diffuse = ctx.normal.normalized().dot(-ctx.light_dir.normalized()).max(0.0);
        let intensity = self.ambient + (1.0 - self.ambient) * diffuse;
        scale_color(ctx.base_color, intensity)
    }
}

/// The sun's own shader: fully emissive, with a soft pulsing glow driven by elapsed time. This is
/// the "shader animado" of the project (its visual output changes over time, not just per-frame
/// lighting) and also one of the four distinct shaders.
pub struct EmissiveCoronaShader {
    pub pulse_speed: f64,
    pub pulse_strength: f32,
}

impl Default for EmissiveCoronaShader {
    fn default() -> Self {
        EmissiveCoronaShader { pulse_speed: 1.6, pulse_strength: 0.25 }
    }
}

impl FragmentShader for EmissiveCoronaShader {
    fn shade(&self, ctx: &ShadingContext) -> Color {
        let pulse = 1.0 + self.pulse_strength * (ctx.time * self.pulse_speed).sin() as f32;
        // Fresnel-like rim brightening: edges (grazing view angle) glow hotter.
        let ndotv = ctx.normal.normalized().dot(ctx.view_dir.normalized()).abs().clamp(0.0, 1.0);
        let rim = 1.0 - ndotv;
        let intensity = (1.0 + rim * 0.8) * pulse;
        scale_color(ctx.base_color, intensity)
    }
}

/// Ruta Orbital's shader: Lambert shading with the surface normal perturbed by a procedural
/// tangent-space normal map, giving the sphere a bumpy/textured look instead of a perfectly
/// smooth one. This is the "mapeo de normales implementado en el renderizador por software" item
/// -- a separate technique from the four themed shaders above, pluggable the same way.
pub struct BumpMappedShader {
    pub ambient: f32,
    pub normal_map: NormalMap,
}

impl FragmentShader for BumpMappedShader {
    fn shade(&self, ctx: &ShadingContext) -> Color {
        let n = ctx.normal.normalized();
        // World-aligned tangent basis: good enough for a convincing bump effect without needing
        // per-vertex tangents through the pipeline. Falls back to a different reference axis near
        // the poles, where `UP` would be parallel to the normal and the cross product degenerates.
        let up_ref = if n.y.abs() > 0.99 { Vec3::RIGHT } else { Vec3::UP };
        let tangent = up_ref.cross(n).normalized();
        let bitangent = n.cross(tangent);

        let bump = self.normal_map.sample(ctx.uv);
        let perturbed = (tangent * bump.x + bitangent * bump.y + n * bump.z).normalized();

        let diffuse = perturbed.dot(-ctx.light_dir.normalized()).max(0.0);
        let intensity = self.ambient + (1.0 - self.ambient) * diffuse;
        scale_color(ctx.base_color, intensity)
    }
}

fn add_glow(c: Color, glow: (f32, f32, f32)) -> Color {
    Color {
        r: (c.r as f32 + glow.0).clamp(0.0, 255.0) as u8,
        g: (c.g as f32 + glow.1).clamp(0.0, 255.0) as u8,
        b: (c.b as f32 + glow.2).clamp(0.0, 255.0) as u8,
        a: c.a,
    }
}

/// 0 away from a UV grid line, rising to 1 exactly on one (`frequency` lines per UV unit, each
/// `line_width` UV-units wide). Shared by the two grid-based shaders below.
fn grid_mask(uv: Vec2, frequency: f32, line_width: f32) -> f32 {
    let dist_to_line = |v: f32| {
        let g = v * frequency;
        let frac = g - g.floor();
        frac.min(1.0 - frac)
    };
    let d = dist_to_line(uv.x).min(dist_to_line(uv.y));
    (1.0 - d / line_width.max(1e-4)).clamp(0.0, 1.0)
}

/// Guatemala futurista planet: a jade/stone base with a glowing Maya-glyph-inspired circuit grid,
/// brighter at grazing angles (rim light) — one of the four distinct shaders.
pub struct JadeCircuitShader {
    pub ambient: f32,
    pub grid_frequency: f32,
    pub line_width: f32,
    pub glow_color: (f32, f32, f32),
}

impl Default for JadeCircuitShader {
    fn default() -> Self {
        JadeCircuitShader { ambient: 0.25, grid_frequency: 26.0, line_width: 0.03, glow_color: (40.0, 210.0, 170.0) }
    }
}

impl FragmentShader for JadeCircuitShader {
    fn shade(&self, ctx: &ShadingContext) -> Color {
        let diffuse = ctx.normal.normalized().dot(-ctx.light_dir.normalized()).max(0.0);
        let intensity = self.ambient + (1.0 - self.ambient) * diffuse;
        let lit = scale_color(ctx.base_color, intensity);

        let mask = grid_mask(ctx.uv, self.grid_frequency, self.line_width);
        let ndotv = ctx.normal.normalized().dot(ctx.view_dir.normalized()).abs().clamp(0.0, 1.0);
        let rim = (1.0 - ndotv).powf(2.0);
        let glow = mask + rim * 0.4;
        add_glow(lit, (self.glow_color.0 * glow, self.glow_color.1 * glow, self.glow_color.2 * glow))
    }
}

/// Pac-Man planet: a dark arcade-cabinet base with a glowing neon maze grid that pulses gently
/// over time — one of the four distinct shaders.
pub struct NeonGridShader {
    pub ambient: f32,
    pub grid_frequency: f32,
    pub line_width: f32,
    pub glow_color: (f32, f32, f32),
    pub pulse_speed: f64,
}

impl Default for NeonGridShader {
    fn default() -> Self {
        NeonGridShader {
            ambient: 0.35,
            grid_frequency: 22.0,
            line_width: 0.04,
            glow_color: (40.0, 160.0, 255.0),
            pulse_speed: 2.4,
        }
    }
}

impl FragmentShader for NeonGridShader {
    fn shade(&self, ctx: &ShadingContext) -> Color {
        let diffuse = ctx.normal.normalized().dot(-ctx.light_dir.normalized()).max(0.0);
        let intensity = self.ambient + (1.0 - self.ambient) * diffuse;
        let lit = scale_color(ctx.base_color, intensity);

        let mask = grid_mask(ctx.uv, self.grid_frequency, self.line_width);
        let pulse = 0.7 + 0.3 * (ctx.time * self.pulse_speed).sin() as f32;
        let glow = mask * pulse;
        add_glow(lit, (self.glow_color.0 * glow, self.glow_color.1 * glow, self.glow_color.2 * glow))
    }
}
