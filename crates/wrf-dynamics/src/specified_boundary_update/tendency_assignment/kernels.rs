use wrf_compute::FieldStorage;

use super::{
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyParameters,
    SpecifiedBoundaryTendencyResult,
};
use crate::{SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity};

/// Backend capability for assigning WRF boundary-file tendencies.
///
/// Implementations retain native field storage so a future GPU backend can
/// copy boundary tendencies without staging complete fields through the host.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFieldLocation, SpecifiedBoundaryTendencies,
///     SpecifiedBoundaryTendencyKernels, SpecifiedBoundaryTendencyParameters,
///     SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 6)?;
/// let mut tendency = backend.create_field(shape, 0.0_f32)?;
/// let boundary_shape = GridShape::try_new(6, 2, 5)?;
/// let boundary = backend.create_field(boundary_shape, 0.25_f32)?;
/// let boundaries = SpecifiedBoundaryTendencies::new(
///     &boundary, &boundary, &boundary, &boundary,
/// );
/// let region = SpecifiedBoundaryUpdateRegion::try_new(
///     shape,
///     SpecifiedBoundaryFieldLocation::MassHalfLevel,
///     1..5,
///     1..5,
///     1..5,
///     1..5,
///     1..5,
///     1..5,
/// )?;
/// backend.assign_specified_boundary_tendencies(
///     &mut tendency,
///     boundaries,
///     SpecifiedBoundaryTendencyParameters::new(2, 1),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryTendencyKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Replaces contacted specified-zone tendencies with boundary-file values.
    ///
    /// Validation completes before mutation. Storage outside the active
    /// boundary traversal remains unchanged, including inactive tile storage.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid widths, shapes, vertical boundary
    /// storage, or backend execution failure.
    fn assign_specified_boundary_tendencies(
        &self,
        tendency: &mut Self::Field,
        boundary_tendencies: SpecifiedBoundaryTendencies<'_, Self::Field>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryTendencyResult<()>;
}
