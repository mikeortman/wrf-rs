/// Borrowed map factors that participate in WRF omega diagnosis.
#[derive(Clone, Copy, Debug)]
pub struct OmegaDiagnosisMapFactors<'a, Field> {
    pub(crate) mass_point_west_east: &'a Field,
    pub(crate) west_east_momentum_south_north: &'a Field,
    pub(crate) inverse_south_north_momentum_west_east: &'a Field,
}

impl<'a, Field> OmegaDiagnosisMapFactors<'a, Field> {
    /// Groups the three map-factor arrays read by `calc_ww_cp`.
    pub const fn new(
        mass_point_west_east: &'a Field,
        west_east_momentum_south_north: &'a Field,
        inverse_south_north_momentum_west_east: &'a Field,
    ) -> Self {
        Self {
            mass_point_west_east,
            west_east_momentum_south_north,
            inverse_south_north_momentum_west_east,
        }
    }
}
