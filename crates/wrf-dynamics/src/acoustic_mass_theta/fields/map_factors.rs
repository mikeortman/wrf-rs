use wrf_compute::FieldStorage;

/// Borrowed map factors actually read by WRF `advance_mu_t`.
#[derive(Debug)]
pub struct AcousticMassThetaMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    pub(crate) west_east_y: &'a Field,
    pub(crate) inverse_south_north_x: &'a Field,
    pub(crate) mass_point_x: &'a Field,
    pub(crate) mass_point_y: &'a Field,
}

impl<Field> Copy for AcousticMassThetaMapFactors<'_, Field> where Field: FieldStorage<f32> {}

impl<Field> Clone for AcousticMassThetaMapFactors<'_, Field>
where
    Field: FieldStorage<f32>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Field> AcousticMassThetaMapFactors<'a, Field>
where
    Field: FieldStorage<f32>,
{
    /// Groups WRF `msfuy`, `msfvx_inv`, `msftx`, and `msfty`.
    pub const fn new(
        west_east_y: &'a Field,
        inverse_south_north_x: &'a Field,
        mass_point_x: &'a Field,
        mass_point_y: &'a Field,
    ) -> Self {
        Self {
            west_east_y,
            inverse_south_north_x,
            mass_point_x,
            mass_point_y,
        }
    }
}
