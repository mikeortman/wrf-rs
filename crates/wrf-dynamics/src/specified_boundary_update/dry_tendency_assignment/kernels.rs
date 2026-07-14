use wrf_compute::FieldStorage;

use super::{
    DryBoundaryTendencies, DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyRegion,
    DryBoundaryTendencyResult, DryBoundaryVerticalTendency,
};
use crate::{SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity};

/// Backend capability for WRF's complete dry boundary-tendency wrapper.
///
/// Implementations retain backend-native fields, allowing a future GPU backend
/// to assign every dry boundary without whole-field host staging.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(8, 8, 6)?;
/// let horizontal_shape = shape.horizontal_shape();
/// let region = DryBoundaryTendencyRegion::try_new(
///     shape, 1..7, 1..7, 1..5, 1..7, 1..7, 1..5,
/// )?;
/// let volume = |value| backend.create_field(shape, value);
/// let horizontal = |value| backend.create_field(horizontal_shape, value);
/// let mut outputs = [volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?];
/// let mut mass_output = horizontal(0.0)?;
/// let boundary_shape = GridShape::try_new(8, 2, 5)?;
/// let boundary = [
///     backend.create_field(boundary_shape, 0.1)?,
///     backend.create_field(boundary_shape, 0.1)?,
///     backend.create_field(boundary_shape, 0.1)?,
///     backend.create_field(boundary_shape, 0.1)?,
/// ];
/// let horizontal_boundary_shape = GridShape::try_new(8, 2, 1)?;
/// let horizontal_boundary = [
///     backend.create_field(horizontal_boundary_shape, 0.1)?,
///     backend.create_field(horizontal_boundary_shape, 0.1)?,
///     backend.create_field(horizontal_boundary_shape, 0.1)?,
///     backend.create_field(horizontal_boundary_shape, 0.1)?,
/// ];
/// let boundaries = SpecifiedBoundaryTendencies::new(
///     &boundary[0], &boundary[1], &boundary[2], &boundary[3],
/// );
/// let horizontal_boundaries = SpecifiedBoundaryTendencies::new(
///     &horizontal_boundary[0], &horizontal_boundary[1],
///     &horizontal_boundary[2], &horizontal_boundary[3],
/// );
/// let [u, v, ph, t] = &mut outputs;
/// backend.assign_dry_boundary_tendencies(
///     DryBoundaryTendencies::new(u, v, ph, t, &mut mass_output),
///     DryBoundaryTendencyBoundaryFields::new(
///         boundaries, boundaries, boundaries, boundaries, horizontal_boundaries,
///     ),
///     DryBoundaryVerticalTendency::Disabled,
///     SpecifiedBoundaryTendencyParameters::new(2, 1),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait DryBoundaryTendencyKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Assigns boundary-file tendencies in U, V, PH, T, MU, optional-W order.
    ///
    /// Structural validation for every active output and boundary completes
    /// before the first tendency changes.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid regions, widths, output shapes,
    /// boundary shapes, or backend execution failure.
    fn assign_dry_boundary_tendencies(
        &self,
        tendencies: DryBoundaryTendencies<'_, Self::Field>,
        boundaries: DryBoundaryTendencyBoundaryFields<'_, Self::Field>,
        vertical: DryBoundaryVerticalTendency<'_, Self::Field>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &DryBoundaryTendencyRegion,
    ) -> DryBoundaryTendencyResult<()>;
}
