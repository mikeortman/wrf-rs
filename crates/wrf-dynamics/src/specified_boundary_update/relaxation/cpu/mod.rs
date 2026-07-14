mod boundary_side;
mod kernel;
mod side_view;
mod update_context;

use wrf_compute::{CpuBackend, CpuField};

use super::{
    SpecifiedBoundaryRelaxationInputs, SpecifiedBoundaryRelaxationKernels,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryRelaxationResult,
};
use crate::{SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity};
use kernel::SpecifiedBoundaryRelaxationCpuKernel;

impl SpecifiedBoundaryRelaxationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn add_specified_boundary_relaxation_tendencies(
        &self,
        tendency: &mut Self::Field,
        inputs: SpecifiedBoundaryRelaxationInputs<'_, Self::Field>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> SpecifiedBoundaryRelaxationResult<()> {
        SpecifiedBoundaryRelaxationCpuKernel::try_new(
            tendency,
            inputs,
            parameters,
            west_east_periodicity,
            region,
        )?
        .execute(self)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, FieldStorage, GridShape};

    use super::*;
    use crate::{
        SpecifiedBoundaryFieldLocation, SpecifiedBoundaryRelaxationBoundaryValues,
        SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationError,
        SpecifiedBoundaryRelaxationField, SpecifiedBoundaryTendencies,
    };

    const BOUNDARY_WIDTH: usize = 4;
    const HORIZONTAL_POINTS: usize = 10;

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        location: SpecifiedBoundaryFieldLocation,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        domain_top: usize,
        memory_bottom: usize,
        memory_top: usize,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
        specified_zone_width: usize,
        relaxation_zone_width: usize,
        tile_field: bool,
        exceptional: bool,
    }

    #[test]
    fn matches_direct_pinned_fortran_for_staggers_tiles_periodicity_and_ieee_values() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = parse_oracle();

        for case in oracle_cases() {
            let actual = run_case(&backend, &case);
            let expected_case: Vec<_> = expected
                .iter()
                .filter_map(|(name, bits)| (*name == case.name).then_some(*bits))
                .collect();
            assert_field_matches(&actual, &expected_case, case.name);
        }
    }

    #[test]
    fn complete_output_is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            assert_eq!(
                run_case(&one_worker, &case),
                run_case(&four_workers, &case),
                "case {}",
                case.name,
            );
        }
    }

    #[test]
    fn every_boundary_role_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        let region = region(&case);
        let field = model_field(&backend, &case, false);
        let valid = boundary_fields(&backend, &case);
        let wrong = backend
            .create_field(GridShape::try_new(9, BOUNDARY_WIDTH, 5).unwrap(), 9.0)
            .unwrap();
        let roles = [
            "west boundary values",
            "east boundary values",
            "south boundary values",
            "north boundary values",
            "west boundary tendencies",
            "east boundary tendencies",
            "south boundary tendencies",
            "north boundary tendencies",
        ];

        for (wrong_index, expected_role) in roles.into_iter().enumerate() {
            let mut tendency = initialized_tendency(&backend, &case);
            let original = tendency.clone();
            let selected = |index: usize| {
                if index == wrong_index {
                    &wrong
                } else {
                    &valid[index]
                }
            };
            let result = backend.add_specified_boundary_relaxation_tendencies(
                &mut tendency,
                SpecifiedBoundaryRelaxationInputs::new(
                    full_field_view(&field),
                    SpecifiedBoundaryRelaxationBoundaryValues::new(
                        selected(0),
                        selected(1),
                        selected(2),
                        selected(3),
                    ),
                    SpecifiedBoundaryTendencies::new(
                        selected(4),
                        selected(5),
                        selected(6),
                        selected(7),
                    ),
                    coefficients(),
                ),
                parameters(&case),
                case.periodicity,
                &region,
            );

            assert!(matches!(
                result,
                Err(SpecifiedBoundaryRelaxationError::BoundaryShapeMismatch {
                    field,
                    ..
                }) if field == expected_role
            ));
            assert_eq!(tendency, original);
        }
    }

    #[test]
    fn zone_coefficient_shape_and_field_coverage_failures_are_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let case = oracle_cases()[0].clone();
        let region = region(&case);
        let field = model_field(&backend, &case, false);
        let boundaries = boundary_fields(&backend, &case);

        for (parameters, expected) in [
            (
                SpecifiedBoundaryRelaxationParameters::new(0, 1, 0, 0.25),
                SpecifiedBoundaryRelaxationError::InvalidBoundaryWidth,
            ),
            (
                SpecifiedBoundaryRelaxationParameters::new(4, 0, 3, 0.25),
                SpecifiedBoundaryRelaxationError::InvalidSpecifiedZoneWidth,
            ),
            (
                SpecifiedBoundaryRelaxationParameters::new(4, 1, 5, 0.25),
                SpecifiedBoundaryRelaxationError::ZoneExceedsBoundaryWidth {
                    boundary_width: 4,
                    zone_width: 5,
                },
            ),
            (
                SpecifiedBoundaryRelaxationParameters::new(4, 1, 4, 0.25),
                SpecifiedBoundaryRelaxationError::RelaxationNeighborOutsideBoundary {
                    boundary_width: 4,
                    relaxation_zone_width: 4,
                },
            ),
        ] {
            let mut tendency = initialized_tendency(&backend, &case);
            let original = tendency.clone();
            let result = backend.add_specified_boundary_relaxation_tendencies(
                &mut tendency,
                inputs(full_field_view(&field), &boundaries, coefficients()),
                parameters,
                case.periodicity,
                &region,
            );
            assert_eq!(result, Err(expected));
            assert_eq!(tendency, original);
        }

        let mut tendency = initialized_tendency(&backend, &case);
        let original = tendency.clone();
        let short_forcing = [0.0, 0.7, 0.4];
        let smoothing = [0.0, 0.1, 0.05, 0.0];
        let result = backend.add_specified_boundary_relaxation_tendencies(
            &mut tendency,
            inputs(
                full_field_view(&field),
                &boundaries,
                SpecifiedBoundaryRelaxationCoefficients::new(&short_forcing, &smoothing),
            ),
            parameters(&case),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            result,
            Err(
                SpecifiedBoundaryRelaxationError::CoefficientLengthMismatch {
                    coefficient: "forcing",
                    ..
                }
            )
        ));
        assert_eq!(tendency, original);

        let forcing = [0.0, 0.7, 0.4, 0.0];
        let short_smoothing = [0.0, 0.1, 0.05];
        let result = backend.add_specified_boundary_relaxation_tendencies(
            &mut tendency,
            inputs(
                full_field_view(&field),
                &boundaries,
                SpecifiedBoundaryRelaxationCoefficients::new(&forcing, &short_smoothing),
            ),
            parameters(&case),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            result,
            Err(
                SpecifiedBoundaryRelaxationError::CoefficientLengthMismatch {
                    coefficient: "smoothing",
                    ..
                }
            )
        ));
        assert_eq!(tendency, original);

        let mismatched_field = backend
            .create_field(GridShape::try_new(5, 10, 6).unwrap(), 1.0)
            .unwrap();
        let result = backend.add_specified_boundary_relaxation_tendencies(
            &mut tendency,
            inputs(
                SpecifiedBoundaryRelaxationField::new(&mismatched_field, 2..8, 0..10, 0..6),
                &boundaries,
                coefficients(),
            ),
            parameters(&case),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            result,
            Err(SpecifiedBoundaryRelaxationError::FieldViewShapeMismatch { .. })
        ));
        assert_eq!(tendency, original);

        let narrow_shape = GridShape::try_new(6, 10, 6).unwrap();
        let narrow_field = backend.create_field(narrow_shape, 1.0).unwrap();
        let result = backend.add_specified_boundary_relaxation_tendencies(
            &mut tendency,
            inputs(
                SpecifiedBoundaryRelaxationField::new(&narrow_field, 2..8, 0..10, 0..6),
                &boundaries,
                coefficients(),
            ),
            parameters(&case),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            result,
            Err(SpecifiedBoundaryRelaxationError::FieldCoverageMissing {
                axis: crate::SpecifiedBoundaryUpdateAxis::WestEast,
                ..
            })
        ));
        assert_eq!(tendency, original);

        let wrong_shape = GridShape::try_new(9, 10, 6).unwrap();
        let mut wrong_tendency = backend.create_field(wrong_shape, 8.0).unwrap();
        let wrong_original = wrong_tendency.clone();
        let result = backend.add_specified_boundary_relaxation_tendencies(
            &mut wrong_tendency,
            inputs(full_field_view(&field), &boundaries, coefficients()),
            parameters(&case),
            case.periodicity,
            &region,
        );
        assert!(matches!(
            result,
            Err(SpecifiedBoundaryRelaxationError::ShapeMismatch { .. })
        ));
        assert_eq!(wrong_tendency, wrong_original);
    }

    fn run_case(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        let mut tendency = initialized_tendency(backend, case);
        let field = model_field(backend, case, case.tile_field);
        let boundaries = boundary_fields(backend, case);
        let field_view = if case.tile_field {
            tile_field_view(&field, case)
        } else {
            full_field_view(&field)
        };
        backend
            .add_specified_boundary_relaxation_tendencies(
                &mut tendency,
                inputs(field_view, &boundaries, coefficients()),
                parameters(case),
                case.periodicity,
                &region(case),
            )
            .unwrap();
        tendency
    }

    fn inputs<'a>(
        field: SpecifiedBoundaryRelaxationField<'a, CpuField<f32>>,
        boundaries: &'a [CpuField<f32>; 8],
        coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
    ) -> SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>> {
        SpecifiedBoundaryRelaxationInputs::new(
            field,
            SpecifiedBoundaryRelaxationBoundaryValues::new(
                &boundaries[0],
                &boundaries[1],
                &boundaries[2],
                &boundaries[3],
            ),
            SpecifiedBoundaryTendencies::new(
                &boundaries[4],
                &boundaries[5],
                &boundaries[6],
                &boundaries[7],
            ),
            coefficients,
        )
    }

    fn coefficients() -> SpecifiedBoundaryRelaxationCoefficients<'static> {
        static FORCING: [f32; 4] = [0.0, 0.7, 0.4, 0.0];
        static SMOOTHING: [f32; 4] = [0.0, 0.1, 0.05, 0.0];
        SpecifiedBoundaryRelaxationCoefficients::new(&FORCING, &SMOOTHING)
    }

    fn parameters(case: &OracleCase) -> SpecifiedBoundaryRelaxationParameters {
        SpecifiedBoundaryRelaxationParameters::new(
            BOUNDARY_WIDTH,
            case.specified_zone_width,
            case.relaxation_zone_width,
            0.25,
        )
    }

    fn region(case: &OracleCase) -> SpecifiedBoundaryUpdateRegion {
        SpecifiedBoundaryUpdateRegion::try_new(
            model_shape(case),
            case.location,
            1..9,
            1..9,
            half_level_domain(case),
            case.tile_west_east.clone(),
            case.tile_south_north.clone(),
            case.tile_bottom_top.clone(),
        )
        .unwrap()
    }

    fn half_level_domain(case: &OracleCase) -> Range<usize> {
        if case.location == SpecifiedBoundaryFieldLocation::HorizontalMass {
            return 0..1;
        }
        (1 - case.memory_bottom)..(case.domain_top - case.memory_bottom)
    }

    fn model_shape(case: &OracleCase) -> GridShape {
        GridShape::try_new(
            HORIZONTAL_POINTS,
            HORIZONTAL_POINTS,
            case.memory_top - case.memory_bottom + 1,
        )
        .unwrap()
    }

    fn initialized_tendency(backend: &CpuBackend, case: &OracleCase) -> CpuField<f32> {
        initialized_field(
            backend,
            model_shape(case),
            |west_east, bottom_top, south_north| {
                let source_bottom_top = bottom_top + case.memory_bottom;
                ((-20.0 + west_east as f32 * 0.25) + source_bottom_top as f32 * 0.0625)
                    - south_north as f32 * 0.5
            },
        )
    }

    fn model_field(backend: &CpuBackend, case: &OracleCase, tile_field: bool) -> CpuField<f32> {
        if !tile_field {
            return initialized_field(
                backend,
                model_shape(case),
                |west_east, bottom_top, south_north| {
                    let source_bottom_top = bottom_top + case.memory_bottom;
                    ((50.0 + west_east as f32 * 0.5) + source_bottom_top as f32 * 0.25)
                        - south_north as f32 * 0.125
                },
            );
        }
        let west_east = tile_field_west_east(case);
        let south_north = tile_field_south_north(case);
        let bottom_top = case.tile_bottom_top.clone();
        let shape =
            GridShape::try_new(west_east.len(), south_north.len(), bottom_top.len()).unwrap();
        initialized_field(
            backend,
            shape,
            |local_west_east, local_bottom_top, local_south_north| {
                let source_west_east = local_west_east + west_east.start;
                let source_bottom_top = local_bottom_top + bottom_top.start + case.memory_bottom;
                let source_south_north = local_south_north + south_north.start;
                ((50.0 + source_west_east as f32 * 0.5) + source_bottom_top as f32 * 0.25)
                    - source_south_north as f32 * 0.125
            },
        )
    }

    fn full_field_view(
        field: &CpuField<f32>,
    ) -> SpecifiedBoundaryRelaxationField<'_, CpuField<f32>> {
        let shape = field.shape();
        SpecifiedBoundaryRelaxationField::new(
            field,
            0..shape.west_east_points(),
            0..shape.south_north_points(),
            0..shape.bottom_top_points(),
        )
    }

    fn tile_field_view<'a>(
        field: &'a CpuField<f32>,
        case: &OracleCase,
    ) -> SpecifiedBoundaryRelaxationField<'a, CpuField<f32>> {
        SpecifiedBoundaryRelaxationField::new(
            field,
            tile_field_west_east(case),
            tile_field_south_north(case),
            case.tile_bottom_top.clone(),
        )
    }

    fn tile_field_west_east(case: &OracleCase) -> Range<usize> {
        case.tile_west_east.start.saturating_sub(1)
            ..case
                .tile_west_east
                .end
                .saturating_add(1)
                .min(HORIZONTAL_POINTS)
    }

    fn tile_field_south_north(case: &OracleCase) -> Range<usize> {
        case.tile_south_north.start.saturating_sub(1)
            ..case
                .tile_south_north
                .end
                .saturating_add(1)
                .min(HORIZONTAL_POINTS)
    }

    fn boundary_fields(backend: &CpuBackend, case: &OracleCase) -> [CpuField<f32>; 8] {
        let shape =
            GridShape::try_new(HORIZONTAL_POINTS, BOUNDARY_WIDTH, boundary_vertical(case)).unwrap();
        let mut fields = [
            boundary_field(backend, shape, 100.0, false),
            boundary_field(backend, shape, 200.0, false),
            boundary_field(backend, shape, 300.0, false),
            boundary_field(backend, shape, 400.0, false),
            boundary_field(backend, shape, -3.0, true),
            boundary_field(backend, shape, 4.0, true),
            boundary_field(backend, shape, -5.0, true),
            boundary_field(backend, shape, 6.0, true),
        ];
        if case.exceptional {
            let index = boundary_index(4, 0, 1, shape);
            fields[2].values_mut()[index] = -0.0;
            fields[6].values_mut()[index] = f32::from_bits(1);
            fields[3].values_mut()[index] = f32::MAX;
            fields[0].values_mut()[index] = f32::INFINITY;
            fields[5].values_mut()[index] = f32::NEG_INFINITY;
        }
        fields
    }

    fn boundary_field(
        backend: &CpuBackend,
        shape: GridShape,
        base: f32,
        tendency: bool,
    ) -> CpuField<f32> {
        initialized_field(backend, shape, |line_point, vertical, distance| {
            if tendency {
                ((base + line_point as f32 * 0.125) + (vertical + 1) as f32 * 0.0625)
                    + (distance + 1) as f32 * 0.015625
            } else {
                ((base + line_point as f32 * 0.5) + (vertical + 1) as f32 * 0.25)
                    + (distance + 1) as f32 * 0.03125
            }
        })
    }

    fn boundary_vertical(case: &OracleCase) -> usize {
        if case.location == SpecifiedBoundaryFieldLocation::HorizontalMass {
            1
        } else {
            case.domain_top
        }
    }

    fn boundary_index(
        line_point: usize,
        vertical: usize,
        distance: usize,
        shape: GridShape,
    ) -> usize {
        line_point + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance)
    }

    fn initialized_field(
        backend: &CpuBackend,
        shape: GridShape,
        mut value: impl FnMut(usize, usize, usize) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] = value(west_east, bottom_top, south_north);
                }
            }
        }
        field
    }

    fn oracle_cases() -> Vec<OracleCase> {
        vec![
            case(
                "mass_full",
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                0..6,
                1..9,
                1..9,
                1..5,
            ),
            case(
                "west_east_full",
                SpecifiedBoundaryFieldLocation::WestEastFace,
                0..6,
                1..10,
                1..9,
                1..5,
            ),
            case(
                "south_north_full",
                SpecifiedBoundaryFieldLocation::SouthNorthFace,
                0..6,
                1..9,
                1..10,
                1..5,
            ),
            tile_case("full_level_tile_south_west", 1..6, 1..6, 2..5),
            tile_case("full_level_tile_north_east", 4..9, 4..9, 1..6),
            case(
                "horizontal_mass",
                SpecifiedBoundaryFieldLocation::HorizontalMass,
                1..2,
                1..9,
                1..9,
                0..1,
            ),
            OracleCase {
                name: "periodic_mass",
                periodicity: SpecifiedBoundaryWestEastPeriodicity::Periodic,
                ..case(
                    "periodic_mass",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                    0..6,
                    1..9,
                    1..9,
                    1..5,
                )
            },
            case(
                "inactive_interior",
                SpecifiedBoundaryFieldLocation::MassHalfLevel,
                0..6,
                4..6,
                4..6,
                1..5,
            ),
            OracleCase {
                name: "empty_relaxation_band",
                specified_zone_width: 2,
                relaxation_zone_width: 2,
                ..case(
                    "empty_relaxation_band",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                    0..6,
                    1..9,
                    1..9,
                    1..5,
                )
            },
            OracleCase {
                name: "exceptional",
                exceptional: true,
                ..case(
                    "exceptional",
                    SpecifiedBoundaryFieldLocation::MassHalfLevel,
                    0..6,
                    1..9,
                    1..9,
                    1..5,
                )
            },
        ]
    }

    fn case(
        name: &'static str,
        location: SpecifiedBoundaryFieldLocation,
        memory_bottom_top: Range<usize>,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> OracleCase {
        OracleCase {
            name,
            location,
            periodicity: SpecifiedBoundaryWestEastPeriodicity::Nonperiodic,
            domain_top: if location == SpecifiedBoundaryFieldLocation::HorizontalMass {
                1
            } else {
                5
            },
            memory_bottom: memory_bottom_top.start,
            memory_top: memory_bottom_top.end - 1,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
            specified_zone_width: 1,
            relaxation_zone_width: 3,
            tile_field: false,
            exceptional: false,
        }
    }

    fn tile_case(
        name: &'static str,
        tile_west_east: Range<usize>,
        tile_south_north: Range<usize>,
        tile_bottom_top: Range<usize>,
    ) -> OracleCase {
        OracleCase {
            tile_field: true,
            ..case(
                name,
                SpecifiedBoundaryFieldLocation::FullLevel,
                0..6,
                tile_west_east,
                tile_south_north,
                tile_bottom_top,
            )
        }
    }

    fn parse_oracle() -> Vec<(&'static str, u32)> {
        include_str!("../../../../test-data/specified_boundary_relaxation.out.correct")
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace();
                let name = match parts.next().unwrap() {
                    "mass_full" => "mass_full",
                    "west_east_full" => "west_east_full",
                    "south_north_full" => "south_north_full",
                    "full_level_tile_south_west" => "full_level_tile_south_west",
                    "full_level_tile_north_east" => "full_level_tile_north_east",
                    "horizontal_mass" => "horizontal_mass",
                    "periodic_mass" => "periodic_mass",
                    "inactive_interior" => "inactive_interior",
                    "empty_relaxation_band" => "empty_relaxation_band",
                    "exceptional" => "exceptional",
                    unexpected => panic!("unexpected oracle case {unexpected}"),
                };
                let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
                assert!(parts.next().is_none());
                (name, bits)
            })
            .collect()
    }

    fn assert_field_matches(field: &CpuField<f32>, expected: &[u32], case: &str) {
        assert_eq!(field.values().len(), expected.len(), "case {case}");
        for (index, (actual, expected)) in field.values().iter().zip(expected).enumerate() {
            if f32::from_bits(*expected).is_nan() {
                assert!(actual.is_nan(), "case {case}, index {index}");
            } else {
                assert_eq!(actual.to_bits(), *expected, "case {case}, index {index}");
            }
        }
    }
}
