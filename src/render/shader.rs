use crate::math::{Vec2, Vec3};
use raylib::color::Color;

/// Everything a fragment shader needs to compute the final pixel color. This is the seam where
/// future per-planet shaders (atmosphere, crystal, lava, toon...) plug in without touching the
/// rasterizer itself.
pub struct ShadingContext {
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
