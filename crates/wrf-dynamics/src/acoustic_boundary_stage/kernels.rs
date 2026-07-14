use wrf_compute::FieldStorage;

use crate::{
    AcousticBoundaryStageControls, AcousticBoundaryStageRegions, AcousticBoundaryStageResult,
    AcousticTrajectoryCoefficients, AcousticTrajectoryDiagnostics, AcousticTrajectoryInputs,
    AcousticTrajectorySavedState, AcousticTrajectoryTimeLevels, AcousticTrajectoryWorkspace,
};

/// Backend capability for the complete local WRF acoustic boundary stage.
pub trait AcousticBoundaryStageKernels {
    /// Native single-precision field storage used by the backend.
    type Field: FieldStorage<f32>;

    /// Advances the pinned `solve_em.F` acoustic window in source call order.
    ///
    /// Every numerical, specified-boundary, and physical-boundary contract is
    /// checked before `small_step_prep` performs the first mutation. All fields
    /// and workspaces remain caller-owned borrowed storage.
    ///
    /// # Errors
    ///
    /// Returns a typed stage error for unsupported polar filtering, mismatched
    /// region roles or field shapes, incomplete coefficients, and backend
    /// worker failures. Validation failures leave every mutable field unchanged.
    #[allow(clippy::too_many_arguments)]
    fn advance_acoustic_boundary_stage(
        &self,
        time_levels: AcousticTrajectoryTimeLevels<'_, Self::Field>,
        saved_state: AcousticTrajectorySavedState<'_, Self::Field>,
        diagnostics: AcousticTrajectoryDiagnostics<'_, Self::Field>,
        workspace: AcousticTrajectoryWorkspace<'_, Self::Field>,
        inputs: AcousticTrajectoryInputs<'_, Self::Field>,
        coefficients: AcousticTrajectoryCoefficients<'_>,
        controls: AcousticBoundaryStageControls,
        regions: AcousticBoundaryStageRegions<'_>,
    ) -> AcousticBoundaryStageResult<()>;
}
