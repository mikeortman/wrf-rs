/// Scalar controls for WRF specified-boundary finalization.
#[derive(Clone, Copy, Debug)]
pub struct SpecifiedBoundaryFinalizationParameters {
    pub(crate) time_step: f32,
    pub(crate) boundary_width: usize,
    pub(crate) specified_zone_width: usize,
}

impl SpecifiedBoundaryFinalizationParameters {
    /// Captures the boundary-file width, active specified zone, and interpolation time.
    pub const fn new(time_step: f32, boundary_width: usize, specified_zone_width: usize) -> Self {
        Self {
            time_step,
            boundary_width,
            specified_zone_width,
        }
    }

    /// Returns the number of stored points normal to each boundary.
    pub const fn boundary_width(self) -> usize {
        self.boundary_width
    }

    /// Returns the outer zone forced to boundary-file values.
    pub const fn specified_zone_width(self) -> usize {
        self.specified_zone_width
    }
}
