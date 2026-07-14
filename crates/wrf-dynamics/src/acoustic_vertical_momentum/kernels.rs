use wrf_compute::FieldStorage;

use crate::{
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    AcousticVerticalInputs, AcousticVerticalParameters, AcousticVerticalRegion,
    AcousticVerticalResult, AcousticVerticalState, AcousticVerticalWorkspace,
    VerticalAcousticTopBoundary,
};

/// Backend capability for WRF's implicit acoustic vertical solve.
///
/// Implementations own their execution strategy and native field storage. The
/// CPU backend uses complete south-north planes as independent work units; a
/// future device backend can retain the same contract without host closures.
///
/// # Example
///
/// ```
/// use wrf_compute::{ComputeBackend, CpuBackend, GridShape};
/// use wrf_dynamics::*;
///
/// let backend = CpuBackend::try_with_worker_count(4)?;
/// let shape = GridShape::try_new(5, 5, 5)?;
/// let volume = |value| backend.create_field(shape, value).unwrap();
/// let horizontal = |value| backend.create_field(shape.horizontal_shape(), value).unwrap();
/// let mut state_fields = [volume(0.2), volume(20.0), volume(295.0)];
/// let mut workspace = volume(0.0);
/// let volume_inputs = (0..16).map(|_| volume(1.0)).collect::<Vec<_>>();
/// let horizontal_inputs = (0..6).map(|_| horizontal(10.0)).collect::<Vec<_>>();
/// let vertical: [Vec<f32>; 8] = std::array::from_fn(|_| vec![1.0; 5]);
/// let region = AcousticVerticalRegion::try_new(
///     shape, 1..4, 1..4, 0..4, 1..5, 1..5, 0..5,
/// )?;
/// let [vertical_momentum, geopotential, thermodynamics] = &mut state_fields;
/// backend.advance_acoustic_vertical_momentum(
///     AcousticVerticalState::new(vertical_momentum, geopotential, thermodynamics),
///     AcousticVerticalWorkspace::new(&mut workspace),
///     AcousticVerticalInputs::new(
///         AcousticVerticalMomentumInputs::new(
///             &volume_inputs[0], &volume_inputs[1], &volume_inputs[2],
///             &volume_inputs[3], &volume_inputs[4],
///         ),
///         AcousticVerticalMassInputs::new(
///             &horizontal_inputs[0], &horizontal_inputs[1], &horizontal_inputs[2],
///         ),
///         AcousticVerticalThermodynamicInputs::new(&volume_inputs[5], &volume_inputs[6]),
///         AcousticVerticalGeopotentialInputs::new(
///             &volume_inputs[7], &volume_inputs[8], &volume_inputs[9],
///         ),
///         AcousticVerticalMapFactors::new(
///             &horizontal_inputs[3], &horizontal_inputs[4], &horizontal_inputs[5],
///         ),
///         AcousticVerticalSolveInputs::new(
///             &volume_inputs[10], &volume_inputs[11], &volume_inputs[12],
///             &volume_inputs[13], &volume_inputs[14], &volume_inputs[15],
///         ),
///         AcousticVerticalLevelCoefficients::new(
///             &vertical[0], &vertical[1], &vertical[2], &vertical[3],
///             &vertical[4], &vertical[5], &vertical[6], &vertical[7],
///         ),
///     ),
///     AcousticVerticalParameters::new(0.002, 0.003, 0.4, 300.0, 0.1, [0.5, 0.3, 0.2]),
///     AcousticVerticalBoundaryPolicy::new(
///         AcousticVerticalLateralDomain::Global,
///         AcousticVerticalWestEastPeriodicity::Nonperiodic,
///     ),
///     VerticalAcousticTopBoundary::Nonrigid,
///     AcousticVerticalAdvection::StaggeredGeopotentialGradient,
///     AcousticVerticalDamping::Disabled,
///     &region,
/// )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait AcousticVerticalKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Advances vertical momentum, geopotential, and time-averaged thermodynamics.
    ///
    /// Shape, coefficient, range, and complete-column contracts are validated
    /// before any state mutation. `workspace` is caller-owned reusable storage
    /// and may contain unspecified values after either success or failure.
    #[allow(clippy::too_many_arguments)]
    fn advance_acoustic_vertical_momentum(
        &self,
        state: AcousticVerticalState<'_, Self::Field>,
        workspace: AcousticVerticalWorkspace<'_, Self::Field>,
        inputs: AcousticVerticalInputs<'_, Self::Field>,
        parameters: AcousticVerticalParameters,
        boundary_policy: AcousticVerticalBoundaryPolicy,
        top_boundary: VerticalAcousticTopBoundary,
        vertical_advection: AcousticVerticalAdvection,
        damping: AcousticVerticalDamping,
        region: &AcousticVerticalRegion,
    ) -> AcousticVerticalResult<()>;
}
