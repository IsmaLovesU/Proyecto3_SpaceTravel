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

    #[inline]
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            None
        } else {
            Some((y * self.width + x) as usize)
        }
    }

    /// Standard depth test (closer wins, smaller NDC z), without writing. Lets the caller skip
    /// shading work for pixels that would be discarded anyway.
    #[inline]
    pub fn depth_test(&self, x: i32, y: i32, depth: f32) -> bool {
        match self.index(x, y) {
            Some(i) => depth < self.depth[i],
            None => false,
        }
    }

    #[inline]
    pub fn write_pixel(&mut self, x: i32, y: i32, depth: f32, color: Color) {
        if let Some(i) = self.index(x, y) {
            self.depth[i] = depth;
            self.color[i] = color;
        }
    }

    pub fn to_rgba_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.color.len() * 4);
        for c in &self.color {
            bytes.push(c.r);
            bytes.push(c.g);
            bytes.push(c.b);
            bytes.push(c.a);
        }
        bytes
    }
}
