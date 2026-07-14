use wrf_compute::FieldStorage;

/// Mutable running acoustic flux sums, finalized as time averages.
#[derive(Debug)]
pub struct AcousticFluxRunningAverages<'a, Field: FieldStorage<f32>> {
    /// West-east average (`ru_m`).
    pub west_east: &'a mut Field,
    /// South-north average (`rv_m`).
    pub south_north: &'a mut Field,
    /// Vertical average (`ww_m`).
    pub vertical: &'a mut Field,
}
