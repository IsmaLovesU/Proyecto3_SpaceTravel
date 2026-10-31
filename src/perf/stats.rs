#[derive(Default, Clone, Copy)]
pub struct FrameStats {
    pub bodies_total: u32,
    pub bodies_culled: u32,
    pub triangles_submitted: u64,
    pub triangles_backface_culled: u64,
    pub triangles_rasterized: u64,
}

impl FrameStats {
    pub fn reset(&mut self) {
        *self = FrameStats::default();
    }
}
