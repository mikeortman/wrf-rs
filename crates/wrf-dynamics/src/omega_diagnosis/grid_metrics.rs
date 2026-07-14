/// Inverse horizontal grid spacing used by omega diagnosis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OmegaDiagnosisGridMetrics {
    pub(crate) inverse_west_east_spacing: f32,
    pub(crate) inverse_south_north_spacing: f32,
}

impl OmegaDiagnosisGridMetrics {
    /// Creates the grid metrics without changing WRF's floating-point policy.
    pub const fn new(inverse_west_east_spacing: f32, inverse_south_north_spacing: f32) -> Self {
        Self {
            inverse_west_east_spacing,
            inverse_south_north_spacing,
        }
    }
}
