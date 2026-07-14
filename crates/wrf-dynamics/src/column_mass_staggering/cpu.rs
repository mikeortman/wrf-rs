use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use crate::column_mass_staggering::axis_boundary::ColumnMassStaggeringAxisBoundary;
use crate::{
    ColumnMassStaggeringAxis, ColumnMassStaggeringError, ColumnMassStaggeringField,
    ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion,
    ColumnMassStaggeringResult,
};

impl ColumnMassStaggeringKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn calculate_full_column_mass(
        &self,
        full_mass: &mut Self::Field,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        region: &ColumnMassStaggeringRegion,
    ) -> ColumnMassStaggeringResult<()> {
        validate_full_mass_operation(full_mass, perturbation_mass, base_mass, region)?;

        let row_length = region.shape().west_east_points();
        let perturbation_values = perturbation_mass.values();
        let base_values = base_mass.values();
        let (west_east, south_north) = region.full_mass_ranges()?;
        self.try_for_each_output_block(
            full_mass.values_mut(),
            row_length,
            |south_north_index, output_row| {
                if south_north.contains(&south_north_index) {
                    let row_start = south_north_index * row_length;
                    for west_east_index in west_east.clone() {
                        let index = row_start + west_east_index;
                        output_row[west_east_index] =
                            base_values[index] + perturbation_values[index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }

    fn stagger_column_mass(
        &self,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
    ) -> ColumnMassStaggeringResult<()> {
        validate_shape(
            perturbation_mass,
            region,
            ColumnMassStaggeringField::PerturbationMass,
        )?;
        validate_shape(base_mass, region, ColumnMassStaggeringField::BaseMass)?;
        validate_shape(
            west_east_momentum_mass,
            region,
            ColumnMassStaggeringField::WestEastMomentumMass,
        )?;
        validate_shape(
            south_north_momentum_mass,
            region,
            ColumnMassStaggeringField::SouthNorthMomentumMass,
        )?;

        let row_length = region.shape().west_east_points();
        let perturbation_values = perturbation_mass.values();
        let base_values = base_mass.values();
        let output_west_east_range = region.west_east_momentum_west_east_range();
        let output_south_north_range = region.west_east_momentum_south_north_range();
        let west_east_boundary = region.west_east_boundary();
        self.try_for_each_output_block(
            west_east_momentum_mass.values_mut(),
            row_length,
            |south_north_index, output_row| {
                if output_south_north_range.contains(&south_north_index) {
                    stagger_west_east_momentum_row(
                        perturbation_values,
                        base_values,
                        output_row,
                        south_north_index * row_length,
                        output_west_east_range.clone(),
                        west_east_boundary,
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)?;

        let output_west_east_range = region.south_north_momentum_west_east_range();
        let output_south_north_range = region.south_north_momentum_south_north_range();
        let south_north_boundary = region.south_north_boundary();
        self.try_for_each_output_block(
            south_north_momentum_mass.values_mut(),
            row_length,
            |south_north_index, output_row| {
                if output_south_north_range.contains(&south_north_index) {
                    let row_start = south_north_index * row_length;
                    let is_lower_boundary_row = south_north_boundary.touches_lower()
                        && south_north_index == output_south_north_range.start;
                    let is_upper_boundary_row = south_north_boundary.touches_upper()
                        && south_north_index + 1 == output_south_north_range.end;
                    if is_lower_boundary_row {
                        copy_full_mass_row(
                            perturbation_values,
                            base_values,
                            output_row,
                            row_start,
                            output_west_east_range.clone(),
                        );
                    } else if is_upper_boundary_row {
                        copy_full_mass_row(
                            perturbation_values,
                            base_values,
                            output_row,
                            row_start - row_length,
                            output_west_east_range.clone(),
                        );
                    } else {
                        average_south_north_row(
                            perturbation_values,
                            base_values,
                            output_row,
                            row_start,
                            output_west_east_range.clone(),
                            row_length,
                        );
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }

    fn stagger_column_mass_for_big_step(
        &self,
        perturbation_mass: &Self::Field,
        base_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> ColumnMassStaggeringResult<()> {
        validate_shape(
            perturbation_mass,
            region,
            ColumnMassStaggeringField::PerturbationMass,
        )?;
        validate_shape(base_mass, region, ColumnMassStaggeringField::BaseMass)?;
        validate_big_step_outputs(
            west_east_momentum_mass,
            south_north_momentum_mass,
            region,
            periodicity,
        )?;

        let perturbation_values = perturbation_mass.values();
        let base_values = base_mass.values();
        stagger_big_step_mass(
            self,
            west_east_momentum_mass,
            south_north_momentum_mass,
            region,
            periodicity,
            |index, adjacent_index| {
                0.5 * (perturbation_values[index]
                    + perturbation_values[adjacent_index]
                    + base_values[index]
                    + base_values[adjacent_index])
            },
        )
    }

    fn stagger_full_column_mass_for_big_step(
        &self,
        full_mass: &Self::Field,
        west_east_momentum_mass: &mut Self::Field,
        south_north_momentum_mass: &mut Self::Field,
        region: &ColumnMassStaggeringRegion,
        periodicity: ColumnMassStaggeringPeriodicity,
    ) -> ColumnMassStaggeringResult<()> {
        validate_shape(full_mass, region, ColumnMassStaggeringField::FullMass)?;
        validate_big_step_outputs(
            west_east_momentum_mass,
            south_north_momentum_mass,
            region,
            periodicity,
        )?;

        let full_mass_values = full_mass.values();
        stagger_big_step_mass(
            self,
            west_east_momentum_mass,
            south_north_momentum_mass,
            region,
            periodicity,
            |index, adjacent_index| {
                0.5 * (full_mass_values[index] + full_mass_values[adjacent_index])
            },
        )
    }
}

pub(crate) fn validate_full_mass_operation(
    full_mass: &CpuField<f32>,
    perturbation_mass: &CpuField<f32>,
    base_mass: &CpuField<f32>,
    region: &ColumnMassStaggeringRegion,
) -> ColumnMassStaggeringResult<()> {
    validate_shape(full_mass, region, ColumnMassStaggeringField::FullMass)?;
    validate_shape(
        perturbation_mass,
        region,
        ColumnMassStaggeringField::PerturbationMass,
    )?;
    validate_shape(base_mass, region, ColumnMassStaggeringField::BaseMass)?;
    region.full_mass_ranges()?;
    Ok(())
}

pub(crate) fn validate_big_step_operation(
    perturbation_mass: &CpuField<f32>,
    base_mass: &CpuField<f32>,
    west_east_momentum_mass: &CpuField<f32>,
    south_north_momentum_mass: &CpuField<f32>,
    region: &ColumnMassStaggeringRegion,
    periodicity: ColumnMassStaggeringPeriodicity,
) -> ColumnMassStaggeringResult<()> {
    validate_shape(
        perturbation_mass,
        region,
        ColumnMassStaggeringField::PerturbationMass,
    )?;
    validate_shape(base_mass, region, ColumnMassStaggeringField::BaseMass)?;
    validate_big_step_outputs(
        west_east_momentum_mass,
        south_north_momentum_mass,
        region,
        periodicity,
    )
}

fn validate_big_step_outputs(
    west_east_momentum_mass: &CpuField<f32>,
    south_north_momentum_mass: &CpuField<f32>,
    region: &ColumnMassStaggeringRegion,
    periodicity: ColumnMassStaggeringPeriodicity,
) -> ColumnMassStaggeringResult<()> {
    validate_shape(
        west_east_momentum_mass,
        region,
        ColumnMassStaggeringField::WestEastMomentumMass,
    )?;
    validate_shape(
        south_north_momentum_mass,
        region,
        ColumnMassStaggeringField::SouthNorthMomentumMass,
    )?;
    validate_periodic_lower_halos(region, periodicity)
}

fn validate_periodic_lower_halos(
    region: &ColumnMassStaggeringRegion,
    periodicity: ColumnMassStaggeringPeriodicity,
) -> ColumnMassStaggeringResult<()> {
    if periodicity.is_west_east_periodic()
        && region.west_east_boundary().touches_lower()
        && region.west_east_momentum_west_east_range().start == 0
    {
        return Err(ColumnMassStaggeringError::PeriodicLowerHaloMissing {
            axis: ColumnMassStaggeringAxis::WestEast,
        });
    }
    if periodicity.is_south_north_periodic()
        && region.south_north_boundary().touches_lower()
        && region.south_north_momentum_south_north_range().start == 0
    {
        return Err(ColumnMassStaggeringError::PeriodicLowerHaloMissing {
            axis: ColumnMassStaggeringAxis::SouthNorth,
        });
    }
    Ok(())
}

fn stagger_big_step_mass<Average>(
    backend: &CpuBackend,
    west_east_momentum_mass: &mut CpuField<f32>,
    south_north_momentum_mass: &mut CpuField<f32>,
    region: &ColumnMassStaggeringRegion,
    periodicity: ColumnMassStaggeringPeriodicity,
    average: Average,
) -> ColumnMassStaggeringResult<()>
where
    Average: Fn(usize, usize) -> f32 + Send + Sync,
{
    let row_length = region.shape().west_east_points();
    let output_west_east_range = region.west_east_momentum_west_east_range();
    let output_south_north_range = region.west_east_momentum_south_north_range();
    let west_east_boundary = region.west_east_boundary();
    backend
        .try_for_each_output_block(
            west_east_momentum_mass.values_mut(),
            row_length,
            |south_north_index, output_row| {
                if output_south_north_range.contains(&south_north_index) {
                    stagger_big_step_west_east_row(
                        output_row,
                        south_north_index * row_length,
                        output_west_east_range.clone(),
                        west_east_boundary,
                        periodicity,
                        &average,
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)?;

    let output_south_north_range = region.south_north_momentum_south_north_range();
    backend
        .try_for_each_output_block(
            south_north_momentum_mass.values_mut(),
            row_length,
            |south_north_index, output_row| {
                if output_south_north_range.contains(&south_north_index) {
                    stagger_big_step_south_north_row(
                        output_row,
                        south_north_index,
                        region,
                        periodicity,
                        &average,
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn stagger_big_step_west_east_row<Average>(
    output_row: &mut [f32],
    row_start: usize,
    output_range: Range<usize>,
    boundary: ColumnMassStaggeringAxisBoundary,
    periodicity: ColumnMassStaggeringPeriodicity,
    average: &Average,
) where
    Average: Fn(usize, usize) -> f32,
{
    let average_start = output_range.start + usize::from(boundary.touches_lower());
    let average_end = output_range.end - usize::from(boundary.touches_upper());
    if boundary.touches_lower() {
        let index = row_start + output_range.start;
        let adjacent_index = if periodicity.is_west_east_periodic() {
            index - 1
        } else {
            index
        };
        output_row[output_range.start] = average(index, adjacent_index);
    }
    for (offset, output) in output_row[average_start..average_end]
        .iter_mut()
        .enumerate()
    {
        let index = row_start + average_start + offset;
        *output = average(index, index - 1);
    }
    if boundary.touches_upper() {
        let west_east_index = output_range.end - 1;
        let index = row_start + west_east_index - 1;
        let adjacent_index = if periodicity.is_west_east_periodic() {
            index + 1
        } else {
            index
        };
        output_row[west_east_index] = average(index, adjacent_index);
    }
}

fn stagger_big_step_south_north_row<Average>(
    output_row: &mut [f32],
    south_north_index: usize,
    region: &ColumnMassStaggeringRegion,
    periodicity: ColumnMassStaggeringPeriodicity,
    average: &Average,
) where
    Average: Fn(usize, usize) -> f32,
{
    let row_length = region.shape().west_east_points();
    let output_west_east_range = region.south_north_momentum_west_east_range();
    let output_south_north_range = region.south_north_momentum_south_north_range();
    let boundary = region.south_north_boundary();
    let row_start = south_north_index * row_length;
    let is_lower_boundary_row =
        boundary.touches_lower() && south_north_index == output_south_north_range.start;
    let is_upper_boundary_row =
        boundary.touches_upper() && south_north_index + 1 == output_south_north_range.end;
    let first_row_start = if is_upper_boundary_row {
        row_start - row_length
    } else {
        row_start
    };
    let adjacent_row_start = if is_lower_boundary_row {
        if periodicity.is_south_north_periodic() {
            row_start - row_length
        } else {
            row_start
        }
    } else if is_upper_boundary_row {
        if periodicity.is_south_north_periodic() {
            row_start
        } else {
            row_start - row_length
        }
    } else {
        row_start - row_length
    };
    let west_east_start = output_west_east_range.start;
    for (offset, output) in output_row[output_west_east_range].iter_mut().enumerate() {
        let west_east_index = west_east_start + offset;
        *output = average(
            first_row_start + west_east_index,
            adjacent_row_start + west_east_index,
        );
    }
}

fn stagger_west_east_momentum_row(
    perturbation_values: &[f32],
    base_values: &[f32],
    output_row: &mut [f32],
    row_start: usize,
    output_range: Range<usize>,
    boundary: ColumnMassStaggeringAxisBoundary,
) {
    let average_start = output_range.start + usize::from(boundary.touches_lower());
    let average_end = output_range.end - usize::from(boundary.touches_upper());

    if boundary.touches_lower() {
        let west_east_index = output_range.start;
        let index = row_start + west_east_index;
        output_row[west_east_index] = full_mass(perturbation_values, base_values, index);
    }
    for (offset, output) in output_row[average_start..average_end]
        .iter_mut()
        .enumerate()
    {
        let west_east_index = average_start + offset;
        let index = row_start + west_east_index;
        *output = average_full_mass(perturbation_values, base_values, index, index - 1);
    }
    if boundary.touches_upper() {
        let west_east_index = output_range.end - 1;
        let preceding_index = row_start + west_east_index - 1;
        output_row[west_east_index] = full_mass(perturbation_values, base_values, preceding_index);
    }
}

fn copy_full_mass_row(
    perturbation_values: &[f32],
    base_values: &[f32],
    output_row: &mut [f32],
    source_row_start: usize,
    output_west_east_range: Range<usize>,
) {
    let west_east_start = output_west_east_range.start;
    for (offset, output) in output_row[output_west_east_range].iter_mut().enumerate() {
        let west_east_index = west_east_start + offset;
        let source_index = source_row_start + west_east_index;
        *output = full_mass(perturbation_values, base_values, source_index);
    }
}

fn average_south_north_row(
    perturbation_values: &[f32],
    base_values: &[f32],
    output_row: &mut [f32],
    row_start: usize,
    output_west_east_range: Range<usize>,
    row_length: usize,
) {
    let west_east_start = output_west_east_range.start;
    for (offset, output) in output_row[output_west_east_range].iter_mut().enumerate() {
        let west_east_index = west_east_start + offset;
        let index = row_start + west_east_index;
        *output = average_full_mass(perturbation_values, base_values, index, index - row_length);
    }
}

fn full_mass(perturbation_values: &[f32], base_values: &[f32], index: usize) -> f32 {
    perturbation_values[index] + base_values[index]
}

fn average_full_mass(
    perturbation_values: &[f32],
    base_values: &[f32],
    index: usize,
    preceding_index: usize,
) -> f32 {
    0.5 * (perturbation_values[index]
        + perturbation_values[preceding_index]
        + base_values[index]
        + base_values[preceding_index])
}

fn validate_shape(
    field: &CpuField<f32>,
    region: &ColumnMassStaggeringRegion,
    field_role: ColumnMassStaggeringField,
) -> ColumnMassStaggeringResult<()> {
    if field.shape() != region.shape() {
        return Err(ColumnMassStaggeringError::FieldShapeMismatch { field: field_role });
    }
    Ok(())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> ColumnMassStaggeringError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => ColumnMassStaggeringError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("validated field shapes produce complete non-empty rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::test_support::{CorpusReader, ExpectedOutputReader};

    #[test]
    fn matches_all_upstream_fortran_boundary_paths_and_preserves_halos() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let shape = GridShape::try_new(6, 5, 1).unwrap();
        let mut perturbation_mass = backend.create_field(shape, 0.0_f32).unwrap();
        let mut base_mass = backend.create_field(shape, 0.0_f32).unwrap();
        initialize_mass_fields(&mut perturbation_mass, &mut base_mass);

        let cases = [
            ("interior", 0..5, 0..4),
            ("lower", 1..5, 1..4),
            ("upper", 0..4, 0..3),
            ("both", 1..4, 1..3),
        ];
        for (case_name, west_east_domain, south_north_domain) in cases {
            let mut west_east_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
            let mut south_north_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
            let region = ColumnMassStaggeringRegion::try_new(
                shape,
                west_east_domain,
                south_north_domain,
                1..5,
                1..4,
            )
            .unwrap();

            backend
                .stagger_column_mass(
                    &perturbation_mass,
                    &base_mass,
                    &mut west_east_momentum_mass,
                    &mut south_north_momentum_mass,
                    &region,
                )
                .unwrap();

            let (expected_west_east, expected_south_north) = parse_fortran_expected_bits(case_name);
            assert_eq!(
                field_bits(&west_east_momentum_mass),
                expected_west_east,
                "west-east output differs for {case_name} boundary path"
            );
            assert_eq!(
                field_bits(&south_north_momentum_mass),
                expected_south_north,
                "south-north output differs for {case_name} boundary path"
            );
        }
    }

    #[test]
    fn big_step_variants_match_all_upstream_periodic_and_boundary_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let shape = GridShape::try_new(6, 5, 1).unwrap();
        let mut perturbation_mass = backend.create_field(shape, 0.0_f32).unwrap();
        let mut base_mass = backend.create_field(shape, 0.0_f32).unwrap();
        initialize_mass_fields(&mut perturbation_mass, &mut base_mass);

        for case in big_step_cases() {
            let region = ColumnMassStaggeringRegion::try_new(
                shape,
                case.west_east_domain.clone(),
                case.south_north_domain.clone(),
                1..5,
                1..4,
            )
            .unwrap();
            let mut split_perturbation_mass = perturbation_mass.clone();
            let mut full_mass = backend.create_field(shape, 0.0_f32).unwrap();
            for ((full, perturbation), base) in full_mass
                .values_mut()
                .iter_mut()
                .zip(split_perturbation_mass.values())
                .zip(base_mass.values())
            {
                *full = *perturbation + *base;
            }
            if case.uses_extreme_value {
                let extreme_index = shape.west_east_points() + 1;
                split_perturbation_mass.values_mut()[extreme_index] = f32::MAX;
                full_mass.values_mut()[extreme_index] = f32::MAX;
            }

            let mut split_west_east = backend.create_field(shape, -999.0_f32).unwrap();
            let mut split_south_north = backend.create_field(shape, -999.0_f32).unwrap();
            backend
                .stagger_column_mass_for_big_step(
                    &split_perturbation_mass,
                    &base_mass,
                    &mut split_west_east,
                    &mut split_south_north,
                    &region,
                    case.periodicity,
                )
                .unwrap();
            assert_big_step_oracle(case.name, "split", &split_west_east, &split_south_north);

            let mut full_west_east = backend.create_field(shape, -999.0_f32).unwrap();
            let mut full_south_north = backend.create_field(shape, -999.0_f32).unwrap();
            backend
                .stagger_full_column_mass_for_big_step(
                    &full_mass,
                    &mut full_west_east,
                    &mut full_south_north,
                    &region,
                    case.periodicity,
                )
                .unwrap();
            assert_big_step_oracle(case.name, "full", &full_west_east, &full_south_north);

            if case.uses_extreme_value {
                let extreme_index = shape.west_east_points() + 1;
                assert!(split_west_east.values()[extreme_index].is_infinite());
                assert!(split_south_north.values()[extreme_index].is_infinite());
                assert!(full_west_east.values()[extreme_index].is_infinite());
                assert!(full_south_north.values()[extreme_index].is_infinite());
            }
        }
    }

    #[test]
    fn big_step_rejects_missing_periodic_lower_halo_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(4, 4, 1).unwrap();
        let perturbation_mass = backend.create_field(shape, 1.0_f32).unwrap();
        let base_mass = backend.create_field(shape, 2.0_f32).unwrap();
        let region = ColumnMassStaggeringRegion::try_new(shape, 0..3, 0..3, 0..4, 0..4).unwrap();

        for (periodicity, expected_axis) in [
            (
                ColumnMassStaggeringPeriodicity::WestEast,
                ColumnMassStaggeringAxis::WestEast,
            ),
            (
                ColumnMassStaggeringPeriodicity::SouthNorth,
                ColumnMassStaggeringAxis::SouthNorth,
            ),
        ] {
            let mut west_east = backend.create_field(shape, -7.0_f32).unwrap();
            let mut south_north = backend.create_field(shape, -7.0_f32).unwrap();
            assert_eq!(
                backend.stagger_column_mass_for_big_step(
                    &perturbation_mass,
                    &base_mass,
                    &mut west_east,
                    &mut south_north,
                    &region,
                    periodicity,
                ),
                Err(ColumnMassStaggeringError::PeriodicLowerHaloMissing {
                    axis: expected_axis,
                })
            );
            assert!(west_east.values().iter().all(|value| *value == -7.0));
            assert!(south_north.values().iter().all(|value| *value == -7.0));
        }
    }

    #[test]
    fn full_big_step_validates_input_shape_before_mutating_outputs() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(4, 4, 1).unwrap();
        let wrong_shape = GridShape::try_new(5, 4, 1).unwrap();
        let full_mass = backend.create_field(wrong_shape, 1.0_f32).unwrap();
        let mut west_east = backend.create_field(shape, -7.0_f32).unwrap();
        let mut south_north = backend.create_field(shape, -7.0_f32).unwrap();
        let region = ColumnMassStaggeringRegion::try_new(shape, 1..3, 1..3, 1..4, 1..4).unwrap();

        assert_eq!(
            backend.stagger_full_column_mass_for_big_step(
                &full_mass,
                &mut west_east,
                &mut south_north,
                &region,
                ColumnMassStaggeringPeriodicity::Both,
            ),
            Err(ColumnMassStaggeringError::FieldShapeMismatch {
                field: ColumnMassStaggeringField::FullMass,
            })
        );
        assert!(west_east.values().iter().all(|value| *value == -7.0));
        assert!(south_north.values().iter().all(|value| *value == -7.0));
    }

    #[test]
    fn periodic_big_step_is_bitwise_deterministic_across_worker_counts() {
        let serial_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let parallel_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let shape = GridShape::try_new(258, 129, 1).unwrap();
        let mut perturbation_mass = serial_backend.create_field(shape, 0.0_f32).unwrap();
        let mut base_mass = serial_backend.create_field(shape, 0.0_f32).unwrap();
        for (index, value) in perturbation_mass.values_mut().iter_mut().enumerate() {
            *value = index as f32 * 0.000_1 - 2.0;
        }
        for (index, value) in base_mass.values_mut().iter_mut().enumerate() {
            *value = 90_000.0 + index as f32 * 0.001;
        }
        let region =
            ColumnMassStaggeringRegion::try_new(shape, 1..257, 1..128, 1..258, 1..129).unwrap();
        let mut serial_west_east = serial_backend.create_field(shape, -1.0_f32).unwrap();
        let mut serial_south_north = serial_backend.create_field(shape, -1.0_f32).unwrap();
        let mut parallel_west_east = serial_west_east.clone();
        let mut parallel_south_north = serial_south_north.clone();

        serial_backend
            .stagger_column_mass_for_big_step(
                &perturbation_mass,
                &base_mass,
                &mut serial_west_east,
                &mut serial_south_north,
                &region,
                ColumnMassStaggeringPeriodicity::Both,
            )
            .unwrap();
        parallel_backend
            .stagger_column_mass_for_big_step(
                &perturbation_mass,
                &base_mass,
                &mut parallel_west_east,
                &mut parallel_south_north,
                &region,
                ColumnMassStaggeringPeriodicity::Both,
            )
            .unwrap();

        assert_eq!(parallel_west_east, serial_west_east);
        assert_eq!(parallel_south_north, serial_south_north);
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts_at_all_boundaries() {
        let serial_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let parallel_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let shape = GridShape::try_new(258, 129, 1).unwrap();
        let mut perturbation_mass = serial_backend.create_field(shape, 0.0_f32).unwrap();
        let mut base_mass = serial_backend.create_field(shape, 0.0_f32).unwrap();
        for (index, value) in perturbation_mass.values_mut().iter_mut().enumerate() {
            *value = index as f32 * 0.000_1 - 2.0;
        }
        for (index, value) in base_mass.values_mut().iter_mut().enumerate() {
            *value = 90_000.0 + index as f32 * 0.001;
        }
        let region =
            ColumnMassStaggeringRegion::try_new(shape, 0..257, 0..128, 0..258, 0..129).unwrap();
        let mut serial_west_east = serial_backend.create_field(shape, -1.0_f32).unwrap();
        let mut serial_south_north = serial_backend.create_field(shape, -1.0_f32).unwrap();
        let mut parallel_west_east = serial_west_east.clone();
        let mut parallel_south_north = serial_south_north.clone();

        serial_backend
            .stagger_column_mass(
                &perturbation_mass,
                &base_mass,
                &mut serial_west_east,
                &mut serial_south_north,
                &region,
            )
            .unwrap();
        parallel_backend
            .stagger_column_mass(
                &perturbation_mass,
                &base_mass,
                &mut parallel_west_east,
                &mut parallel_south_north,
                &region,
            )
            .unwrap();

        assert_eq!(parallel_west_east, serial_west_east);
        assert_eq!(parallel_south_north, serial_south_north);
    }

    #[test]
    fn rejects_a_mismatched_field_before_mutating_outputs() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(4, 4, 1).unwrap();
        let wrong_shape = GridShape::try_new(5, 4, 1).unwrap();
        let perturbation_mass = backend.create_field(wrong_shape, 1.0_f32).unwrap();
        let base_mass = backend.create_field(shape, 2.0_f32).unwrap();
        let mut west_east = backend.create_field(shape, -7.0_f32).unwrap();
        let mut south_north = backend.create_field(shape, -7.0_f32).unwrap();
        let region = ColumnMassStaggeringRegion::try_new(shape, 0..3, 0..3, 1..4, 1..4).unwrap();

        assert_eq!(
            backend.stagger_column_mass(
                &perturbation_mass,
                &base_mass,
                &mut west_east,
                &mut south_north,
                &region,
            ),
            Err(ColumnMassStaggeringError::FieldShapeMismatch {
                field: ColumnMassStaggeringField::PerturbationMass,
            })
        );
        assert!(west_east.values().iter().all(|value| *value == -7.0));
        assert!(south_north.values().iter().all(|value| *value == -7.0));
    }

    #[test]
    fn matches_seeded_randomized_fortran_corpus() {
        let backend = CpuBackend::try_new().unwrap();
        let mut corpus = CorpusReader::new(include_str!(
            "../../test-data/randomized-arw/column_mass_staggering.in"
        ));
        let mut expected = ExpectedOutputReader::new(include_str!(
            "../../test-data/randomized-arw/column_mass_staggering.out.correct"
        ));
        let case_count = corpus.read_usize("column-mass case count");

        for _ in 0..case_count {
            let seed = corpus.read_seed();
            let domain_west_east_start = corpus.read_i32("domain west-east start");
            let domain_west_east_end = corpus.read_i32("domain west-east end");
            let domain_south_north_start = corpus.read_i32("domain south-north start");
            let domain_south_north_end = corpus.read_i32("domain south-north end");
            let _domain_bottom_top_start = corpus.read_i32("domain bottom-top start");
            let _domain_bottom_top_end = corpus.read_i32("domain bottom-top end");
            let memory_west_east_start = corpus.read_i32("memory west-east start");
            let memory_west_east_end = corpus.read_i32("memory west-east end");
            let memory_south_north_start = corpus.read_i32("memory south-north start");
            let memory_south_north_end = corpus.read_i32("memory south-north end");
            let _memory_bottom_top_start = corpus.read_i32("memory bottom-top start");
            let _memory_bottom_top_end = corpus.read_i32("memory bottom-top end");
            let tile_west_east_start = corpus.read_i32("tile west-east start");
            let tile_west_east_end = corpus.read_i32("tile west-east end");
            let tile_south_north_start = corpus.read_i32("tile south-north start");
            let tile_south_north_end = corpus.read_i32("tile south-north end");
            let _tile_bottom_top_start = corpus.read_i32("tile bottom-top start");
            let _tile_bottom_top_end = corpus.read_i32("tile bottom-top end");
            let shape = GridShape::try_new(
                extent(memory_west_east_start, memory_west_east_end),
                extent(memory_south_north_start, memory_south_north_end),
                1,
            )
            .unwrap();
            let perturbation_mass = read_corpus_field(&backend, shape, &mut corpus);
            let base_mass = read_corpus_field(&backend, shape, &mut corpus);
            let mut west_east_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
            let mut south_north_momentum_mass = backend.create_field(shape, -999.0_f32).unwrap();
            let region = ColumnMassStaggeringRegion::try_new(
                shape,
                offset(domain_west_east_start, memory_west_east_start)
                    ..offset(domain_west_east_end, memory_west_east_start),
                offset(domain_south_north_start, memory_south_north_start)
                    ..offset(domain_south_north_end, memory_south_north_start),
                offset(tile_west_east_start, memory_west_east_start)
                    ..offset(tile_west_east_end + 1, memory_west_east_start),
                offset(tile_south_north_start, memory_south_north_start)
                    ..offset(tile_south_north_end + 1, memory_south_north_start),
            )
            .unwrap_or_else(|error| panic!("seed {seed}: invalid column-mass region: {error}"));

            backend
                .stagger_column_mass(
                    &perturbation_mass,
                    &base_mass,
                    &mut west_east_momentum_mass,
                    &mut south_north_momentum_mass,
                    &region,
                )
                .unwrap_or_else(|error| {
                    panic!("seed {seed}: column-mass execution failed: {error}")
                });
            for (value_index, actual_value) in
                west_east_momentum_mass.values().iter().copied().enumerate()
            {
                expected.assert_next(seed, "west_east_mass", value_index, actual_value);
            }
            for (value_index, actual_value) in south_north_momentum_mass
                .values()
                .iter()
                .copied()
                .enumerate()
            {
                expected.assert_next(seed, "south_north_mass", value_index, actual_value);
            }
        }

        corpus.finish();
        expected.finish();
    }

    fn initialize_mass_fields(
        perturbation_mass: &mut CpuField<f32>,
        base_mass: &mut CpuField<f32>,
    ) {
        let shape = perturbation_mass.shape();
        for south_north_index in 0..shape.south_north_points() {
            for west_east_index in 0..shape.west_east_points() {
                let index = south_north_index * shape.west_east_points() + west_east_index;
                perturbation_mass.values_mut()[index] =
                    west_east_index as f32 * 0.25 + south_north_index as f32 * 1.5 - 0.3;
                base_mass.values_mut()[index] =
                    100.0 + west_east_index as f32 * 0.5 - south_north_index as f32 * 0.75;
            }
        }
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }

    fn parse_fortran_expected_bits(case_name: &str) -> (Vec<u32>, Vec<u32>) {
        let mut west_east = Vec::new();
        let mut south_north = Vec::new();
        for line in include_str!("../../test-data/column_mass_staggering.out.correct").lines() {
            let mut columns = line.split_whitespace();
            if columns.next().unwrap() != case_name {
                continue;
            }
            let field = columns.next().unwrap();
            columns.next();
            columns.next();
            let bits = u32::from_str_radix(columns.next().unwrap(), 16).unwrap();
            match field {
                "west_east" => west_east.push(bits),
                "south_north" => south_north.push(bits),
                _ => unreachable!("oracle contains only known fields"),
            }
        }
        (west_east, south_north)
    }

    struct BigStepCase {
        name: &'static str,
        west_east_domain: Range<usize>,
        south_north_domain: Range<usize>,
        periodicity: ColumnMassStaggeringPeriodicity,
        uses_extreme_value: bool,
    }

    fn big_step_cases() -> [BigStepCase; 8] {
        [
            BigStepCase {
                name: "interior",
                west_east_domain: 0..5,
                south_north_domain: 0..4,
                periodicity: ColumnMassStaggeringPeriodicity::None,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "lower",
                west_east_domain: 1..5,
                south_north_domain: 1..4,
                periodicity: ColumnMassStaggeringPeriodicity::None,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "upper",
                west_east_domain: 0..4,
                south_north_domain: 0..3,
                periodicity: ColumnMassStaggeringPeriodicity::None,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "both",
                west_east_domain: 1..4,
                south_north_domain: 1..3,
                periodicity: ColumnMassStaggeringPeriodicity::None,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "periodic_x",
                west_east_domain: 1..4,
                south_north_domain: 1..3,
                periodicity: ColumnMassStaggeringPeriodicity::WestEast,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "periodic_y",
                west_east_domain: 1..4,
                south_north_domain: 1..3,
                periodicity: ColumnMassStaggeringPeriodicity::SouthNorth,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "periodic_xy",
                west_east_domain: 1..4,
                south_north_domain: 1..3,
                periodicity: ColumnMassStaggeringPeriodicity::Both,
                uses_extreme_value: false,
            },
            BigStepCase {
                name: "physical_expression",
                west_east_domain: 1..4,
                south_north_domain: 1..3,
                periodicity: ColumnMassStaggeringPeriodicity::None,
                uses_extreme_value: true,
            },
        ]
    }

    fn assert_big_step_oracle(
        case_name: &str,
        routine_name: &str,
        west_east: &CpuField<f32>,
        south_north: &CpuField<f32>,
    ) {
        let (expected_west_east, expected_south_north) =
            parse_big_step_fortran_expected_bits(case_name, routine_name);
        assert_eq!(
            field_bits(west_east),
            expected_west_east,
            "west-east {routine_name} output differs for {case_name}"
        );
        assert_eq!(
            field_bits(south_north),
            expected_south_north,
            "south-north {routine_name} output differs for {case_name}"
        );
    }

    fn parse_big_step_fortran_expected_bits(
        case_name: &str,
        routine_name: &str,
    ) -> (Vec<u32>, Vec<u32>) {
        let mut west_east = Vec::new();
        let mut south_north = Vec::new();
        for line in include_str!("../../test-data/periodic_column_mass.out.correct").lines() {
            let mut columns = line.split_whitespace();
            if columns.next().unwrap() != case_name || columns.next().unwrap() != routine_name {
                continue;
            }
            let field = columns.next().unwrap();
            columns.next();
            columns.next();
            let bits = u32::from_str_radix(columns.next().unwrap(), 16).unwrap();
            match field {
                "west_east" => west_east.push(bits),
                "south_north" => south_north.push(bits),
                _ => unreachable!("oracle contains only known fields"),
            }
        }
        assert_eq!(west_east.len(), 30, "incomplete west-east oracle case");
        assert_eq!(south_north.len(), 30, "incomplete south-north oracle case");
        (west_east, south_north)
    }

    fn read_corpus_field(
        backend: &CpuBackend,
        shape: GridShape,
        corpus: &mut CorpusReader<'_>,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for value in field.values_mut() {
            *value = corpus.read_f32("column-mass field value");
        }
        field
    }

    fn extent(start: i32, end: i32) -> usize {
        (end - start + 1) as usize
    }

    fn offset(index: i32, memory_start: i32) -> usize {
        (index - memory_start) as usize
    }
}
