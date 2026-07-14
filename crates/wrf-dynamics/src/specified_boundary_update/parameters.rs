/// Scalar controls for one specified-zone tendency update.
#[derive(Clone, Copy, Debug)]
pub struct SpecifiedBoundaryUpdateParameters {
    pub(crate) time_step: f32,
    pub(crate) specified_zone_width: usize,
}

impl SpecifiedBoundaryUpdateParameters {
    /// Preserves WRF's IEEE timestep and permits a zero-width no-op zone.
    pub const fn new(time_step: f32, specified_zone_width: usize) -> Self {
        Self {
            time_step,
            specified_zone_width,
        }
    }
}
