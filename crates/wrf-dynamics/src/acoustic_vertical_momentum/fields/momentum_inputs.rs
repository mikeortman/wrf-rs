use wrf_compute::FieldStorage;

/// Coupled momentum fields used by the vertical solve and surface condition.
#[derive(Debug)]
pub struct AcousticVerticalMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) tendency: &'a Field,
    pub(crate) vertical_mass_flux: &'a Field,
    pub(crate) saved_vertical_momentum: &'a Field,
    pub(crate) west_east_momentum: &'a Field,
    pub(crate) south_north_momentum: &'a Field,
}

impl<Field> Copy for AcousticVerticalMomentumInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticVerticalMomentumInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticVerticalMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `rw_tend`, `ww`, `w_save`, `u`, and `v`.
    pub const fn new(
        tendency: &'a Field,
        vertical_mass_flux: &'a Field,
        saved_vertical_momentum: &'a Field,
        west_east_momentum: &'a Field,
        south_north_momentum: &'a Field,
    ) -> Self {
        Self {
            tendency,
            vertical_mass_flux,
            saved_vertical_momentum,
            west_east_momentum,
            south_north_momentum,
        }
    }
}
