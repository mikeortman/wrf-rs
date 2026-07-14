use std::convert::Infallible;

use pulp::{Arch, Simd, WithSimd};
use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    HeldSuarezDampingError, HeldSuarezDampingField, HeldSuarezDampingFields,
    HeldSuarezDampingKernels, HeldSuarezDampingRegion, HeldSuarezDampingResult,
};

use super::line_layout::{MomentumDampingInputSlices, MomentumDampingLayout};
use super::simd::damp_momentum_line;

#[cfg(test)]
use super::line_layout::linear_index;

impl HeldSuarezDampingKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_held_suarez_damping(
        &self,
        fields: HeldSuarezDampingFields<'_, Self::Field>,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()> {
        Arch::new().dispatch(ApplyHeldSuarezDamping {
            backend: self,
            fields,
            region,
        })
    }
}

struct ApplyHeldSuarezDamping<'backend, 'fields, 'region> {
    backend: &'backend CpuBackend,
    fields: HeldSuarezDampingFields<'fields, CpuField<f32>>,
    region: &'region HeldSuarezDampingRegion,
}

impl WithSimd for ApplyHeldSuarezDamping<'_, '_, '_> {
    type Output = HeldSuarezDampingResult<()>;

    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        apply_held_suarez_damping_with_simd(self.backend, self.fields, self.region, simd)
    }
}

fn apply_held_suarez_damping_with_simd<S: Simd>(
    backend: &CpuBackend,
    fields: HeldSuarezDampingFields<'_, CpuField<f32>>,
    region: &HeldSuarezDampingRegion,
    simd: S,
) -> HeldSuarezDampingResult<()> {
    let HeldSuarezDampingFields {
        west_east_momentum_tendency,
        south_north_momentum_tendency,
        west_east_momentum,
        south_north_momentum,
        perturbation_pressure,
        base_pressure,
    } = fields;
    let expected_shape = region.shape();
    validate_field_shape(
        west_east_momentum_tendency,
        HeldSuarezDampingField::WestEastMomentumTendency,
        expected_shape,
    )?;
    validate_field_shape(
        south_north_momentum_tendency,
        HeldSuarezDampingField::SouthNorthMomentumTendency,
        expected_shape,
    )?;
    validate_field_shape(
        west_east_momentum,
        HeldSuarezDampingField::WestEastMomentum,
        expected_shape,
    )?;
    validate_field_shape(
        south_north_momentum,
        HeldSuarezDampingField::SouthNorthMomentum,
        expected_shape,
    )?;
    validate_field_shape(
        perturbation_pressure,
        HeldSuarezDampingField::PerturbationPressure,
        expected_shape,
    )?;
    validate_field_shape(
        base_pressure,
        HeldSuarezDampingField::BasePressure,
        expected_shape,
    )?;

    let west_east_points = expected_shape.west_east_points();
    let bottom_top_points = expected_shape.bottom_top_points();
    let bottom_top_range = region.bottom_top_range();
    let input_slices = MomentumDampingInputSlices::new(
        west_east_momentum.values(),
        south_north_momentum.values(),
        perturbation_pressure.values(),
        base_pressure.values(),
        MomentumDampingLayout {
            west_east_points,
            bottom_top_points,
            west_east_range: region.west_east_range(),
            surface_level: region.surface_level(),
        },
    );

    let south_north_range = region.south_north_momentum_south_north_range();
    backend
        .try_for_each_output_block(
            south_north_momentum_tendency.values_mut(),
            west_east_points,
            |line_index, tendency_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    damp_momentum_line(
                        simd,
                        input_slices.south_north_momentum_line(
                            tendency_line,
                            bottom_top_index,
                            south_north_index,
                        ),
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)?;

    let south_north_range = region.west_east_momentum_south_north_range();
    backend
        .try_for_each_output_block(
            west_east_momentum_tendency.values_mut(),
            west_east_points,
            |line_index, tendency_line| {
                let south_north_index = line_index / bottom_top_points;
                let bottom_top_index = line_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    damp_momentum_line(
                        simd,
                        input_slices.west_east_momentum_line(
                            tendency_line,
                            bottom_top_index,
                            south_north_index,
                        ),
                    );
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn validate_field_shape(
    field: &CpuField<f32>,
    field_name: HeldSuarezDampingField,
    expected: GridShape,
) -> HeldSuarezDampingResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(HeldSuarezDampingError::FieldShapeMismatch {
            field: field_name,
            expected,
            actual,
        });
    }
    Ok(())
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> HeldSuarezDampingError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => HeldSuarezDampingError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. } => {
            unreachable!("validated field shapes always produce complete non-empty lines")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::test_support::{CorpusReader, ExpectedOutputReader};

    #[test]
    fn matches_upstream_fortran_boundary_and_active_point_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let (mut fields, region) = create_fortran_fixture(&backend);

        apply_fixture(&backend, &mut fields, &region).unwrap();

        let actual_bits = selected_fortran_fixture_bits(&fields);
        assert_eq!(actual_bits, expected_fortran_bits());
    }

    #[test]
    fn is_bitwise_deterministic_across_worker_counts() {
        let single_worker_backend = CpuBackend::try_with_worker_count(1).unwrap();
        let four_worker_backend = CpuBackend::try_with_worker_count(4).unwrap();
        let (mut single_worker_fields, region) = create_fortran_fixture(&single_worker_backend);
        let (mut four_worker_fields, _) = create_fortran_fixture(&four_worker_backend);

        apply_fixture(&single_worker_backend, &mut single_worker_fields, &region).unwrap();
        apply_fixture(&four_worker_backend, &mut four_worker_fields, &region).unwrap();

        assert_eq!(
            single_worker_fields.west_east_momentum_tendency.values(),
            four_worker_fields.west_east_momentum_tendency.values()
        );
        assert_eq!(
            single_worker_fields.south_north_momentum_tendency.values(),
            four_worker_fields.south_north_momentum_tendency.values()
        );
    }

    #[test]
    fn rejects_a_field_shape_mismatch_before_mutating_outputs() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let (mut fields, region) = create_fortran_fixture(&backend);
        let original_west_east_tendency = fields.west_east_momentum_tendency.values().to_vec();
        let wrong_shape = GridShape::try_new(5, 6, 4).unwrap();
        fields.base_pressure = backend.create_field(wrong_shape, 0.0).unwrap();

        let HeldSuarezFixture {
            ref mut west_east_momentum_tendency,
            ref mut south_north_momentum_tendency,
            ref west_east_momentum,
            ref south_north_momentum,
            ref perturbation_pressure,
            ref base_pressure,
        } = fields;
        let result = backend.apply_held_suarez_damping(
            HeldSuarezDampingFields::new(
                west_east_momentum_tendency,
                south_north_momentum_tendency,
                west_east_momentum,
                south_north_momentum,
                perturbation_pressure,
                base_pressure,
            ),
            &region,
        );

        assert_eq!(
            result,
            Err(HeldSuarezDampingError::FieldShapeMismatch {
                field: HeldSuarezDampingField::BasePressure,
                expected: region.shape(),
                actual: wrong_shape,
            })
        );
        assert_eq!(
            west_east_momentum_tendency.values(),
            original_west_east_tendency
        );
    }

    #[test]
    fn matches_seeded_randomized_fortran_corpus() {
        let backend = CpuBackend::try_new().unwrap();
        let mut corpus = CorpusReader::new(include_str!(
            "../../test-data/randomized-arw/held_suarez.in"
        ));
        let mut expected = ExpectedOutputReader::new(include_str!(
            "../../test-data/randomized-arw/held_suarez.out.correct"
        ));
        let case_count = corpus.read_usize("Held-Suarez case count");

        for _ in 0..case_count {
            let seed = corpus.read_seed();
            let _domain_west_east_start = corpus.read_i32("domain west-east start");
            let _domain_west_east_end = corpus.read_i32("domain west-east end");
            let domain_south_north_start = corpus.read_i32("domain south-north start");
            let domain_south_north_end = corpus.read_i32("domain south-north end");
            let _domain_bottom_top_start = corpus.read_i32("domain bottom-top start");
            let domain_bottom_top_end = corpus.read_i32("domain bottom-top end");
            let memory_west_east_start = corpus.read_i32("memory west-east start");
            let memory_west_east_end = corpus.read_i32("memory west-east end");
            let memory_south_north_start = corpus.read_i32("memory south-north start");
            let memory_south_north_end = corpus.read_i32("memory south-north end");
            let memory_bottom_top_start = corpus.read_i32("memory bottom-top start");
            let memory_bottom_top_end = corpus.read_i32("memory bottom-top end");
            let tile_west_east_start = corpus.read_i32("tile west-east start");
            let tile_west_east_end = corpus.read_i32("tile west-east end");
            let tile_south_north_start = corpus.read_i32("tile south-north start");
            let tile_south_north_end = corpus.read_i32("tile south-north end");
            let tile_bottom_top_start = corpus.read_i32("tile bottom-top start");
            let tile_bottom_top_end = corpus.read_i32("tile bottom-top end");
            let shape = GridShape::try_new(
                extent(memory_west_east_start, memory_west_east_end),
                extent(memory_south_north_start, memory_south_north_end),
                extent(memory_bottom_top_start, memory_bottom_top_end),
            )
            .unwrap();
            let mut fields = HeldSuarezFixture {
                west_east_momentum_tendency: read_corpus_field(&backend, shape, &mut corpus),
                south_north_momentum_tendency: read_corpus_field(&backend, shape, &mut corpus),
                west_east_momentum: read_corpus_field(&backend, shape, &mut corpus),
                south_north_momentum: read_corpus_field(&backend, shape, &mut corpus),
                perturbation_pressure: read_corpus_field(&backend, shape, &mut corpus),
                base_pressure: read_corpus_field(&backend, shape, &mut corpus),
            };
            let active_bottom_top_end = tile_bottom_top_end.min(domain_bottom_top_end - 1);
            let active_south_north_end = tile_south_north_end.min(domain_south_north_end - 1);
            let region = HeldSuarezDampingRegion::try_new(
                shape,
                offset(tile_west_east_start, memory_west_east_start)
                    ..offset(tile_west_east_end + 1, memory_west_east_start),
                offset(tile_bottom_top_start, memory_bottom_top_start)
                    ..offset(active_bottom_top_end + 1, memory_bottom_top_start),
                offset(tile_south_north_start, memory_south_north_start)
                    ..offset(active_south_north_end + 1, memory_south_north_start),
                offset(
                    (domain_south_north_start + 1).max(tile_south_north_start),
                    memory_south_north_start,
                )..offset(active_south_north_end + 1, memory_south_north_start),
                offset(1, memory_bottom_top_start),
            )
            .unwrap_or_else(|error| panic!("seed {seed}: invalid Held-Suarez region: {error}"));

            apply_fixture(&backend, &mut fields, &region).unwrap_or_else(|error| {
                panic!("seed {seed}: Held-Suarez execution failed: {error}")
            });
            for (value_index, actual_value) in fields
                .west_east_momentum_tendency
                .values()
                .iter()
                .copied()
                .enumerate()
            {
                expected.assert_next(seed, "west_east_tendency", value_index, actual_value);
            }
            for (value_index, actual_value) in fields
                .south_north_momentum_tendency
                .values()
                .iter()
                .copied()
                .enumerate()
            {
                expected.assert_next(seed, "south_north_tendency", value_index, actual_value);
            }
        }

        corpus.finish();
        expected.finish();
    }

    struct HeldSuarezFixture {
        west_east_momentum_tendency: CpuField<f32>,
        south_north_momentum_tendency: CpuField<f32>,
        west_east_momentum: CpuField<f32>,
        south_north_momentum: CpuField<f32>,
        perturbation_pressure: CpuField<f32>,
        base_pressure: CpuField<f32>,
    }

    fn create_fortran_fixture(
        backend: &CpuBackend,
    ) -> (HeldSuarezFixture, HeldSuarezDampingRegion) {
        let shape = GridShape::try_new(6, 6, 4).unwrap();
        let mut fields = HeldSuarezFixture {
            west_east_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
            south_north_momentum_tendency: backend.create_field(shape, 0.0).unwrap(),
            west_east_momentum: backend.create_field(shape, 0.0).unwrap(),
            south_north_momentum: backend.create_field(shape, 0.0).unwrap(),
            perturbation_pressure: backend.create_field(shape, 0.0).unwrap(),
            base_pressure: backend.create_field(shape, 0.0).unwrap(),
        };

        for south_north_index in 0..shape.south_north_points() {
            let fortran_j = south_north_index as i32 - 1;
            for bottom_top_index in 0..shape.bottom_top_points() {
                let fortran_k = bottom_top_index as i32;
                for west_east_index in 0..shape.west_east_points() {
                    let fortran_i = west_east_index as i32 - 1;
                    let index = linear_index(
                        west_east_index,
                        bottom_top_index,
                        south_north_index,
                        shape.west_east_points(),
                        shape.bottom_top_points(),
                    );
                    fields.perturbation_pressure.values_mut()[index] =
                        (10 * fortran_i + 3 * fortran_j + 2 * fortran_k) as f32;
                    fields.base_pressure.values_mut()[index] = match bottom_top_index {
                        0 => 110_000.0,
                        1 => 100_000.0,
                        2 => 80_000.0,
                        _ => 50_000.0,
                    };
                    fields.west_east_momentum.values_mut()[index] =
                        (2 * fortran_i + 3 * fortran_k + 5 * fortran_j) as f32;
                    fields.south_north_momentum.values_mut()[index] =
                        (-fortran_i + 4 * fortran_k + 2 * fortran_j) as f32;
                    fields.west_east_momentum_tendency.values_mut()[index] =
                        (100 + fortran_i + 2 * fortran_k + 3 * fortran_j) as f32;
                    fields.south_north_momentum_tendency.values_mut()[index] =
                        (200 + 2 * fortran_i + fortran_k + 4 * fortran_j) as f32;
                }
            }
        }

        let region = HeldSuarezDampingRegion::try_new(shape, 1..5, 1..4, 1..5, 2..5, 1).unwrap();
        (fields, region)
    }

    fn apply_fixture(
        backend: &CpuBackend,
        fields: &mut HeldSuarezFixture,
        region: &HeldSuarezDampingRegion,
    ) -> HeldSuarezDampingResult<()> {
        backend.apply_held_suarez_damping(
            HeldSuarezDampingFields::new(
                &mut fields.west_east_momentum_tendency,
                &mut fields.south_north_momentum_tendency,
                &fields.west_east_momentum,
                &fields.south_north_momentum,
                &fields.perturbation_pressure,
                &fields.base_pressure,
            ),
            region,
        )
    }

    fn selected_fortran_fixture_bits(fields: &HeldSuarezFixture) -> Vec<u32> {
        let west_east_points = 6;
        let bottom_top_points = 4;
        let selected_west_east_tendency_points = [
            (0, 1, 0),
            (0, 2, 0),
            (3, 3, 3),
            (4, 1, 0),
            (0, 0, 0),
            (0, 1, 4),
            (2, 2, 2),
            (3, 1, 3),
        ];
        let selected_south_north_tendency_points = [
            (0, 1, 1),
            (0, 2, 1),
            (3, 3, 3),
            (0, 1, 0),
            (0, 1, 4),
            (4, 1, 1),
            (0, 0, 1),
            (2, 2, 2),
        ];
        selected_west_east_tendency_points
            .into_iter()
            .map(|point| fortran_point_index(point, west_east_points, bottom_top_points))
            .map(|index| fields.west_east_momentum_tendency.values()[index].to_bits())
            .chain(
                selected_south_north_tendency_points
                    .into_iter()
                    .map(|point| fortran_point_index(point, west_east_points, bottom_top_points))
                    .map(|index| fields.south_north_momentum_tendency.values()[index].to_bits()),
            )
            .collect()
    }

    fn fortran_point_index(
        (fortran_i, fortran_k, fortran_j): (i32, usize, i32),
        west_east_points: usize,
        bottom_top_points: usize,
    ) -> usize {
        linear_index(
            (fortran_i + 1) as usize,
            fortran_k,
            (fortran_j + 1) as usize,
            west_east_points,
            bottom_top_points,
        )
    }

    fn expected_fortran_bits() -> Vec<u32> {
        include_str!("../../test-data/held_suarez_damp.out.correct")
            .split_whitespace()
            .skip(1)
            .map(|value| u32::from_str_radix(value, 16).unwrap())
            .collect()
    }

    fn read_corpus_field(
        backend: &CpuBackend,
        shape: GridShape,
        corpus: &mut CorpusReader<'_>,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0_f32).unwrap();
        for value in field.values_mut() {
            *value = corpus.read_f32("Held-Suarez field value");
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
