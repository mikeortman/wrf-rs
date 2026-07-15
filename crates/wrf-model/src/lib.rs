//! Registry-backed ownership and accepted-stage ARW model trajectories.
//!
//! This crate is the integration boundary between build-time Registry
//! selection, backend-native model state, parity-tested dynamics, and selected
//! physical parameterizations. Its first trajectory is deliberately a
//! dependency-closed projection of accepted WRF v4.7.1 stages, not a complete
//! `solve_em` timestep or evidence of whole-model forecast parity.

#![forbid(unsafe_code)]

mod coefficients;
mod controls;
mod error;
mod field;
mod geometry;
mod model;
mod registry_binding;
mod stage;
mod state;
mod workspace;
mod workspace_column_field;
mod workspace_volume_field;

pub use coefficients::ArwModelCoefficients;
pub use controls::ArwModelControls;
pub use error::{ArwModelError, ArwModelResult, ArwRegistryField};
pub use field::{
    ArwColumnField, ArwGeopotentialField, ArwMapField, ArwMassField, ArwRestartVolumeField,
};
pub use geometry::ArwModelGeometry;
pub use model::RegistryBoundArwModel;
pub use registry_binding::ArwRegistryBinding;
pub use stage::{ArwModelStage, ArwModelStageView};
pub use state::ArwModelState;
pub use workspace::ArwModelWorkspace;
pub use workspace_column_field::ArwWorkspaceColumnField;
pub use workspace_volume_field::ArwWorkspaceVolumeField;
