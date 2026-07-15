use wrf_compute::FieldStorage;

/// Immutable large-step and final column masses used to uncouple fields.
#[derive(Clone, Copy, Debug)]
pub struct AcousticStepFinalizationMasses<'a, Field: FieldStorage<f32>> {
    pub(crate) large_step_full: &'a Field,
    pub(crate) final_full: &'a Field,
    pub(crate) large_step_west_east: &'a Field,
    pub(crate) final_west_east: &'a Field,
    pub(crate) large_step_south_north: &'a Field,
    pub(crate) final_south_north: &'a Field,
}

impl<'a, Field: FieldStorage<f32>> AcousticStepFinalizationMasses<'a, Field> {
    /// Groups WRF `mut`, `muts`, `muu`, `muus`, `muv`, and `muvs`.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        large_step_full: &'a Field,
        final_full: &'a Field,
        large_step_west_east: &'a Field,
        final_west_east: &'a Field,
        large_step_south_north: &'a Field,
        final_south_north: &'a Field,
    ) -> Self {
        Self {
            large_step_full,
            final_full,
            large_step_west_east,
            final_west_east,
            large_step_south_north,
            final_south_north,
        }
    }
}
