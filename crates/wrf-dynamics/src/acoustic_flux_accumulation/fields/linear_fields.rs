use wrf_compute::FieldStorage;

/// Saved large-step linear fluxes added after the final substep.
#[derive(Clone, Copy, Debug)]
pub struct AcousticFluxLinearFields<'a, Field: FieldStorage<f32>> {
    /// West-east linear flux (`u_lin`).
    pub west_east: &'a Field,
    /// South-north linear flux (`v_lin`).
    pub south_north: &'a Field,
    /// Vertical linear flux (`ww_lin`).
    pub vertical: &'a Field,
}
