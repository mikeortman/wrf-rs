use wrf_compute::FieldStorage;

use super::{
    SpecifiedBoundaryRelaxationInputs, SpecifiedBoundaryRelaxationParameters,
    SpecifiedBoundaryRelaxationResult,
};
use crate::{SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity};

/// Backend capability for WRF specified-boundary relaxation forcing.
///
/// Implementations retain native field storage so a future GPU backend can
/// execute the same validated stencil without whole-field host staging.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFieldLocation, SpecifiedBoundaryRelaxationBoundaryValues,
///     SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationField,
///     SpecifiedBoundaryRelaxationInputs, SpecifiedBoundaryRelaxationKernels,
///     SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryTendencies,
///     SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(8, 8, 6)?;
/// let field = backend.create_field(shape, 1.0_f32)?;
/// let mut tendency = backend.create_field(shape, 0.0_f32)?;
/// let boundary_shape = GridShape::try_new(8, 5, 5)?;
/// let boundary_value = backend.create_field(boundary_shape, 2.0_f32)?;
/// let boundary_tendency = backend.create_field(boundary_shape, 0.25_f32)?;
/// let region = SpecifiedBoundaryUpdateRegion::try_new(
///     shape,
///     SpecifiedBoundaryFieldLocation::MassHalfLevel,
///     1..7,
///     1..7,
///     1..5,
///     1..7,
///     1..7,
///     1..5,
/// )?;
/// let forcing = [0.0, 0.7, 0.5, 0.3, 0.0];
/// let smoothing = [0.0, 0.1, 0.08, 0.04, 0.0];
/// backend.add_specified_boundary_relaxation_tendencies(
///     &mut tendency,
///     SpecifiedBoundaryRelaxationInputs::new(
///         SpecifiedBoundaryRelaxationField::new(&field, 0..8, 0..8, 0..6),
///         SpecifiedBoundaryRelaxationBoundaryValues::new(
///             &boundary_value,
///             &boundary_value,
///             &boundary_value,
///             &boundary_value,
///         ),
///         SpecifiedBoundaryTendencies::new(
///             &boundary_tendency,
///             &boundary_tendency,
///             &boundary_tendency,
///             &boundary_tendency,
///         ),
///         SpecifiedBoundaryRelaxationCoefficients::new(&forcing, &smoothing),
///     ),
///     SpecifiedBoundaryRelaxationParameters::new(5, 1, 4, 0.5),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryRelaxationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Adds five-point boundary-file forcing in the contacted relaxation zone.
    ///
    /// Validation completes before mutation. The fixed specified zone,
    /// untouched corners, inactive tiles, and non-contacted boundaries retain
    /// their prior tendencies.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid zone geometry, field-view coverage,
    /// array shapes, coefficient lengths, or backend execution failure.
    fn add_specified_boundary_relaxation_tendencies(
        &self,
        tendency: &mut Self::Field,
        inputs: SpecifiedBoundaryRelaxationInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryRelaxationResult<()>;
}
