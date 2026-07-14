use std::sync::Mutex;

use rayon::prelude::*;
use wrf_compute::{CpuBackend, ParallelExecutionError};

use crate::{
    KesslerMicrophysicsError, KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
    KesslerMicrophysicsResult,
};

use super::sedimentation_fields::KesslerSedimentationFields;
use super::sedimentation_row::KesslerSedimentationRow;
use crate::microphysics::kessler::column_scratch::KesslerColumnScratch;

/// Executes the vertically coupled fallout phase over independent rows.
pub(crate) struct KesslerSedimentation;

impl KesslerSedimentation {
    pub(crate) fn apply(
        backend: &CpuBackend,
        fields: KesslerSedimentationFields<'_>,
        parameters: KesslerMicrophysicsParameters,
        region: &KesslerMicrophysicsRegion,
        column_scratch_by_worker: &[Mutex<KesslerColumnScratch>],
    ) -> KesslerMicrophysicsResult<()> {
        let west_east_points = region.field_shape().west_east_points();
        let bottom_top_points = region.field_shape().bottom_top_points();
        let field_row_length = west_east_points * bottom_top_points;
        let south_north_range = region.south_north_range();
        let field_start = south_north_range.start * field_row_length;
        let field_end = south_north_range.end * field_row_length;
        let precipitation_start = south_north_range.start * west_east_points;
        let precipitation_end = south_north_range.end * west_east_points;

        let KesslerSedimentationFields {
            rain_water_mixing_ratio,
            dry_air_density,
            height,
            vertical_layer_thickness,
            production,
            accumulated_precipitation,
            step_precipitation,
        } = fields;
        let rain_water_rows = &rain_water_mixing_ratio[field_start..field_end];
        let density_rows = &dry_air_density[field_start..field_end];
        let height_rows = &height[field_start..field_end];
        let thickness_rows = &vertical_layer_thickness[field_start..field_end];
        let production_rows = &mut production[field_start..field_end];
        let accumulated_rows =
            &mut accumulated_precipitation[precipitation_start..precipitation_end];
        let step_rows = &mut step_precipitation[precipitation_start..precipitation_end];
        let west_east_range = region.west_east_range();
        let bottom_top_range = region.bottom_top_range();

        backend
            .try_execute_parallel(|| {
                production_rows
                    .par_chunks_mut(field_row_length)
                    .zip(accumulated_rows.par_chunks_mut(west_east_points))
                    .zip(step_rows.par_chunks_mut(west_east_points))
                    .enumerate()
                    .try_for_each(
                        |(row_index, ((production_row, accumulated_row), step_row))| {
                            let worker_index = rayon::current_thread_index()
                                .ok_or(KesslerMicrophysicsError::WorkerIndexUnavailable)?;
                            let scratch = column_scratch_by_worker.get(worker_index).ok_or(
                                KesslerMicrophysicsError::WorkerIndexOutOfBounds {
                                    worker_index,
                                    worker_count: column_scratch_by_worker.len(),
                                },
                            )?;
                            let mut scratch = scratch
                                .lock()
                                .map_err(|_| KesslerMicrophysicsError::WorkspacePoisoned)?;
                            let row_offset = row_index * field_row_length;
                            let mut row = KesslerSedimentationRow::new(
                                &rain_water_rows[row_offset..row_offset + field_row_length],
                                &density_rows[row_offset..row_offset + field_row_length],
                                &height_rows[row_offset..row_offset + field_row_length],
                                &thickness_rows[row_offset..row_offset + field_row_length],
                                production_row,
                                accumulated_row,
                                step_row,
                                west_east_points,
                            );
                            row.apply(
                                west_east_range.clone(),
                                bottom_top_range.clone(),
                                scratch.terminal_velocity_mut(),
                                parameters,
                            )
                        },
                    )
            })
            .map_err(map_parallel_error)
    }
}

fn map_parallel_error(
    error: ParallelExecutionError<KesslerMicrophysicsError>,
) -> KesslerMicrophysicsError {
    match error {
        ParallelExecutionError::Kernel(error) => error,
        ParallelExecutionError::WorkerPanicked => KesslerMicrophysicsError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("Kessler uses only the generic CPU parallel scheduler")
        }
    }
}
