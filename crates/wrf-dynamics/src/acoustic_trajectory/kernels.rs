use wrf_compute::FieldStorage;

use crate::{
    AcousticTrajectoryCoefficients, AcousticTrajectoryControls, AcousticTrajectoryDiagnostics,
    AcousticTrajectoryInputs, AcousticTrajectoryRegions, AcousticTrajectoryResult,
    AcousticTrajectorySavedState, AcousticTrajectoryTimeLevels, AcousticTrajectoryWorkspace,
};

/// Backend capability for one complete local WRF acoustic trajectory.
pub trait AcousticTrajectoryKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Runs preparation, initial diagnosis/coefficient construction, and every
    /// acoustic substep in WRF `solve_em.F` order.
    ///
    /// Every stage's structural contracts are checked before the first shared
    /// state mutation. Communication and external boundary operations are not
    /// hidden inside this local numerical capability.
    #[allow(clippy::too_many_arguments)]
    fn advance_acoustic_trajectory(
        &self,
        time_levels: AcousticTrajectoryTimeLevels<'_, Self::Field>,
        saved_state: AcousticTrajectorySavedState<'_, Self::Field>,
        diagnostics: AcousticTrajectoryDiagnostics<'_, Self::Field>,
        workspace: AcousticTrajectoryWorkspace<'_, Self::Field>,
        inputs: AcousticTrajectoryInputs<'_, Self::Field>,
        coefficients: AcousticTrajectoryCoefficients<'_>,
        controls: AcousticTrajectoryControls,
        regions: AcousticTrajectoryRegions<'_>,
    ) -> AcousticTrajectoryResult<()>;
}
