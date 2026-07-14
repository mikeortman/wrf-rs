use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use super::super::{DryBoundaryRelaxationError, DryBoundaryRelaxationRegion};

pub(super) struct DryBoundaryMassWeightingCpuKernel;

pub(super) struct DryBoundaryMassWeightingInputs<'a> {
    pub(super) field: &'a CpuField<f32>,
    pub(super) full_column_mass: &'a CpuField<f32>,
    pub(super) multiplier: &'a [f32],
    pub(super) offset: &'a [f32],
    pub(super) bottom_top_end: usize,
    pub(super) region: &'a DryBoundaryRelaxationRegion,
}

impl DryBoundaryMassWeightingCpuKernel {
    pub(super) fn execute(
        backend: &CpuBackend,
        output: &mut CpuField<f32>,
        inputs: DryBoundaryMassWeightingInputs<'_>,
    ) -> Result<(), DryBoundaryRelaxationError> {
        let output_shape = output.shape();
        let field_shape = inputs.field.shape();
        let (west_east_range, south_north_range, bottom_top_range) =
            inputs.region.workspace_ranges();
        let plane_length = output_shape.west_east_points() * output_shape.bottom_top_points();

        backend
            .try_for_each_output_block::<_, Infallible, _>(
                output.values_mut(),
                plane_length,
                |local_south_north, output_plane| {
                    let south_north = south_north_range.start + local_south_north;
                    let active_bottom_top =
                        bottom_top_range.start..bottom_top_range.end.min(inputs.bottom_top_end);
                    for (local_bottom_top, bottom_top) in active_bottom_top.enumerate() {
                        let output_row_start = local_bottom_top * output_shape.west_east_points();
                        let input_row_start = field_shape.west_east_points()
                            * (bottom_top + field_shape.bottom_top_points() * south_north);
                        let mass_row_start = south_north * field_shape.west_east_points();
                        for (local_west_east, west_east) in west_east_range.clone().enumerate() {
                            let mass = inputs.full_column_mass.values()[mass_row_start + west_east];
                            let weight =
                                inputs.multiplier[bottom_top] * mass + inputs.offset[bottom_top];
                            output_plane[output_row_start + local_west_east] =
                                inputs.field.values()[input_row_start + west_east] * weight;
                        }
                    }
                    Ok(())
                },
            )
            .map_err(Self::map_parallel_error)
    }

    fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> DryBoundaryRelaxationError {
        match error {
            ParallelExecutionError::Kernel(infallible) => match infallible {},
            ParallelExecutionError::WorkerPanicked => DryBoundaryRelaxationError::WorkerPanicked,
            ParallelExecutionError::ZeroBlockLength
            | ParallelExecutionError::IncompleteOutputBlock { .. }
            | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
                DryBoundaryRelaxationError::SchedulerContractViolated
            }
        }
    }
}
