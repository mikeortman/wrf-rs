use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, ParallelExecutionError};

use crate::{
    PositiveDefiniteError, PositiveDefiniteKernels, PositiveDefiniteResult,
    PositiveDefiniteSlabRegion,
};

const MINIMUM_SCALABLE_LINE_SUM: f32 = 1.0e-15;

impl PositiveDefiniteKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_positive_definite_sheet(
        &self,
        field: &mut Self::Field,
        line_totals: &[f32],
    ) -> PositiveDefiniteResult<()> {
        let shape = field.shape();
        if shape.bottom_top_points() != 1 {
            return Err(PositiveDefiniteError::SheetRequiresSingleVerticalLevel {
                bottom_top_points: shape.bottom_top_points(),
            });
        }
        if line_totals.len() != shape.south_north_points() {
            return Err(PositiveDefiniteError::LineTotalCountMismatch {
                line_count: shape.south_north_points(),
                line_total_count: line_totals.len(),
            });
        }

        self.try_for_each_output_block(
            field.values_mut(),
            shape.west_east_points(),
            |line_index, line| {
                correct_sheet_line(line, line_totals[line_index]);
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }

    fn apply_positive_definite_slab(
        &self,
        field: &mut Self::Field,
        region: &PositiveDefiniteSlabRegion,
    ) -> PositiveDefiniteResult<()> {
        let shape = field.shape();
        if region.shape() != shape {
            return Err(PositiveDefiniteError::SlabFieldShapeMismatch);
        }

        let bottom_top_points = shape.bottom_top_points();
        let west_east_range = region.west_east_range();
        let west_east_start = west_east_range.start;
        let west_east_end = west_east_range.end;
        let bottom_top_range = region.bottom_top_range();
        let south_north_range = region.south_north_range();
        self.try_for_each_output_block(
            field.values_mut(),
            shape.west_east_points(),
            |line_index, complete_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    correct_slab_line(&mut complete_line[west_east_start..west_east_end]);
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
    }
}

fn correct_sheet_line(line: &mut [f32], target_total: f32) {
    if !line.iter().any(|value| *value < 0.0) {
        return;
    }
    if target_total < 0.0 {
        line.fill(0.0);
        return;
    }

    let minimum_value = line
        .iter()
        .copied()
        .fold(f32::INFINITY, |minimum, value| minimum.min(value));
    for value in line.iter_mut() {
        *value -= minimum_value;
    }

    let corrected_sum = line.iter().copied().fold(0.0_f32, |sum, value| sum + value);
    if corrected_sum > MINIMUM_SCALABLE_LINE_SUM {
        let reciprocal_sum = 1.0 / corrected_sum;
        for value in line {
            *value = *value * target_total * reciprocal_sum;
        }
    } else {
        line.fill(0.0);
    }
}

fn correct_slab_line(line: &mut [f32]) {
    if !line.iter().any(|value| *value < 0.0) {
        return;
    }

    let original_total = line.iter().copied().fold(0.0_f32, |sum, value| sum + value);
    if original_total < 0.0 {
        line.fill(0.0);
        return;
    }

    let minimum_value = line
        .iter()
        .copied()
        .fold(f32::INFINITY, |minimum, value| minimum.min(value));
    for value in line.iter_mut() {
        *value -= minimum_value;
    }
    let corrected_sum = line.iter().copied().fold(0.0_f32, |sum, value| sum + value);
    let reciprocal_sum = 1.0 / corrected_sum;
    for value in line {
        *value = *value * original_total * reciprocal_sum;
    }
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> PositiveDefiniteError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => PositiveDefiniteError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("validated field shapes always produce complete non-empty lines")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::test_support::{CorpusReader, ExpectedOutputReader};

    fn create_sheet(
        backend: &CpuBackend,
        west_east_points: usize,
        south_north_points: usize,
        values: &[f32],
    ) -> CpuField<f32> {
        let shape = GridShape::try_new(west_east_points, south_north_points, 1).unwrap();
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        field.values_mut().copy_from_slice(values);
        field
    }

    #[test]
    fn sheet_matches_upstream_fortran_bit_patterns() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let cases = [
            ("unchanged", 4, 1, vec![1.0, 2.0, 0.0, 3.0], vec![6.0]),
            (
                "negative_total",
                4,
                1,
                vec![-1.0, 2.0, 3.0, 4.0],
                vec![-1.0],
            ),
            ("redistribute", 4, 1, vec![-1.0, 1.0, 2.0, 4.0], vec![10.0]),
            ("degenerate", 4, 1, vec![-1.0, -1.0, -1.0, -1.0], vec![4.0]),
            (
                "multiple_lines",
                3,
                4,
                vec![
                    1.0, 2.0, 3.0, -2.0, 1.0, 4.0, -1.0, 0.5, 0.25, -3.0, -3.0, -3.0,
                ],
                vec![6.0, 7.0, -1.0, 9.0],
            ),
            ("below_epsilon", 2, 1, vec![-1.0e-20, 0.0], vec![1.0]),
            ("signed_zero", 3, 1, vec![1.0, -0.0, 2.0], vec![99.0]),
            ("zero_total", 2, 1, vec![-1.0, 2.0], vec![0.0]),
            ("negative_zero_total", 2, 1, vec![-1.0, 2.0], vec![-0.0]),
        ];
        let expected_cases = parse_fortran_expected_bits();

        for (name, west_east_points, south_north_points, values, totals) in cases {
            let mut field = create_sheet(&backend, west_east_points, south_north_points, &values);

            backend
                .apply_positive_definite_sheet(&mut field, &totals)
                .unwrap();

            let actual_bits = field
                .values()
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>();
            assert_eq!(
                actual_bits, expected_cases[name],
                "Fortran parity case {name}"
            );
        }
    }

    #[test]
    fn sheet_rejects_incompatible_shape_and_totals() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let shape = GridShape::try_new(3, 2, 2).unwrap();
        let mut three_dimensional_field = backend.create_field(shape, 0.0_f32).unwrap();

        assert_eq!(
            backend.apply_positive_definite_sheet(&mut three_dimensional_field, &[0.0, 0.0]),
            Err(PositiveDefiniteError::SheetRequiresSingleVerticalLevel {
                bottom_top_points: 2,
            })
        );

        let mut sheet = create_sheet(&backend, 3, 2, &[0.0; 6]);
        assert_eq!(
            backend.apply_positive_definite_sheet(&mut sheet, &[0.0]),
            Err(PositiveDefiniteError::LineTotalCountMismatch {
                line_count: 2,
                line_total_count: 1,
            })
        );
    }

    #[test]
    fn sheet_is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let line_count = 1_024;
        let mut values = Vec::with_capacity(line_count * 5);
        let mut totals = Vec::with_capacity(line_count);
        for line_index in 0..line_count {
            let offset = line_index as f32 * 0.000_1;
            values.extend_from_slice(&[-1.0 - offset, 0.25, 0.5, 1.0, 2.0 + offset]);
            totals.push(4.0 + offset);
        }
        let mut serial_field = create_sheet(&single_worker_backend, 5, line_count, &values);
        let mut parallel_field = create_sheet(&four_worker_backend, 5, line_count, &values);

        single_worker_backend
            .apply_positive_definite_sheet(&mut serial_field, &totals)
            .unwrap();
        four_worker_backend
            .apply_positive_definite_sheet(&mut parallel_field, &totals)
            .unwrap();

        let serial_bits = serial_field
            .values()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        let parallel_bits = parallel_field
            .values()
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        assert_eq!(parallel_bits, serial_bits);
    }

    #[test]
    fn sheet_matches_seeded_randomized_fortran_corpus() {
        let backend = CpuBackend::try_new().unwrap();
        let mut corpus = CorpusReader::new(include_str!(
            "../../test-data/randomized-arw/positive_definite_sheet.in"
        ));
        let mut expected = ExpectedOutputReader::new(include_str!(
            "../../test-data/randomized-arw/positive_definite_sheet.out.correct"
        ));
        let case_count = corpus.read_usize("sheet case count");

        for _ in 0..case_count {
            let seed = corpus.read_seed();
            let west_east_points = corpus.read_usize("sheet west-east point count");
            let south_north_points = corpus.read_usize("sheet south-north point count");
            let line_totals = (0..south_north_points)
                .map(|_| corpus.read_f32("sheet line total"))
                .collect::<Vec<_>>();
            let value_count = west_east_points * south_north_points;
            let values = (0..value_count)
                .map(|_| corpus.read_f32("sheet field value"))
                .collect::<Vec<_>>();
            let mut field = create_sheet(&backend, west_east_points, south_north_points, &values);

            backend
                .apply_positive_definite_sheet(&mut field, &line_totals)
                .unwrap_or_else(|error| panic!("seed {seed}: sheet execution failed: {error}"));
            for (value_index, actual_value) in field.values().iter().copied().enumerate() {
                expected.assert_next(seed, "sheet", value_index, actual_value);
            }
        }

        corpus.finish();
        expected.finish();
    }

    #[test]
    fn slab_matches_upstream_fortran_boundaries_and_preserves_halos() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let shape = GridShape::try_new(6, 4, 4).unwrap();
        let mut field = backend.create_field(shape, 8.0_f32).unwrap();
        let active_lines = [
            (1, 1, [-1.0, 1.0, 2.0, 4.0]),
            (1, 2, [-1.0, -2.0, 0.0, 0.0]),
            (2, 1, [1.0, 2.0, 3.0, 4.0]),
            (2, 2, [-1.0, -1.0, -1.0, 4.0]),
        ];
        for (south_north_index, bottom_top_index, values) in active_lines {
            let line_start = (south_north_index * shape.bottom_top_points() + bottom_top_index)
                * shape.west_east_points();
            field.values_mut()[line_start + 1..line_start + 5].copy_from_slice(&values);
        }
        let region = PositiveDefiniteSlabRegion::try_new(shape, 1..5, 1..3, 1..3).unwrap();

        backend
            .apply_positive_definite_slab(&mut field, &region)
            .unwrap();

        assert_eq!(
            slab_oracle_selection(&field),
            parse_slab_fortran_expected_bits()
        );
    }

    #[test]
    fn slab_rejects_region_for_another_field_shape() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let region_shape = GridShape::try_new(4, 2, 2).unwrap();
        let field_shape = GridShape::try_new(5, 2, 2).unwrap();
        let region = PositiveDefiniteSlabRegion::try_new(region_shape, 0..4, 0..2, 0..2).unwrap();
        let mut field = backend.create_field(field_shape, 0.0_f32).unwrap();

        assert_eq!(
            backend.apply_positive_definite_slab(&mut field, &region),
            Err(PositiveDefiniteError::SlabFieldShapeMismatch)
        );
    }

    #[test]
    fn slab_matches_seeded_randomized_fortran_corpus() {
        let backend = CpuBackend::try_new().unwrap();
        let mut corpus = CorpusReader::new(include_str!(
            "../../test-data/randomized-arw/positive_definite_slab.in"
        ));
        let mut expected = ExpectedOutputReader::new(include_str!(
            "../../test-data/randomized-arw/positive_definite_slab.out.correct"
        ));
        let case_count = corpus.read_usize("slab case count");

        for _ in 0..case_count {
            let seed = corpus.read_seed();
            let domain_west_east_start = corpus.read_i32("slab domain west-east start");
            let domain_west_east_end = corpus.read_i32("slab domain west-east end");
            let _domain_south_north_start = corpus.read_i32("slab domain south-north start");
            let domain_south_north_end = corpus.read_i32("slab domain south-north end");
            let _domain_bottom_top_start = corpus.read_i32("slab domain bottom-top start");
            let _domain_bottom_top_end = corpus.read_i32("slab domain bottom-top end");
            let memory_west_east_start = corpus.read_i32("slab memory west-east start");
            let memory_west_east_end = corpus.read_i32("slab memory west-east end");
            let memory_south_north_start = corpus.read_i32("slab memory south-north start");
            let memory_south_north_end = corpus.read_i32("slab memory south-north end");
            let memory_bottom_top_start = corpus.read_i32("slab memory bottom-top start");
            let memory_bottom_top_end = corpus.read_i32("slab memory bottom-top end");
            let _tile_west_east_start = corpus.read_i32("slab tile west-east start");
            let _tile_west_east_end = corpus.read_i32("slab tile west-east end");
            let tile_south_north_start = corpus.read_i32("slab tile south-north start");
            let tile_south_north_end = corpus.read_i32("slab tile south-north end");
            let tile_bottom_top_start = corpus.read_i32("slab tile bottom-top start");
            let tile_bottom_top_end = corpus.read_i32("slab tile bottom-top end");
            let west_east_points = extent(memory_west_east_start, memory_west_east_end);
            let south_north_points = extent(memory_south_north_start, memory_south_north_end);
            let bottom_top_points = extent(memory_bottom_top_start, memory_bottom_top_end);
            let shape = GridShape::try_new(west_east_points, south_north_points, bottom_top_points)
                .unwrap();
            let value_count = shape.point_count();
            let values = (0..value_count)
                .map(|_| corpus.read_f32("slab field value"))
                .collect::<Vec<_>>();
            let mut field = backend.create_field(shape, 0.0_f32).unwrap();
            field.values_mut().copy_from_slice(&values);
            let region = PositiveDefiniteSlabRegion::try_new(
                shape,
                offset(domain_west_east_start, memory_west_east_start)
                    ..offset(domain_west_east_end, memory_west_east_start),
                offset(tile_bottom_top_start, memory_bottom_top_start)
                    ..offset(tile_bottom_top_end, memory_bottom_top_start),
                offset(tile_south_north_start, memory_south_north_start)
                    ..offset(
                        tile_south_north_end.min(domain_south_north_end - 1) + 1,
                        memory_south_north_start,
                    ),
            )
            .unwrap_or_else(|error| panic!("seed {seed}: invalid slab region: {error}"));

            backend
                .apply_positive_definite_slab(&mut field, &region)
                .unwrap_or_else(|error| panic!("seed {seed}: slab execution failed: {error}"));
            for (value_index, actual_value) in field.values().iter().copied().enumerate() {
                expected.assert_next(seed, "slab", value_index, actual_value);
            }
        }

        corpus.finish();
        expected.finish();
    }

    fn parse_fortran_expected_bits() -> std::collections::HashMap<&'static str, Vec<u32>> {
        include_str!("../../test-data/positive_definite_sheet.out.correct")
            .lines()
            .map(|line| {
                let mut columns = line.split_whitespace();
                let name = columns.next().unwrap();
                let bits = columns
                    .map(|hexadecimal| u32::from_str_radix(hexadecimal, 16).unwrap())
                    .collect();
                (name, bits)
            })
            .collect()
    }

    fn slab_oracle_selection(field: &CpuField<f32>) -> Vec<u32> {
        let shape = field.shape();
        let mut selected_values = Vec::new();
        for south_north_index in 1..3 {
            for bottom_top_index in 1..3 {
                let line_start = (south_north_index * shape.bottom_top_points() + bottom_top_index)
                    * shape.west_east_points();
                selected_values.extend_from_slice(&field.values()[line_start + 1..line_start + 5]);
            }
        }
        selected_values.extend(
            [0, 35, 43, 79]
                .into_iter()
                .map(|index| field.values()[index]),
        );
        selected_values.into_iter().map(f32::to_bits).collect()
    }

    fn parse_slab_fortran_expected_bits() -> Vec<u32> {
        include_str!("../../test-data/positive_definite_slab.out.correct")
            .split_whitespace()
            .skip(1)
            .map(|hexadecimal| u32::from_str_radix(hexadecimal, 16).unwrap())
            .collect()
    }

    fn extent(start: i32, end: i32) -> usize {
        (end - start + 1) as usize
    }

    fn offset(index: i32, memory_start: i32) -> usize {
        (index - memory_start) as usize
    }
}
