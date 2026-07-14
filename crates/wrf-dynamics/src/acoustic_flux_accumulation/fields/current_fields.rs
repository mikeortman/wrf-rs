use wrf_compute::FieldStorage;

/// Current nonlinear mass fluxes read during one acoustic substep.
#[derive(Clone, Copy, Debug)]
pub struct AcousticFluxCurrentFields<'a, Field: FieldStorage<f32>> {
    /// West-east flux (`ru`).
    pub west_east: &'a Field,
    /// South-north flux (`rv`).
    pub south_north: &'a Field,
    /// Vertical flux (`ww`).
    pub vertical: &'a Field,
}
