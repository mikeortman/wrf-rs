use wrf_compute::FieldStorage;

/// Borrowed current and saved momentum fluxes used by continuity and theta transport.
#[derive(Debug)]
pub struct AcousticMassThetaMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east: &'a Field,
    pub(crate) saved_west_east: &'a Field,
    pub(crate) south_north: &'a Field,
    pub(crate) saved_south_north: &'a Field,
    pub(crate) saved_vertical_mass_flux: &'a Field,
}

impl<Field> Copy for AcousticMassThetaMomentumInputs<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticMassThetaMomentumInputs<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticMassThetaMomentumInputs<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `u`, `u_1`, `v`, `v_1`, and `ww_1` without copying data.
    pub const fn new(
        west_east: &'a Field,
        saved_west_east: &'a Field,
        south_north: &'a Field,
        saved_south_north: &'a Field,
        saved_vertical_mass_flux: &'a Field,
    ) -> Self {
        Self {
            west_east,
            saved_west_east,
            south_north,
            saved_south_north,
            saved_vertical_mass_flux,
        }
    }
}
