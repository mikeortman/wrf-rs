use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

use crate::{
    ArwMicrophysicsResult, CpuKesslerMicrophysicsWorkspace, MicrophysicsDriver,
    MicrophysicsDriverWorkspace,
};

/// Reusable diagnostics, snapshots, tendencies, and scheme scratch.
///
/// Kessler scratch is allocated per persistent CPU worker, so each workspace
/// remains bound to the worker count of the backend that created it.
#[derive(Debug)]
pub struct ArwMicrophysicsWorkspace {
    pub(crate) worker_count: usize,
    pub(crate) mass_shape: GridShape,
    pub(crate) full_potential_temperature: CpuField<f32>,
    pub(crate) dry_air_density: CpuField<f32>,
    pub(crate) exner_function: CpuField<f32>,
    pub(crate) height: CpuField<f32>,
    pub(crate) vertical_layer_thickness: CpuField<f32>,
    pub(crate) h_diabatic: CpuField<f32>,
    pub(crate) qv_diabatic: CpuField<f32>,
    pub(crate) qc_diabatic: CpuField<f32>,
    pub(crate) perturbation_dry_potential_temperature: CpuField<f32>,
    pub(crate) driver_workspace: MicrophysicsDriverWorkspace<CpuKesslerMicrophysicsWorkspace>,
}

impl ArwMicrophysicsWorkspace {
    pub(crate) fn try_new(
        backend: &CpuBackend,
        driver: &MicrophysicsDriver,
        mass_shape: GridShape,
    ) -> ArwMicrophysicsResult<Self> {
        Ok(Self {
            worker_count: backend.worker_count(),
            mass_shape,
            full_potential_temperature: backend.create_field(mass_shape, 0.0)?,
            dry_air_density: backend.create_field(mass_shape, 0.0)?,
            exner_function: backend.create_field(mass_shape, 0.0)?,
            height: backend.create_field(mass_shape, 0.0)?,
            vertical_layer_thickness: backend.create_field(mass_shape, 0.0)?,
            h_diabatic: backend.create_field(mass_shape, 0.0)?,
            qv_diabatic: backend.create_field(mass_shape, 0.0)?,
            qc_diabatic: backend.create_field(mass_shape, 0.0)?,
            perturbation_dry_potential_temperature: backend.create_field(mass_shape, 0.0)?,
            driver_workspace: driver.create_workspace(backend)?,
        })
    }

    /// Returns microphysics latent heating in K/s after a completed step.
    pub const fn latent_heating_tendency(&self) -> &CpuField<f32> {
        &self.h_diabatic
    }

    /// Returns the water-vapor microphysics tendency in kg/kg/s.
    pub const fn water_vapor_tendency(&self) -> &CpuField<f32> {
        &self.qv_diabatic
    }

    /// Returns the cloud-water microphysics tendency in kg/kg/s.
    pub const fn cloud_water_tendency(&self) -> &CpuField<f32> {
        &self.qc_diabatic
    }

    /// Returns WRF's dry perturbation-theta diagnostic `th_phy_m_t0`.
    pub const fn perturbation_dry_potential_temperature(&self) -> &CpuField<f32> {
        &self.perturbation_dry_potential_temperature
    }
}
