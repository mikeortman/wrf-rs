use wrf_physics::ArwMicrophysicsStageView;

use crate::{ArwModelState, ArwModelWorkspace};

/// Observable boundaries in the accepted-stage ARW projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArwModelStage {
    /// The seven `rk_step_prep` dependencies completed.
    RungeKuttaPrepared,
    /// `rk_addtend_dry` completed.
    DryTendenciesAssembled,
    /// The complete accepted local acoustic trajectory completed.
    AcousticAdvanced,
    /// `calc_mu_uv_1` and `small_step_finish` reconstructed full state.
    AcousticFinalized,
    /// Full fields were prepared for the selected microphysics scheme.
    MicrophysicsPrepared,
    /// Kessler updated thermodynamics, moisture, and precipitation.
    MicrophysicsApplied,
    /// Kessler results were converted back to perturbation state and tendencies.
    MicrophysicsFinished,
}

/// Zero-copy state exposed at a model trajectory observation boundary.
pub enum ArwModelStageView<'a> {
    /// Registry state plus the complete reusable dynamics workspace.
    Dynamics {
        /// Restart-owned Registry state.
        state: &'a ArwModelState,
        /// Diagnostics, tendencies, maps, and adapter storage.
        workspace: &'a ArwModelWorkspace,
    },
    /// Existing zero-copy Kessler trajectory view.
    Microphysics(ArwMicrophysicsStageView<'a>),
}
