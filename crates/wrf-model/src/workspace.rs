use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_physics::{ArwMicrophysicsTrajectory, ArwMicrophysicsWorkspace};

use crate::{ArwModelError, ArwModelResult, ArwWorkspaceColumnField, ArwWorkspaceVolumeField};

/// Reusable nonrestart storage for one Registry-backed model instance.
pub struct ArwModelWorkspace {
    pub(crate) worker_count: usize,
    pub(crate) volume_fields: Vec<CpuField<f32>>,
    pub(crate) column_fields: Vec<CpuField<f32>>,
    pub(crate) microphysics_mass_fields: [CpuField<f32>; 5],
    pub(crate) microphysics_geopotential_fields: [CpuField<f32>; 2],
    pub(crate) microphysics_moisture_fields: Vec<CpuField<f32>>,
    pub(crate) microphysics: ArwMicrophysicsWorkspace,
    shape: GridShape,
    microphysics_shape: GridShape,
    moisture_field_count: usize,
}

impl ArwModelWorkspace {
    pub(crate) fn try_new(
        backend: &CpuBackend,
        shape: GridShape,
        microphysics_shape: GridShape,
        moisture_field_count: usize,
        microphysics_trajectory: &ArwMicrophysicsTrajectory,
    ) -> ArwModelResult<Self> {
        let volume_fields = (0..ArwWorkspaceVolumeField::COUNT)
            .map(|_| backend.create_field(shape, 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        let horizontal_shape = shape.horizontal_shape();
        let column_fields = (0..ArwWorkspaceColumnField::COUNT)
            .map(|_| backend.create_field(horizontal_shape, 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        let microphysics_mass_fields = [
            backend.create_field(microphysics_shape, 0.0)?,
            backend.create_field(microphysics_shape, 0.0)?,
            backend.create_field(microphysics_shape, 0.0)?,
            backend.create_field(microphysics_shape, 0.0)?,
            backend.create_field(microphysics_shape, 0.0)?,
        ];
        let microphysics_w_shape = GridShape::try_new(
            microphysics_shape.west_east_points(),
            microphysics_shape.south_north_points(),
            microphysics_shape
                .bottom_top_points()
                .checked_add(1)
                .ok_or(wrf_compute::ComputeError::GridPointCountOverflow)?,
        )?;
        let microphysics_geopotential_fields = [
            backend.create_field(microphysics_w_shape, 0.0)?,
            backend.create_field(microphysics_w_shape, 0.0)?,
        ];
        let microphysics_moisture_fields = (0..moisture_field_count)
            .map(|_| backend.create_field(microphysics_shape, 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            worker_count: backend.worker_count(),
            volume_fields,
            column_fields,
            microphysics_mass_fields,
            microphysics_geopotential_fields,
            microphysics_moisture_fields,
            microphysics: microphysics_trajectory.create_workspace(backend)?,
            shape,
            microphysics_shape,
            moisture_field_count,
        })
    }

    /// Returns the common dynamics storage shape.
    #[must_use]
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    pub(crate) const fn microphysics_shape(&self) -> GridShape {
        self.microphysics_shape
    }

    /// Borrows one typed three-dimensional workspace field.
    #[must_use]
    pub fn volume_field_values(&self, field: ArwWorkspaceVolumeField) -> &[f32] {
        self.volume_fields[field as usize].values()
    }

    /// Mutably borrows one three-dimensional workspace field for initialization.
    pub fn volume_field_values_mut(&mut self, field: ArwWorkspaceVolumeField) -> &mut [f32] {
        self.volume_fields[field as usize].values_mut()
    }

    /// Borrows one typed horizontal workspace field.
    #[must_use]
    pub fn column_field_values(&self, field: ArwWorkspaceColumnField) -> &[f32] {
        self.column_fields[field as usize].values()
    }

    /// Mutably borrows one horizontal workspace field for initialization.
    pub fn column_field_values_mut(&mut self, field: ArwWorkspaceColumnField) -> &mut [f32] {
        self.column_fields[field as usize].values_mut()
    }

    pub(crate) fn validate_role_counts(&self) -> ArwModelResult<()> {
        for (collection, actual, expected) in [
            (
                "volume workspace",
                self.volume_fields.len(),
                ArwWorkspaceVolumeField::COUNT,
            ),
            (
                "column workspace",
                self.column_fields.len(),
                ArwWorkspaceColumnField::COUNT,
            ),
            (
                "microphysics moisture adapter",
                self.microphysics_moisture_fields.len(),
                self.moisture_field_count,
            ),
        ] {
            if actual != expected {
                return Err(ArwModelError::InternalRoleCountMismatch { collection });
            }
        }
        Ok(())
    }
}
