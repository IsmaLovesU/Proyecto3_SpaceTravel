/// Picks a LOD index (into a body's `mesh_lods`, finest last) from camera distance and a set of
/// ascending distance thresholds. `thresholds[i]` is the distance below which LOD level `i + 1`
/// (the next-finer level) is used; beyond the last threshold the coarsest level (0) is used.
pub fn choose_lod(distance: f32, thresholds: &[f32], level_count: usize) -> usize {
    if level_count == 0 {
        return 0;
    }
    let finest = level_count - 1;
    for (i, &t) in thresholds.iter().enumerate() {
        if distance < t {
            return (finest - i).min(finest);
        }
    }
    0
}
