use raylib::color::Color;

/// Our own color + depth buffers. Raylib is only used afterwards to blit `to_rgba_bytes()` onto
/// a texture and present it — all writes here come from our software rasterizer.
pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub color: Vec<Color>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let count = (width * height) as usize;
        Framebuffer {
            width,
            height,
            color: vec![Color::BLACK; count],
            depth: vec![1.0; count],
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.color.fill(color);
        self.depth.fill(1.0);
    }

    /// Packs the color buffer as RGBA8 into `out`, reusing its allocation frame to frame.
    pub fn write_rgba_into(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(self.color.len() * 4);
        for c in &self.color {
            out.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }
}
