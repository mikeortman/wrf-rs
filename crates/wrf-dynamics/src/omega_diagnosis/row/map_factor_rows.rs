#[derive(Clone, Copy)]
pub(crate) struct OmegaDiagnosisMapFactorRows<'a> {
    pub(super) mass_point_west_east: &'a [f32],
    pub(super) west_east_momentum_south_north: &'a [f32],
    pub(super) east_momentum_south_north: &'a [f32],
    pub(super) inverse_south_north_momentum_west_east: &'a [f32],
    pub(super) inverse_north_momentum_west_east: &'a [f32],
}

impl<'a> OmegaDiagnosisMapFactorRows<'a> {
    pub(crate) fn new(
        mass_point_west_east: &'a [f32],
        west_east_momentum_south_north: &'a [f32],
        east_momentum_south_north: &'a [f32],
        inverse_south_north_momentum_west_east: &'a [f32],
        inverse_north_momentum_west_east: &'a [f32],
    ) -> Self {
        let point_count = mass_point_west_east.len();
        for row in [
            west_east_momentum_south_north,
            east_momentum_south_north,
            inverse_south_north_momentum_west_east,
            inverse_north_momentum_west_east,
        ] {
            assert_eq!(row.len(), point_count);
        }
        Self {
            mass_point_west_east,
            west_east_momentum_south_north,
            east_momentum_south_north,
            inverse_south_north_momentum_west_east,
            inverse_north_momentum_west_east,
        }
    }

    pub(super) const fn point_count(&self) -> usize {
        self.mass_point_west_east.len()
    }
}
