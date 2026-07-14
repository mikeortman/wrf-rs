use wrf_compute::FieldStorage;

use crate::{
    SpecifiedBoundaryFlowInputs, SpecifiedBoundaryFlowParameters, SpecifiedBoundaryFlowRegion,
    SpecifiedBoundaryFlowResult, SpecifiedBoundaryWestEastPeriodicity,
};

/// Backend capability for WRF flow-dependent scalar boundaries.
///
/// [`SpecifiedBoundaryFlowParameters::new`] selects zero inflow for the base
/// routine. [`SpecifiedBoundaryFlowParameters::with_inflow_policy`] also
/// reproduces WRF's constant-CCN and preserve-current variants without
/// duplicating traversal code.
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::{
///     SpecifiedBoundaryFlowInputs, SpecifiedBoundaryFlowKernels,
///     SpecifiedBoundaryFlowParameters, SpecifiedBoundaryFlowRegion,
///     SpecifiedBoundaryWestEastPeriodicity,
/// };
///
/// let backend = CpuBackend::try_with_worker_count(2)?;
/// let shape = GridShape::try_new(8, 8, 8)?;
/// let mut scalar = backend.create_field(shape, 0.5_f32)?;
/// let west_east_velocity = backend.create_field(shape, -1.0_f32)?;
/// let south_north_velocity = backend.create_field(shape, -1.0_f32)?;
/// let region = SpecifiedBoundaryFlowRegion::try_new(
///     shape,
///     1..7,
///     1..7,
///     1..7,
///     1..8,
///     1..8,
///     1..7,
/// )?;
/// backend.apply_flow_dependent_specified_boundary(
///     &mut scalar,
///     SpecifiedBoundaryFlowInputs::new(&west_east_velocity, &south_north_velocity),
///     SpecifiedBoundaryFlowParameters::new(2),
///     SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait SpecifiedBoundaryFlowKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Copies the nearest interior scalar at outflow and writes zero at inflow.
    ///
    /// Validation completes before mutation. A zero-width zone is an exact
    /// no-op, and storage outside contacted boundary ranges remains unchanged.
    /// As in WRF, execution starts at the tile's lower vertical bound and
    /// continues through the physical half-level domain top.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid shapes, missing staggered velocity
    /// neighbors, a zone without an independent interior source, or backend
    /// execution failure.
    fn apply_flow_dependent_specified_boundary(
        &self,
        scalar: &mut Self::Field,
        inputs: SpecifiedBoundaryFlowInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryFlowParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryFlowRegion,
    ) -> SpecifiedBoundaryFlowResult<()>;
}
