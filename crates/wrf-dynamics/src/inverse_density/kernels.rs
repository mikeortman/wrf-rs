use wrf_compute::FieldStorage;

use crate::{InverseDensityRegion, InverseDensityResult};

/// Backend capability for WRF full inverse density on mass points.
///
/// Implementations reproduce `calc_alt` while retaining native field storage.
/// The CPU implementation uses the persistent default worker pool; a future
/// GPU backend can implement the same capability without host copies.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{InverseDensityKernels, InverseDensityRegion};
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(5, 5, 5)?;
/// let mut full = backend.create_field(shape, -1.0_f32)?;
/// let perturbation = backend.create_field(shape, 0.25_f32)?;
/// let base_state = backend.create_field(shape, 0.75_f32)?;
/// let region = InverseDensityRegion::try_new(
///     shape, 1..4, 1..4, 1..4, 1..5, 1..5, 1..5,
/// )?;
///
/// backend.calculate_full_inverse_density(
///     &mut full,
///     &perturbation,
///     &base_state,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait InverseDensityKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Adds perturbation and base-state inverse density over WRF's clipped tile.
    ///
    /// Storage outside the active mass-point range is left unchanged, matching
    /// observed WRF caller behavior. Validation completes before mutation.
    ///
    /// # Errors
    ///
    /// Returns a typed error if any field shape differs from `region`, or if
    /// backend execution fails.
    fn calculate_full_inverse_density(
        &self,
        full_inverse_density: &mut Self::Field,
        perturbation_inverse_density: &Self::Field,
        base_state_inverse_density: &Self::Field,
        region: &InverseDensityRegion,
    ) -> InverseDensityResult<()>;
}
