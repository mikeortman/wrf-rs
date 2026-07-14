use wrf_compute::FieldStorage;

use crate::{
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryUpdateResult, SpecifiedBoundaryWestEastPeriodicity,
};

/// Backend capability for WRF specified-zone tendency updates.
///
/// Implementations retain native field storage while reproducing the field
/// location, tile clipping, and trapezoidal corners of `spec_bdyupdate`.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateKernels,
///     SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
///     SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 6)?;
/// let mut field = backend.create_field(shape, 1.0_f32)?;
/// let tendency = backend.create_field(shape, 0.25_f32)?;
/// let region = SpecifiedBoundaryUpdateRegion::try_new(
///     shape,
///     SpecifiedBoundaryFieldLocation::MassHalfLevel,
///     1..5,
///     1..5,
///     1..5,
///     1..6,
///     1..6,
///     1..6,
/// )?;
/// backend.update_specified_boundary(
///     &mut field,
///     &tendency,
///     SpecifiedBoundaryUpdateParameters::new(0.5, 2),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryUpdateKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Adds the supplied tendency in every contacted specified-zone point.
    ///
    /// A zero-width zone is an exact no-op. Validation completes before
    /// mutation, and storage outside the active boundary ranges is unchanged.
    ///
    /// # Errors
    ///
    /// Returns a typed error if either field shape differs from `region` or if
    /// backend execution fails.
    fn update_specified_boundary(
        &self,
        field: &mut Self::Field,
        tendency: &Self::Field,
        parameters: SpecifiedBoundaryUpdateParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryUpdateResult<()>;
}
