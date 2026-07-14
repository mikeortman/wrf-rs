use std::convert::Infallible;

use rayon::prelude::*;
use wrf_compute::{CpuBackend, ParallelExecutionError};

use crate::{
    KesslerMicrophysicsError, KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
    KesslerMicrophysicsResult,
};

use super::warm_rain_fields::KesslerWarmRainFields;
use super::warm_rain_row::KesslerWarmRainRow;

/// Executes pointwise cloud conversion, condensation, and rain evaporation.
pub(crate) struct KesslerWarmRainConversion;

impl KesslerWarmRainConversion {
    pub(crate) fn apply(
        backend: &CpuBackend,
        fields: KesslerWarmRainFields<'_>,
        parameters: KesslerMicrophysicsParameters,
        region: &KesslerMicrophysicsRegion,
    ) -> KesslerMicrophysicsResult<()> {
        let west_east_points = region.field_shape().west_east_points();
        let bottom_top_points = region.field_shape().bottom_top_points();
        let field_row_length = west_east_points * bottom_top_points;
        let south_north_range = region.south_north_range();
        let field_start = south_north_range.start * field_row_length;
        let field_end = south_north_range.end * field_row_length;

        let KesslerWarmRainFields {
            potential_temperature,
            water_vapor_mixing_ratio,
            cloud_water_mixing_ratio,
            rain_water_mixing_ratio,
            dry_air_density,
            exner_function,
            production,
        } = fields;
        let temperature_rows = &mut potential_temperature[field_start..field_end];
        let vapor_rows = &mut water_vapor_mixing_ratio[field_start..field_end];
        let cloud_rows = &mut cloud_water_mixing_ratio[field_start..field_end];
        let rain_rows = &mut rain_water_mixing_ratio[field_start..field_end];
        let density_rows = &dry_air_density[field_start..field_end];
        let exner_rows = &exner_function[field_start..field_end];
        let production_rows = &production[field_start..field_end];
        let west_east_range = region.west_east_range();
        let bottom_top_range = region.bottom_top_range();

        backend
            .try_execute_parallel(|| {
                temperature_rows
                    .par_chunks_mut(field_row_length)
                    .zip(vapor_rows.par_chunks_mut(field_row_length))
                    .zip(cloud_rows.par_chunks_mut(field_row_length))
                    .zip(rain_rows.par_chunks_mut(field_row_length))
                    .enumerate()
                    .for_each(
                        |(row_index, (((temperature_row, vapor_row), cloud_row), rain_row))| {
                            let row_offset = row_index * field_row_length;
                            let mut row = KesslerWarmRainRow::new(
                                temperature_row,
                                vapor_row,
                                cloud_row,
                                rain_row,
                                &density_rows[row_offset..row_offset + field_row_length],
                                &exner_rows[row_offset..row_offset + field_row_length],
                                &production_rows[row_offset..row_offset + field_row_length],
                                west_east_points,
                            );
                            row.apply(
                                west_east_range.clone(),
                                bottom_top_range.clone(),
                                parameters,
                            );
                        },
                    );
                Ok::<(), Infallible>(())
            })
            .map_err(map_parallel_error)
    }
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> KesslerMicrophysicsError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => KesslerMicrophysicsError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("Kessler uses only the generic CPU parallel scheduler")
        }
    }
}
