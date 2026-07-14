use wrf_compute::FieldStorage;

use crate::{
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
    SpecifiedBoundaryZeroGradientParameters, SpecifiedBoundaryZeroGradientResult,
};

/// Backend capability for WRF zero-gradient specified boundaries.
///
/// Implementations copy from WRF's nearest independent interior row or column
/// while retaining native storage for CPU and future accelerator backends.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFieldLocation, SpecifiedBoundaryUpdateRegion,
///     SpecifiedBoundaryWestEastPeriodicity, SpecifiedBoundaryZeroGradientKernels,
///     SpecifiedBoundaryZeroGradientParameters,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 6)?;
/// let mut vertical_momentum = backend.create_field(shape, 1.0_f32)?;
/// let region = SpecifiedBoundaryUpdateRegion::try_new(
///     shape,
///     SpecifiedBoundaryFieldLocation::FullLevel,
///     1..5,
///     1..5,
///     1..5,
///     1..6,
///     1..6,
///     1..6,
/// )?;
/// backend.apply_zero_gradient_specified_boundary(
///     &mut vertical_momentum,
///     SpecifiedBoundaryZeroGradientParameters::new(1),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryZeroGradientKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Copies the nearest interior value into each active specified-zone point.
    ///
    /// Validation completes before mutation. A zero-width zone is an exact
    /// no-op, and inactive storage remains unchanged. As in WRF, execution
    /// starts at the tile's lower vertical bound and continues to the physical
    /// domain top regardless of the tile's supplied upper vertical bound.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an incompatible shape, a zone without an
    /// independent interior source, or backend execution failure.
    fn apply_zero_gradient_specified_boundary(
        &self,
        field: &mut Self::Field,
        parameters: SpecifiedBoundaryZeroGradientParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryZeroGradientResult<()>;
}
