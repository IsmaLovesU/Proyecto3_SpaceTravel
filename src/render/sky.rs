//! Background elements drawn straight into the framebuffer before any geometry: a procedural
//! starfield and the sun's screen-space halo. Both write color only (depth stays cleared), so
//! every real object rasterized afterwards simply covers them via the normal depth test.

use super::framebuffer::Framebuffer;
use crate::math::{Mat4, Vec3, Vec4};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use raylib::color::Color;

struct Star {
    dir: Vec3,
    color: Color,
    /// 1 = single pixel, 2 = 2x2 block (the few "bright" stars).
    size: u8,
    twinkle_phase: f32,
}

pub struct Sky {
    stars: Vec<Star>,
}

impl Sky {
    pub fn new(count: usize, seed: u64) -> Sky {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut stars = Vec::with_capacity(count);
        for _ in 0..count {
            // Marsaglia: uniform direction on the sphere.
            let dir = loop {
                let x1 = rng.random_range(-1.0f32..1.0f32);
                let x2 = rng.random_range(-1.0f32..1.0f32);
                let d2 = x1 * x1 + x2 * x2;
                if d2 < 1.0 {
                    let s = (1.0 - d2).sqrt();
                    break Vec3::new(2.0 * x1 * s, 2.0 * x2 * s, 1.0 - 2.0 * d2);
                }
            };
            let brightness = rng.random_range(0.35f32..1.0f32);
            let tint = rng.random_range(0.0f32..1.0f32);
            let (r, g, b) = if tint < 0.15 {
                (255.0, 215.0, 170.0) // warm
            } else if tint < 0.35 {
                (180.0, 205.0, 255.0) // cool blue
            } else {
                (235.0, 238.0, 250.0) // white
            };
            let color = Color::new((r * brightness) as u8, (g * brightness) as u8, (b * brightness) as u8, 255);
            let size = if brightness > 0.9 { 2 } else { 1 };
            stars.push(Star { dir, color, size, twinkle_phase: rng.random_range(0.0f32..6.28f32) });
        }
        Sky { stars }
    }

    /// Projects every star as a point at "infinity" (a fixed large distance along its direction
    /// from the camera, so it never parallaxes) and plots it. Cheap: one matrix multiply per star.
    /// `warp` (0..1) is the "light speed" effect strength: stars stretch into streaks radiating
    /// from the screen center, longer and brighter toward the edges. Purely visual -- the ship's
    /// actual speed is untouched.
    pub fn draw_stars(&self, fb: &mut Framebuffer, view_proj: &Mat4, camera_pos: Vec3, time: f64, warp: f32) {
        let (w, h) = (fb.width, fb.height);
        let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
        let warp = warp.clamp(0.0, 1.0);
        for star in &self.stars {
            let p = camera_pos + star.dir * 2000.0;
            let c = view_proj.mul_vec4(Vec4::from_point(p));
            if c.w <= 0.0 {
                continue;
            }
            let inv_w = 1.0 / c.w;
            let nx = c.x * inv_w;
            let ny = c.y * inv_w;
            if nx < -1.0 || nx > 1.0 || ny < -1.0 || ny > 1.0 {
                continue;
            }
            let sx = ((nx * 0.5 + 0.5) * w as f32) as i32;
            let sy = ((1.0 - (ny * 0.5 + 0.5)) * h as f32) as i32;

            let twinkle = 0.8 + 0.2 * ((time as f32) * 1.7 + star.twinkle_phase).sin();
            // While warping, stars go brighter and shift toward blue-white.
            let boost = 1.0 + warp * 0.6;
            let col = Color::new(
                (star.color.r as f32 * twinkle * boost).min(255.0) as u8,
                (star.color.g as f32 * twinkle * boost).min(255.0) as u8,
                ((star.color.b as f32 * twinkle + warp * 60.0) * boost).min(255.0) as u8,
                255,
            );

            if warp > 0.02 {
                // Streak trailing back toward the screen center: length grows with warp and with
                // the star's distance from the center (stars near the edge fly past fastest).
                let (dx, dy) = (sx as f32 - cx, sy as f32 - cy);
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let len = (warp * (10.0 + dist * 0.45)).min(160.0);
                let (ux, uy) = (dx / dist, dy / dist);
                plot_line(fb, sx as f32, sy as f32, sx as f32 - ux * len, sy as f32 - uy * len, col);
            }
            let size = star.size as i32;
            for dy in 0..size {
                for dx in 0..size {
                    let (x, y) = (sx + dx, sy + dy);
                    if x >= 0 && y >= 0 && x < w && y < h {
                        fb.color[(y * w + x) as usize] = col;
                    }
                }
            }
        }
    }

    /// Radial "tunnel" tint while warping: a blue-white vignette that gets stronger toward the
    /// screen edges, so the whole frame reads as motion even where there are few stars.
    pub fn draw_warp_vignette(fb: &mut Framebuffer, warp: f32) {
        let warp = warp.clamp(0.0, 1.0);
        if warp < 0.02 {
            return;
        }
        let (w, h) = (fb.width as usize, fb.height as usize);
        let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
        let inv_r = 1.0 / (cx * cx + cy * cy).sqrt();
        for y in 0..h {
            let dy = (y as f32 + 0.5 - cy) * inv_r;
            let row = y * w;
            for x in 0..w {
                let dx = (x as f32 + 0.5 - cx) * inv_r;
                let d = dx * dx + dy * dy; // 0 at center, ~1 at corners
                if d < 0.12 {
                    continue;
                }
                let k = ((d - 0.12) * 1.4).min(1.0) * warp;
                let px = fb.color[row + x];
                fb.color[row + x] = Color::new(
                    (px.r as f32 + 35.0 * k).min(255.0) as u8,
                    (px.g as f32 + 55.0 * k).min(255.0) as u8,
                    (px.b as f32 + 105.0 * k).min(255.0) as u8,
                    255,
                );
            }
        }
    }

    /// Additive radial halo around the sun's projected disc. The sun's own mesh is drawn on top
    /// afterwards; this just gives it a soft corona that bleeds into the sky, and it pulses in
    /// sync with the emissive shader.
    pub fn draw_sun_glow(fb: &mut Framebuffer, view_proj: &Mat4, sun_pos: Vec3, sun_radius: f32, camera_up: Vec3, time: f64) {
        let c = view_proj.mul_vec4(Vec4::from_point(sun_pos));
        if c.w <= 0.0 {
            return;
        }
        let edge = view_proj.mul_vec4(Vec4::from_point(sun_pos + camera_up * sun_radius));
        if edge.w <= 0.0 {
            return;
        }
        let (w, h) = (fb.width as f32, fb.height as f32);
        let to_screen = |v: Vec4| ((v.x / v.w * 0.5 + 0.5) * w, (1.0 - (v.y / v.w * 0.5 + 0.5)) * h);
        let (cx, cy) = to_screen(c);
        let (ex, ey) = to_screen(edge);
        let disc_px = ((ex - cx).powi(2) + (ey - cy).powi(2)).sqrt().max(1.0);
        let glow_px = (disc_px * 2.4).min(w.max(h));

        let min_x = (cx - glow_px).floor().max(0.0) as i32;
        let max_x = (cx + glow_px).ceil().min(w - 1.0) as i32;
        let min_y = (cy - glow_px).floor().max(0.0) as i32;
        let max_y = (cy + glow_px).ceil().min(h - 1.0) as i32;
        if min_x > max_x || min_y > max_y {
            return;
        }

        let pulse = 0.85 + 0.15 * (time * 1.6).sin() as f32;
        let width = fb.width as usize;
        let inv_disc = 1.0 / disc_px;
        // Falloff lookup indexed by d^2 (in rim units), so the inner loop needs no sqrt/exp. The
        // curve is shifted to reach exactly 0 at the outer radius (no visible disc edge).
        const OUTER: f32 = 2.4;
        const LUT_SIZE: usize = 256;
        let edge_val = (-(OUTER - 1.0) * 2.2f32).exp();
        let lut: Vec<f32> = (0..LUT_SIZE)
            .map(|i| {
                let d2 = (i as f32 / (LUT_SIZE - 1) as f32) * OUTER * OUTER;
                let d = d2.sqrt();
                if d < 1.0 { 1.0 } else { (((-(d - 1.0) * 2.2).exp() - edge_val) / (1.0 - edge_val)).max(0.0) }
            })
            .collect();
        let lut_scale = (LUT_SIZE - 1) as f32 / (OUTER * OUTER);
        for y in min_y..=max_y {
            let dy = (y as f32 + 0.5 - cy) * inv_disc;
            for x in min_x..=max_x {
                let dx = (x as f32 + 0.5 - cx) * inv_disc;
                let d2 = dx * dx + dy * dy;
                if d2 >= OUTER * OUTER {
                    continue;
                }
                let k = lut[(d2 * lut_scale) as usize] * pulse;
                if k <= 0.003 {
                    continue;
                }
                let idx = y as usize * width + x as usize;
                let px = fb.color[idx];
                fb.color[idx] = Color::new(
                    (px.r as f32 + 255.0 * k).min(255.0) as u8,
                    (px.g as f32 + 150.0 * k).min(255.0) as u8,
                    (px.b as f32 + 45.0 * k).min(255.0) as u8,
                    255,
                );
            }
        }
    }
}

/// Simple DDA line into the color buffer (clipped per pixel). Used for warp streaks.
fn plot_line(fb: &mut Framebuffer, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let (w, h) = (fb.width, fb.height);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as i32;
    let (sx, sy) = (dx / steps as f32, dy / steps as f32);
    let (mut x, mut y) = (x0, y0);
    for i in 0..=steps {
        let (px, py) = (x as i32, y as i32);
        if px >= 0 && py >= 0 && px < w && py < h {
            // Fade the tail so the streak tapers toward the center.
            let t = 1.0 - i as f32 / steps as f32;
            let idx = (py * w + px) as usize;
            let old = fb.color[idx];
            fb.color[idx] = Color::new(
                (old.r as f32).max(color.r as f32 * t) as u8,
                (old.g as f32).max(color.g as f32 * t) as u8,
                (old.b as f32).max(color.b as f32 * t) as u8,
                255,
            );
        }
        x += sx;
        y += sy;
    }
}
