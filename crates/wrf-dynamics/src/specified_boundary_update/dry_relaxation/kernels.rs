use wrf_compute::FieldStorage;

use super::{
    DryBoundaryRelaxationBoundaryFields, DryBoundaryRelaxationMassCoefficients,
    DryBoundaryRelaxationRegion, DryBoundaryRelaxationResult, DryBoundaryRelaxationState,
    DryBoundaryRelaxationTendencies, DryBoundaryRelaxationWorkspace, DryBoundaryVerticalRelaxation,
};
use crate::{
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationParameters,
    SpecifiedBoundaryWestEastPeriodicity,
};

/// Backend capability for WRF's complete dry boundary-relaxation wrapper.
///
/// Implementations retain backend-native fields and one caller-owned workspace,
/// allowing a future GPU backend to avoid whole-field host staging.
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
/// let region = DryBoundaryRelaxationRegion::try_new(
///     shape, 1..7, 1..7, 1..5, 1..7, 1..7, 1..6,
/// )?;
/// let volume = |value| backend.create_field(shape, value);
/// let horizontal = |value| backend.create_field(horizontal_shape, value);
/// let state = [volume(1.0)?, volume(1.1)?, volume(1.2)?, volume(1.3)?, volume(1.4)?];
/// let perturbation_mass = horizontal(0.5)?;
/// let full_mass = horizontal(10.0)?;
/// let mut outputs = [volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?, volume(0.0)?];
/// let mut mass_output = horizontal(0.0)?;
/// let boundary_shape = GridShape::try_new(8, 5, 5)?;
/// let boundary = [
///     backend.create_field(boundary_shape, 2.0)?, backend.create_field(boundary_shape, 2.0)?,
///     backend.create_field(boundary_shape, 2.0)?, backend.create_field(boundary_shape, 2.0)?,
///     backend.create_field(boundary_shape, 0.1)?, backend.create_field(boundary_shape, 0.1)?,
///     backend.create_field(boundary_shape, 0.1)?, backend.create_field(boundary_shape, 0.1)?,
/// ];
/// let horizontal_boundary_shape = GridShape::try_new(8, 5, 1)?;
/// let horizontal_boundary = [
///     backend.create_field(horizontal_boundary_shape, 2.0)?, backend.create_field(horizontal_boundary_shape, 2.0)?,
///     backend.create_field(horizontal_boundary_shape, 2.0)?, backend.create_field(horizontal_boundary_shape, 2.0)?,
///     backend.create_field(horizontal_boundary_shape, 0.1)?, backend.create_field(horizontal_boundary_shape, 0.1)?,
///     backend.create_field(horizontal_boundary_shape, 0.1)?, backend.create_field(horizontal_boundary_shape, 0.1)?,
/// ];
/// let volume_boundary = DryBoundaryRelaxationBoundaryData::new(
///     SpecifiedBoundaryRelaxationBoundaryValues::new(
///         &boundary[0], &boundary[1], &boundary[2], &boundary[3],
///     ),
///     SpecifiedBoundaryTendencies::new(
///         &boundary[4], &boundary[5], &boundary[6], &boundary[7],
///     ),
/// );
/// let horizontal_boundary = DryBoundaryRelaxationBoundaryData::new(
///     SpecifiedBoundaryRelaxationBoundaryValues::new(
///         &horizontal_boundary[0], &horizontal_boundary[1],
///         &horizontal_boundary[2], &horizontal_boundary[3],
///     ),
///     SpecifiedBoundaryTendencies::new(
///         &horizontal_boundary[4], &horizontal_boundary[5],
///         &horizontal_boundary[6], &horizontal_boundary[7],
///     ),
/// );
/// let mut workspace = backend.create_field(region.workspace_shape(), 0.0)?;
/// let [u_tendency, v_tendency, ph_tendency, t_tendency, w_tendency] = &mut outputs;
/// backend.add_dry_boundary_relaxation_tendencies(
///     DryBoundaryRelaxationTendencies::new(
///         u_tendency, v_tendency, ph_tendency, t_tendency, &mut mass_output,
///     ),
///     DryBoundaryRelaxationState::new(
///         &state[0], &state[1], &state[2], &state[3], &perturbation_mass, &full_mass,
///     ),
///     DryBoundaryRelaxationBoundaryFields::new(
///         volume_boundary, volume_boundary, volume_boundary, volume_boundary,
///         horizontal_boundary,
///     ),
///     DryBoundaryVerticalRelaxation::Nested {
///         velocity: &state[4], tendency: w_tendency, boundary: volume_boundary,
///     },
///     DryBoundaryRelaxationWorkspace::new(&mut workspace),
///     DryBoundaryRelaxationMassCoefficients::new(
///         &[0.6; 6], &[0.4; 6], &[0.55; 6], &[0.45; 6],
///     ),
///     SpecifiedBoundaryRelaxationCoefficients::new(
///         &[0.0, 0.7, 0.5, 0.3, 0.0], &[0.0, 0.1, 0.08, 0.04, 0.0],
///     ),
///     SpecifiedBoundaryRelaxationParameters::new(5, 1, 4, 0.25),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait DryBoundaryRelaxationKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Mass-weights the required states and adds relaxation to dry tendencies.
    ///
    /// Structural validation for every active field completes before the first
    /// tendency changes. Calls retain WRF's U, V, geopotential, temperature,
    /// column-mass, then optional nested-W order.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid shapes, coefficients, regions,
    /// boundary arrays, stencil coverage, or backend execution failure.
    #[allow(clippy::too_many_arguments)]
    fn add_dry_boundary_relaxation_tendencies(
        &self,
        tendencies: DryBoundaryRelaxationTendencies<'_, Self::Field>,
        state: DryBoundaryRelaxationState<'_, Self::Field>,
        boundaries: DryBoundaryRelaxationBoundaryFields<'_, Self::Field>,
        vertical: DryBoundaryVerticalRelaxation<'_, Self::Field>,
        workspace: DryBoundaryRelaxationWorkspace<'_, Self::Field>,
        mass_coefficients: DryBoundaryRelaxationMassCoefficients<'_>,
        relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'_>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &DryBoundaryRelaxationRegion,
    ) -> DryBoundaryRelaxationResult<()>;
}
