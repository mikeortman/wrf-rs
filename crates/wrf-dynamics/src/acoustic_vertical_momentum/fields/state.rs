use wrf_compute::FieldStorage;

/// Mutable prognostic and diagnostic state updated by `advance_w`.
#[derive(Debug)]
pub struct AcousticVerticalState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) vertical_momentum: &'a mut Field,
    pub(crate) perturbation_geopotential: &'a mut Field,
    pub(crate) time_averaged_thermodynamics: &'a mut Field,
}

impl<'a, Field> AcousticVerticalState<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `w`, `ph`, and `t_2ave` as non-aliasing mutable fields.
    pub const fn new(
        vertical_momentum: &'a mut Field,
        perturbation_geopotential: &'a mut Field,
        time_averaged_thermodynamics: &'a mut Field,
    ) -> Self {
        Self {
            vertical_momentum,
            perturbation_geopotential,
            time_averaged_thermodynamics,
        }
    }
}
