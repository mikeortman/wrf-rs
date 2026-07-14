use wrf_compute::FieldStorage;

/// Mutable saved boundary tendencies owned once across the three-stage sequence.
///
/// WRF's `*_save` fields are relaxed in place on the first Runge–Kutta substep
/// and then consumed immutably by dry-tendency assembly. Grouping them behind
/// one mutable owner lets the stage hand mutable reborrows to relaxation and
/// immutable reborrows to assembly without cloning any field.
pub struct DryLargeStepSavedTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_momentum: &'a mut Field,
    pub(crate) south_north_momentum: &'a mut Field,
    pub(crate) vertical_momentum: &'a mut Field,
    pub(crate) geopotential: &'a mut Field,
    pub(crate) potential_temperature: &'a mut Field,
}

impl<'a, Field> DryLargeStepSavedTendencies<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF's `u_save`, `v_save`, `w_save`, `ph_save`, and `t_save`.
    pub const fn new(
        west_east_momentum: &'a mut Field,
        south_north_momentum: &'a mut Field,
        vertical_momentum: &'a mut Field,
        geopotential: &'a mut Field,
        potential_temperature: &'a mut Field,
    ) -> Self {
        Self {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
        }
    }
}
