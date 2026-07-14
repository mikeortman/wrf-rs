use std::sync::Mutex;

use wrf_compute::{ComputeBackend, CpuBackend, CpuField, GridShape};

use crate::{KesslerMicrophysicsError, KesslerMicrophysicsRegion, KesslerMicrophysicsResult};

use super::column_scratch::KesslerColumnScratch;

/// Reusable CPU scratch storage for Kessler sedimentation.
///
/// The production field matches WRF's tile-sized `prod` scratch. One vertical
/// terminal-velocity buffer is allocated per persistent CPU worker. Creation is
/// a setup operation; timestep execution does not allocate numerical scratch.
#[derive(Debug)]
pub struct CpuKesslerMicrophysicsWorkspace {
    shape: GridShape,
    production: CpuField<f32>,
    column_scratch_by_worker: Vec<Mutex<KesslerColumnScratch>>,
}

impl CpuKesslerMicrophysicsWorkspace {
    pub(super) fn try_new(
        backend: &CpuBackend,
        region: &KesslerMicrophysicsRegion,
    ) -> KesslerMicrophysicsResult<Self> {
        let shape = region.field_shape();
        let production = backend.create_field(shape, 0.0).map_err(|error| {
            KesslerMicrophysicsError::WorkspaceAllocationFailed {
                message: error.to_string(),
            }
        })?;
        let column_scratch_by_worker = (0..backend.worker_count())
            .map(|_| Mutex::new(KesslerColumnScratch::new(shape.bottom_top_points())))
            .collect();

        Ok(Self {
            shape,
            production,
            column_scratch_by_worker,
        })
    }

    /// Returns the field shape for which this workspace was allocated.
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    /// Returns bytes held by numerical scratch values, excluding allocator metadata.
    pub fn numeric_scratch_byte_count(&self) -> usize {
        let production_bytes = self.shape.point_count() * size_of::<f32>();
        let terminal_velocity_values =
            self.column_scratch_by_worker.len() * self.shape.bottom_top_points();
        production_bytes + terminal_velocity_values * size_of::<f32>()
    }

    pub(super) fn execution_parts(
        &mut self,
    ) -> (&mut CpuField<f32>, &[Mutex<KesslerColumnScratch>]) {
        (&mut self.production, &self.column_scratch_by_worker)
    }

    pub(super) fn production(&self) -> &CpuField<f32> {
        &self.production
    }
}
