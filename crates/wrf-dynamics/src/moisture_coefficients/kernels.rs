use wrf_compute::FieldStorage;

use crate::{
    MoistureCoefficientOutputs, MoistureCoefficientRegion, MoistureCoefficientResult,
    MoistureSpecies,
};

/// Backend capability for WRF moisture coefficients on momentum staggers.
///
/// Implementations reproduce `calc_cq` while retaining native field storage.
/// The CPU implementation uses the persistent default worker pool; future GPU
/// implementations can provide device kernels behind the same capability.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     MoistureCoefficientKernels, MoistureCoefficientOutputs,
///     MoistureCoefficientRegion, MoistureSpecies,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(5, 5, 5)?;
/// let mut west_east = backend.create_field(shape, -1.0_f32)?;
/// let mut south_north = backend.create_field(shape, -1.0_f32)?;
/// let mut vertical = backend.create_field(shape, -1.0_f32)?;
/// let water_vapor = backend.create_field(shape, 0.01_f32)?;
/// let species = [water_vapor];
/// let region = MoistureCoefficientRegion::try_new(
///     shape, 1..4, 1..4, 1..4, 1..5, 1..5, 1..5,
/// )?;
///
/// backend.calculate_moisture_coefficients(
///     MoistureCoefficientOutputs::new(&mut west_east, &mut south_north, &mut vertical),
///     MoistureSpecies::new(&species),
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait MoistureCoefficientKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Calculates `cqu`, `cqv`, and `cqw` with WRF clipping and operation order.
    ///
    /// # Errors
    ///
    /// Returns a typed error if an output or active species shape differs from
    /// `region`, or if backend execution fails. Validation completes before any
    /// output changes.
    fn calculate_moisture_coefficients(
        &self,
        outputs: MoistureCoefficientOutputs<'_, Self::Field>,
        moisture: MoistureSpecies<'_, Self::Field>,
        region: &MoistureCoefficientRegion,
    ) -> MoistureCoefficientResult<()>;
}
