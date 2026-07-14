use wrf_compute::FieldStorage;

/// Horizontal map factors actually read by WRF `sumflux`.
#[derive(Clone, Copy, Debug)]
pub struct AcousticFluxMapFactors<'a, Field: FieldStorage<f32>> {
    /// West-east Y map factor (`msfuy`).
    pub west_east_y: &'a Field,
    /// Inverse south-north X map factor (`msfvx_inv`).
    pub inverse_south_north_x: &'a Field,
}
