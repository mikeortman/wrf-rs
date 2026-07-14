use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField};

use crate::{MoistureCoefficientRegion, MoistureCoefficientResult, MoistureSpecies};

use super::map_parallel_error;

pub(super) fn calculate(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    moisture: MoistureSpecies<'_, CpuField<f32>>,
    region: &MoistureCoefficientRegion,
) -> MoistureCoefficientResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    let (west_east, south_north, bottom_top) = region.south_north_output_ranges();

    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |line_index, output_row| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north.contains(&south_north_index)
                    && bottom_top.contains(&bottom_top_index)
                {
                    let totals = &mut output_row[west_east.clone()];
                    if moisture.is_empty() {
                        totals.fill(1.0);
                    } else {
                        totals.fill(0.0);
                        let current_row_start = line_index * west_east_points;
                        let south_line_index =
                            (south_north_index - 1) * bottom_top_points + bottom_top_index;
                        let south_row_start = south_line_index * west_east_points;
                        for species in moisture.active {
                            let values = species.values();
                            for (offset, total) in totals.iter_mut().enumerate() {
                                let west_east_index = west_east.start + offset;
                                *total = (*total + values[current_row_start + west_east_index])
                                    + values[south_row_start + west_east_index];
                            }
                        }
                        for total in totals {
                            *total = 1.0 / (1.0 + 0.5 * *total);
                        }
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}
