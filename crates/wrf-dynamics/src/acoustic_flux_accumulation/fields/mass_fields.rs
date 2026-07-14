use wrf_compute::FieldStorage;

/// Staggered full column masses used to recouple horizontal linear fluxes.
#[derive(Clone, Copy, Debug)]
pub struct AcousticFluxMassFields<'a, Field: FieldStorage<f32>> {
    /// West-east staggered mass (`muu`).
    pub west_east: &'a Field,
    /// South-north staggered mass (`muv`).
    pub south_north: &'a Field,
}
