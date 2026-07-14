use wrf_compute::FieldStorage;

use super::{
    SpecifiedBoundaryFinalizationInputs, SpecifiedBoundaryFinalizationParameters,
    SpecifiedBoundaryFinalizationRegion, SpecifiedBoundaryFinalizationResult,
};
use crate::SpecifiedBoundaryWestEastPeriodicity;

/// Backend capability for forcing WRF state to interpolated specified boundaries.
///
/// Implementations retain native field storage so a future GPU backend can
/// normalize boundary state without host round trips.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFinalizationBoundaryFields,
///     SpecifiedBoundaryFinalizationFieldLocation, SpecifiedBoundaryFinalizationInputs,
///     SpecifiedBoundaryFinalizationKernels, SpecifiedBoundaryFinalizationParameters,
///     SpecifiedBoundaryFinalizationRegion, SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(6, 6, 6)?;
/// let mut field = backend.create_field(shape, 0.0_f32)?;
/// let horizontal = backend.create_field(shape.horizontal_shape(), 5.0_f32)?;
/// let map_factor = backend.create_field(shape.horizontal_shape(), 1.0_f32)?;
/// let boundary_shape = GridShape::try_new(6, 2, 5)?;
/// let boundary_value = backend.create_field(boundary_shape, 2.0_f32)?;
/// let boundary_tendency = backend.create_field(boundary_shape, 0.25_f32)?;
/// let boundary_values = SpecifiedBoundaryFinalizationBoundaryFields::new(
///     &boundary_value, &boundary_value, &boundary_value, &boundary_value,
/// );
/// let boundary_tendencies = SpecifiedBoundaryFinalizationBoundaryFields::new(
///     &boundary_tendency, &boundary_tendency, &boundary_tendency, &boundary_tendency,
/// );
/// let coefficients = vec![0.4_f32; shape.bottom_top_points()];
/// let offsets = vec![1.3_f32; shape.bottom_top_points()];
/// let region = SpecifiedBoundaryFinalizationRegion::try_new(
///     shape,
///     SpecifiedBoundaryFinalizationFieldLocation::VerticalMomentum,
///     1..5,
///     1..5,
///     1..5,
///     1..5,
///     1..5,
///     1..6,
/// )?;
/// backend.finalize_specified_boundary(
///     &mut field,
///     SpecifiedBoundaryFinalizationInputs::new(
///         &horizontal,
///         &map_factor,
///         &coefficients,
///         &offsets,
///         boundary_values,
///         boundary_tendencies,
///     ),
///     SpecifiedBoundaryFinalizationParameters::new(0.5, 2, 1),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryFinalizationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Replaces active specified-zone state with boundary-file values.
    ///
    /// Validation completes before the first mutation. A zero-width specified
    /// zone is an exact no-op after validation.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid shapes, coefficient lengths, boundary
    /// widths, region contracts, or backend execution failure.
    fn finalize_specified_boundary(
        &self,
        field: &mut Self::Field,
        inputs: SpecifiedBoundaryFinalizationInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryFinalizationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFinalizationRegion,
    ) -> SpecifiedBoundaryFinalizationResult<()>;
}
