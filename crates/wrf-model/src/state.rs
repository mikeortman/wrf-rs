use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};
use wrf_registry::RustDenseScalarIndex;

use crate::{
    ArwColumnField, ArwGeopotentialField, ArwMapField, ArwMassField, ArwModelResult,
    ArwRegistryBinding, ArwRestartVolumeField,
};

/// Restart-owned Registry state for the accepted ARW trajectory.
///
/// Ordinary mass and geopotential fields retain WRF's common padded
/// allocation. The active geopotential range includes the upper W point while
/// the active mass range ends one level below it. No numerical state is cloned
/// during stepping.
pub struct ArwModelState {
    pub(crate) mass_fields: [CpuField<f32>; ArwMassField::COUNT],
    pub(crate) geopotential_fields: [CpuField<f32>; ArwGeopotentialField::COUNT],
    pub(crate) column_fields: [CpuField<f32>; ArwColumnField::COUNT],
    pub(crate) restart_volume_fields: [CpuField<f32>; ArwRestartVolumeField::COUNT],
    pub(crate) map_fields: [CpuField<f32>; ArwMapField::COUNT],
    pub(crate) moisture_fields: Vec<CpuField<f32>>,
    mass_shape: GridShape,
    geopotential_shape: GridShape,
}

impl ArwModelState {
    /// Allocates every Registry-selected restart field once.
    ///
    /// # Errors
    ///
    /// Returns a backend allocation or shape-overflow failure.
    pub fn try_new(
        backend: &CpuBackend,
        binding: &ArwRegistryBinding,
        mass_shape: GridShape,
    ) -> ArwModelResult<Self> {
        let geopotential_shape = mass_shape;
        let horizontal_shape = mass_shape.horizontal_shape();
        let mass_fields = [
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
        ];
        let geopotential_fields = [
            backend.create_field(geopotential_shape, 0.0)?,
            backend.create_field(geopotential_shape, 0.0)?,
            backend.create_field(geopotential_shape, 0.0)?,
        ];
        let column_fields = [
            backend.create_field(horizontal_shape, 0.0)?,
            backend.create_field(horizontal_shape, 0.0)?,
            backend.create_field(horizontal_shape, 0.0)?,
            backend.create_field(horizontal_shape, 0.0)?,
            backend.create_field(horizontal_shape, 0.0)?,
        ];
        let restart_volume_fields = [
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
            backend.create_field(mass_shape, 0.0)?,
        ];
        let mut map_fields = [
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
            backend.create_field(horizontal_shape, 1.0)?,
        ];
        map_fields[ArwMapField::TerrainHeight as usize]
            .values_mut()
            .fill(0.0);
        let moisture_fields = binding
            .moisture_layout()
            .members()
            .iter()
            .map(|_| backend.create_field(mass_shape, 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            mass_fields,
            geopotential_fields,
            column_fields,
            restart_volume_fields,
            map_fields,
            moisture_fields,
            mass_shape,
            geopotential_shape,
        })
    }

    /// Returns the common padded mass-field shape.
    #[must_use]
    pub const fn mass_shape(&self) -> GridShape {
        self.mass_shape
    }

    /// Returns the explicit W-level geopotential shape.
    #[must_use]
    pub const fn geopotential_shape(&self) -> GridShape {
        self.geopotential_shape
    }

    /// Borrows values from one typed mass field.
    #[must_use]
    pub fn mass_field_values(&self, field: ArwMassField) -> &[f32] {
        self.mass_fields[field as usize].values()
    }

    /// Mutably borrows values from one typed mass field for initialization or restart loading.
    pub fn mass_field_values_mut(&mut self, field: ArwMassField) -> &mut [f32] {
        self.mass_fields[field as usize].values_mut()
    }

    /// Borrows values from one typed geopotential field.
    #[must_use]
    pub fn geopotential_field_values(&self, field: ArwGeopotentialField) -> &[f32] {
        self.geopotential_fields[field as usize].values()
    }

    /// Mutably borrows one geopotential field for initialization or restart loading.
    pub fn geopotential_field_values_mut(&mut self, field: ArwGeopotentialField) -> &mut [f32] {
        self.geopotential_fields[field as usize].values_mut()
    }

    /// Borrows values from one typed horizontal field.
    #[must_use]
    pub fn column_field_values(&self, field: ArwColumnField) -> &[f32] {
        self.column_fields[field as usize].values()
    }

    /// Mutably borrows one horizontal field for initialization or restart loading.
    pub fn column_field_values_mut(&mut self, field: ArwColumnField) -> &mut [f32] {
        self.column_fields[field as usize].values_mut()
    }

    /// Borrows one Registry restart diagnostic or tendency field.
    #[must_use]
    pub fn restart_volume_field_values(&self, field: ArwRestartVolumeField) -> &[f32] {
        self.restart_volume_fields[field as usize].values()
    }

    /// Mutably borrows one Registry restart diagnostic or tendency field.
    pub fn restart_volume_field_values_mut(&mut self, field: ArwRestartVolumeField) -> &mut [f32] {
        self.restart_volume_fields[field as usize].values_mut()
    }

    /// Borrows one Registry-owned map factor or terrain field.
    #[must_use]
    pub fn map_field_values(&self, field: ArwMapField) -> &[f32] {
        self.map_fields[field as usize].values()
    }

    /// Mutably borrows one Registry-owned map factor or terrain field.
    pub fn map_field_values_mut(&mut self, field: ArwMapField) -> &mut [f32] {
        self.map_fields[field as usize].values_mut()
    }

    /// Borrows one Registry-dense moisture field.
    #[must_use]
    pub fn moisture_field_values(&self, index: RustDenseScalarIndex) -> Option<&[f32]> {
        self.moisture_fields
            .get(index.as_usize())
            .map(|field| field.values())
    }

    /// Mutably borrows one Registry-dense moisture field.
    pub fn moisture_field_values_mut(&mut self, index: RustDenseScalarIndex) -> Option<&mut [f32]> {
        self.moisture_fields
            .get_mut(index.as_usize())
            .map(|field| field.values_mut())
    }

    /// Returns the number of active Registry-dense moisture fields.
    #[must_use]
    pub fn moisture_field_count(&self) -> usize {
        self.moisture_fields.len()
    }

    /// Borrows all moisture fields in resolved Registry dense order.
    #[must_use]
    pub fn moisture_fields(&self) -> &[CpuField<f32>] {
        &self.moisture_fields
    }
}
