use wrf_compute::CpuField;

use crate::{ArwMicrophysicsState, ArwMicrophysicsWorkspace};

/// Immutable zero-copy view supplied at a trajectory observation point.
pub struct ArwMicrophysicsStageView<'a> {
    perturbation_potential_temperature: &'a CpuField<f32>,
    moisture_species_fields: &'a [CpuField<f32>],
    accumulated_precipitation: &'a CpuField<f32>,
    step_precipitation: &'a CpuField<f32>,
    workspace: &'a ArwMicrophysicsWorkspace,
}

impl<'a> ArwMicrophysicsStageView<'a> {
    pub(crate) fn new(
        state: &'a ArwMicrophysicsState<'_>,
        workspace: &'a ArwMicrophysicsWorkspace,
    ) -> Self {
        Self {
            perturbation_potential_temperature: &*state.perturbation_potential_temperature,
            moisture_species_fields: &*state.moisture_species_fields,
            accumulated_precipitation: &*state.accumulated_precipitation,
            step_precipitation: &*state.step_precipitation,
            workspace,
        }
    }

    /// Returns the ARW perturbation potential-temperature state.
    pub const fn perturbation_potential_temperature(&self) -> &CpuField<f32> {
        self.perturbation_potential_temperature
    }

    /// Returns moisture fields in resolved Registry dense order.
    pub const fn moisture_species_fields(&self) -> &[CpuField<f32>] {
        self.moisture_species_fields
    }

    /// Returns accumulated non-convective precipitation.
    pub const fn accumulated_precipitation(&self) -> &CpuField<f32> {
        self.accumulated_precipitation
    }

    /// Returns current-step non-convective precipitation.
    pub const fn step_precipitation(&self) -> &CpuField<f32> {
        self.step_precipitation
    }

    /// Returns full dry potential temperature passed through the scheme.
    pub const fn full_potential_temperature(&self) -> &CpuField<f32> {
        &self.workspace.full_potential_temperature
    }

    /// Returns dry-air density prepared from ARW inverse density.
    pub const fn dry_air_density(&self) -> &CpuField<f32> {
        &self.workspace.dry_air_density
    }

    /// Returns the prepared Exner function.
    pub const fn exner_function(&self) -> &CpuField<f32> {
        &self.workspace.exner_function
    }

    /// Returns mass-level geometric height in meters.
    pub const fn height(&self) -> &CpuField<f32> {
        &self.workspace.height
    }

    /// Returns W-level layer thickness in meters.
    pub const fn vertical_layer_thickness(&self) -> &CpuField<f32> {
        &self.workspace.vertical_layer_thickness
    }

    /// Returns WRF's stage-dependent `h_diabatic` buffer.
    ///
    /// It contains the pre-scheme full-theta snapshot at `Prepared` and
    /// `MicrophysicsApplied`, then K/s latent heating at `Finished`.
    pub const fn h_diabatic(&self) -> &CpuField<f32> {
        &self.workspace.h_diabatic
    }

    /// Returns WRF's stage-dependent `qv_diabatic` buffer.
    pub const fn qv_diabatic(&self) -> &CpuField<f32> {
        &self.workspace.qv_diabatic
    }

    /// Returns WRF's stage-dependent `qc_diabatic` buffer.
    pub const fn qc_diabatic(&self) -> &CpuField<f32> {
        &self.workspace.qc_diabatic
    }

    /// Returns WRF's `th_phy_m_t0` diagnostic.
    pub const fn perturbation_dry_potential_temperature(&self) -> &CpuField<f32> {
        &self.workspace.perturbation_dry_potential_temperature
    }
}
