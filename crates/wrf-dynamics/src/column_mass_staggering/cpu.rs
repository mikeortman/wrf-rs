use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use crate::column_mass_staggering::axis_boundary::ColumnMassStaggeringAxisBoundary;
use crate::{
    ColumnMassStaggeringError, ColumnMassStaggeringField, ColumnMassStaggeringKernels,
    ColumnMassStaggeringRegion, ColumnMassStaggeringResult,
};

impl ColumnMassStaggeringKernels for CpuBackend {
    type Field = CpuField<f32>;

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
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes produce complete non-empty rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

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
}
