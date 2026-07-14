use wrf_compute::FieldStorage;

use crate::{PressurePointGeopotentialRegion, PressurePointGeopotentialResult};

/// Backend capability for WRF full geopotential at pressure points.
///
/// Implementations reproduce `calc_php` while retaining native field storage.
/// The CPU implementation uses the persistent default worker pool; a future
/// GPU backend can implement the same capability without host copies.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     PressurePointGeopotentialKernels, PressurePointGeopotentialRegion,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(5, 5, 6)?;
/// let mut pressure_points = backend.create_field(shape, -1.0_f32)?;
/// let perturbation = backend.create_field(shape, 0.25_f32)?;
/// let base_state = backend.create_field(shape, 0.75_f32)?;
/// let region = PressurePointGeopotentialRegion::try_new(
///     shape, 1..4, 1..4, 1..4, 1..5, 1..5, 1..5,
/// )?;
///
/// backend.calculate_pressure_point_geopotential(
///     &mut pressure_points,
///     &perturbation,
///     &base_state,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait PressurePointGeopotentialKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Averages full-level base-state and perturbation geopotential to mass levels.
    ///
    /// Storage outside the clipped active tile is left unchanged. All field and
    /// vertical-neighbor validation completes before mutation.
    ///
    /// # Errors
    ///
    /// Returns a typed error if any field shape differs from `region`, the
    /// region lacks WRF's upper full-level neighbor, or backend execution fails.
    fn calculate_pressure_point_geopotential(
        &self,
        pressure_point_geopotential: &mut Self::Field,
        perturbation_full_level_geopotential: &Self::Field,
        base_state_full_level_geopotential: &Self::Field,
        region: &PressurePointGeopotentialRegion,
    ) -> PressurePointGeopotentialResult<()>;
}
