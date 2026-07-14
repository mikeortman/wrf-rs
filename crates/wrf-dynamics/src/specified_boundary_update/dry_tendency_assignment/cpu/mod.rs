mod execution;
mod validation;

use wrf_compute::{CpuBackend, CpuField};

use super::{
    DryBoundaryTendencies, DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyKernels,
    DryBoundaryTendencyRegion, DryBoundaryTendencyResult, DryBoundaryVerticalTendency,
};
use crate::{SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryWestEastPeriodicity};
use execution::DryBoundaryTendencyCpuExecution;

impl DryBoundaryTendencyKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn assign_dry_boundary_tendencies(
        &self,
        tendencies: DryBoundaryTendencies<'_, Self::Field>,
        boundaries: DryBoundaryTendencyBoundaryFields<'_, Self::Field>,
        vertical: DryBoundaryVerticalTendency<'_, Self::Field>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &DryBoundaryTendencyRegion,
    ) -> DryBoundaryTendencyResult<()> {
        DryBoundaryTendencyCpuExecution::new(
            self,
            tendencies,
            boundaries,
            vertical,
            parameters,
            west_east_periodicity,
            region,
        )
        .run()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, FieldStorage, GridShape};

    use super::*;
    use crate::{
        DryBoundaryTendencyError, DryBoundaryTendencyTarget, SpecifiedBoundaryTendencies,
        SpecifiedBoundaryTendencyError,
    };

    const BOUNDARY_WIDTH: usize = 3;

    #[derive(Clone)]
    struct OracleCase {
        name: &'static str,
        periodicity: SpecifiedBoundaryWestEastPeriodicity,
        is_nested: bool,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
        specified_zone_width: usize,
        exceptional: bool,
    }

    struct BoundarySet {
        fields: [CpuField<f32>; 4],
    }

    struct Fixture {
        case: OracleCase,
        outputs: [CpuField<f32>; 5],
        column_mass_output: CpuField<f32>,
        boundaries: [BoundarySet; 6],
        region: DryBoundaryTendencyRegion,
    }

    #[test]
    fn matches_direct_pinned_fortran_for_every_dry_field_and_wrapper_branch() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let expected = parse_oracle();

        for case in oracle_cases() {
            let mut fixture = create_fixture(&backend, case.clone());
            apply_fixture(&backend, &mut fixture).unwrap();
            for (field_name, field) in fixture.named_outputs() {
                assert_output(&expected, case.name, field_name, field);
            }
        }
    }

    #[test]
    fn complete_outputs_are_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();

        for case in oracle_cases() {
            let mut serial = create_fixture(&one_worker, case.clone());
            let mut parallel = create_fixture(&four_workers, case.clone());
            apply_fixture(&one_worker, &mut serial).unwrap();
            apply_fixture(&four_workers, &mut parallel).unwrap();
            assert_eq!(
                output_bits(&serial),
                output_bits(&parallel),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn late_nested_boundary_failure_is_atomic_across_every_tendency() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = create_fixture(&backend, oracle_cases()[1].clone());
        fixture.boundaries[4].fields[3] = backend
            .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
            .unwrap();
        let before = output_bits(&fixture);

        let error = apply_fixture(&backend, &mut fixture).unwrap_err();

        assert!(matches!(
            error,
            DryBoundaryTendencyError::SpecifiedTendency {
                target: DryBoundaryTendencyTarget::VerticalMomentum,
                source: SpecifiedBoundaryTendencyError::BoundaryShapeMismatch { .. },
            }
        ));
        assert_eq!(output_bits(&fixture), before);
    }

    #[test]
    fn every_output_role_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let targets = [
            DryBoundaryTendencyTarget::WestEastMomentum,
            DryBoundaryTendencyTarget::SouthNorthMomentum,
            DryBoundaryTendencyTarget::PerturbationGeopotential,
            DryBoundaryTendencyTarget::PotentialTemperature,
            DryBoundaryTendencyTarget::PerturbationColumnMass,
            DryBoundaryTendencyTarget::VerticalMomentum,
        ];

        for (index, target) in targets.into_iter().enumerate() {
            let mut fixture = create_fixture(&backend, oracle_cases()[1].clone());
            let wrong = backend
                .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
                .unwrap();
            match index {
                0..=3 => fixture.outputs[index] = wrong,
                4 => fixture.column_mass_output = wrong,
                5 => fixture.outputs[4] = wrong,
                _ => unreachable!(),
            }
            let before = output_bits(&fixture);

            let error = apply_fixture(&backend, &mut fixture).unwrap_err();

            assert!(matches!(
                error,
                DryBoundaryTendencyError::SpecifiedTendency {
                    target: actual,
                    source: SpecifiedBoundaryTendencyError::ShapeMismatch { .. },
                } if actual == target
            ));
            assert_eq!(output_bits(&fixture), before, "{target}");
        }
    }

    #[test]
    fn every_boundary_role_is_validated_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let targets = [
            DryBoundaryTendencyTarget::WestEastMomentum,
            DryBoundaryTendencyTarget::SouthNorthMomentum,
            DryBoundaryTendencyTarget::PerturbationGeopotential,
            DryBoundaryTendencyTarget::PotentialTemperature,
            DryBoundaryTendencyTarget::VerticalMomentum,
            DryBoundaryTendencyTarget::PerturbationColumnMass,
        ];

        for (field_index, target) in targets.into_iter().enumerate() {
            for side_index in 0..4 {
                let mut fixture = create_fixture(&backend, oracle_cases()[1].clone());
                fixture.boundaries[field_index].fields[side_index] = backend
                    .create_field(GridShape::try_new(2, 2, 2).unwrap(), 0.0)
                    .unwrap();
                let before = output_bits(&fixture);

                let error = apply_fixture(&backend, &mut fixture).unwrap_err();

                assert!(matches!(
                    error,
                    DryBoundaryTendencyError::SpecifiedTendency {
                        target: actual,
                        source: SpecifiedBoundaryTendencyError::BoundaryShapeMismatch { .. },
                    } if actual == target
                ));
                assert_eq!(output_bits(&fixture), before, "{target} side {side_index}");
            }
        }
    }

    #[test]
    fn inactive_and_zero_width_cases_preserve_complete_outputs() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        for case in oracle_cases()
            .into_iter()
            .filter(|case| case.name == "inactive" || case.name == "zero_zone")
        {
            let mut fixture = create_fixture(&backend, case.clone());
            let before = output_bits(&fixture);
            apply_fixture(&backend, &mut fixture).unwrap();
            assert_eq!(output_bits(&fixture), before, "{}", case.name);
        }
    }

    fn apply_fixture(backend: &CpuBackend, fixture: &mut Fixture) -> DryBoundaryTendencyResult<()> {
        let case = fixture.case.clone();
        let boundaries = DryBoundaryTendencyBoundaryFields::new(
            fixture.boundaries[0].references(),
            fixture.boundaries[1].references(),
            fixture.boundaries[2].references(),
            fixture.boundaries[3].references(),
            fixture.boundaries[5].references(),
        );
        let [west_east, south_north, geopotential, temperature, vertical] = &mut fixture.outputs;
        let vertical = if case.is_nested {
            DryBoundaryVerticalTendency::Nested {
                tendency: vertical,
                boundaries: fixture.boundaries[4].references(),
            }
        } else {
            DryBoundaryVerticalTendency::Disabled
        };
        backend.assign_dry_boundary_tendencies(
            DryBoundaryTendencies::new(
                west_east,
                south_north,
                geopotential,
                temperature,
                &mut fixture.column_mass_output,
            ),
            boundaries,
            vertical,
            SpecifiedBoundaryTendencyParameters::new(BOUNDARY_WIDTH, case.specified_zone_width),
            case.periodicity,
            &fixture.region,
        )
    }

    fn create_fixture(backend: &CpuBackend, case: OracleCase) -> Fixture {
        let volume_shape = GridShape::try_new(10, 10, 6).unwrap();
        let horizontal_shape = volume_shape.horizontal_shape();
        let region = DryBoundaryTendencyRegion::try_new(
            volume_shape,
            1..9,
            1..9,
            1..5,
            case.west_east_tile.clone(),
            case.south_north_tile.clone(),
            case.bottom_top_tile.clone(),
        )
        .unwrap();
        let outputs = std::array::from_fn(|index| {
            initialized_field(backend, volume_shape, |i, k, j| {
                (-1000.0 * (index + 1) as f32 + i * 11.0) + k * 0.25 - j * 3.0
            })
        });
        let mut fixture = Fixture {
            case,
            outputs,
            column_mass_output: initialized_field(backend, horizontal_shape, |i, _, j| {
                -6000.0 + i * 7.0 - j * 2.0
            }),
            boundaries: [
                boundary_set(backend, 1, false),
                boundary_set(backend, 2, false),
                boundary_set(backend, 3, false),
                boundary_set(backend, 4, false),
                boundary_set(backend, 5, false),
                boundary_set(backend, 6, true),
            ],
            region,
        };
        if fixture.case.exceptional {
            set_boundary_bits(&mut fixture.boundaries[0].fields[2], 2, 0, 0, 0x8000_0000);
            set_boundary_bits(&mut fixture.boundaries[1].fields[3], 2, 0, 0, 0x7f80_0000);
            set_boundary_bits(&mut fixture.boundaries[2].fields[0], 2, 0, 0, 0xff80_0000);
            set_boundary_bits(&mut fixture.boundaries[3].fields[1], 2, 0, 0, 0x0000_0001);
            set_boundary_bits(&mut fixture.boundaries[4].fields[2], 3, 0, 0, 0x7fc1_2345);
            set_boundary_bits(&mut fixture.boundaries[5].fields[3], 2, 0, 0, 0x7f7f_ffff);
        }
        fixture
    }

    fn boundary_set(backend: &CpuBackend, field: usize, horizontal: bool) -> BoundarySet {
        let vertical_points = if horizontal { 1 } else { 5 };
        let shape = GridShape::try_new(10, BOUNDARY_WIDTH, vertical_points).unwrap();
        let fields = std::array::from_fn(|side| {
            initialized_field(backend, shape, |line, vertical, distance| {
                let side_base = (side + 1) as f32 * 100.0;
                field as f32 * 1000.0
                    + side_base
                    + line * 10.0
                    + if horizontal { 0.0 } else { vertical + 1.0 }
                    + (distance + 1.0) * 0.01
            })
        });
        BoundarySet { fields }
    }

    fn initialized_field(
        backend: &CpuBackend,
        shape: GridShape,
        value: impl Fn(f32, f32, f32) -> f32,
    ) -> CpuField<f32> {
        let mut field = backend.create_field(shape, 0.0).unwrap();
        for south_north in 0..shape.south_north_points() {
            for bottom_top in 0..shape.bottom_top_points() {
                for west_east in 0..shape.west_east_points() {
                    let index = west_east
                        + shape.west_east_points()
                            * (bottom_top + shape.bottom_top_points() * south_north);
                    field.values_mut()[index] =
                        value(west_east as f32, bottom_top as f32, south_north as f32);
                }
            }
        }
        field
    }

    fn set_boundary_bits(
        field: &mut CpuField<f32>,
        line: usize,
        vertical: usize,
        distance: usize,
        bits: u32,
    ) {
        let shape = field.shape();
        let index =
            line + shape.west_east_points() * (vertical + shape.bottom_top_points() * distance);
        field.values_mut()[index] = f32::from_bits(bits);
    }

    fn output_bits(fixture: &Fixture) -> Vec<u32> {
        fixture
            .outputs
            .iter()
            .chain([&fixture.column_mass_output])
            .flat_map(|field| field.values().iter().map(|value| value.to_bits()))
            .collect()
    }

    fn parse_oracle() -> Vec<(&'static str, &'static str, u32)> {
        include_str!("../../../../test-data/dry_boundary_tendencies.out.correct")
            .lines()
            .map(|line| {
                let mut parts = line.split_whitespace();
                let case = intern_case(parts.next().unwrap());
                let field = intern_field(parts.next().unwrap());
                let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
                assert!(parts.next().is_none());
                (case, field, bits)
            })
            .collect()
    }

    fn intern_case(name: &str) -> &'static str {
        oracle_cases()
            .into_iter()
            .find(|case| case.name == name)
            .map(|case| case.name)
            .unwrap_or_else(|| panic!("unexpected oracle case {name}"))
    }

    fn intern_field(name: &str) -> &'static str {
        match name {
            "u" => "u",
            "v" => "v",
            "ph" => "ph",
            "t" => "t",
            "w" => "w",
            "mu" => "mu",
            _ => panic!("unexpected oracle field {name}"),
        }
    }

    fn assert_output(
        expected: &[(&str, &str, u32)],
        case: &str,
        field_name: &str,
        field: &CpuField<f32>,
    ) {
        let expected_bits: Vec<_> = expected
            .iter()
            .filter_map(|(expected_case, expected_field, bits)| {
                (*expected_case == case && *expected_field == field_name).then_some(*bits)
            })
            .collect();
        let actual_bits: Vec<_> = field.values().iter().map(|value| value.to_bits()).collect();
        assert_eq!(actual_bits, expected_bits, "{case} {field_name}");
    }

    fn oracle_cases() -> [OracleCase; 9] {
        [
            oracle_case("full_global", false, false, 1..9, 1..9, 1..6, 2, false),
            oracle_case("full_nested", false, true, 1..9, 1..9, 1..6, 2, false),
            oracle_case("periodic_nested", true, true, 1..9, 1..9, 1..6, 2, false),
            oracle_case("south_west", false, true, 1..6, 1..6, 1..6, 2, false),
            oracle_case("north_east", false, true, 4..9, 4..9, 1..6, 2, false),
            oracle_case("partial_vertical", false, true, 1..9, 1..9, 2..4, 2, false),
            oracle_case("inactive", false, true, 4..6, 4..6, 1..6, 2, false),
            oracle_case("zero_zone", false, true, 1..9, 1..9, 1..6, 0, false),
            oracle_case("exceptional", false, true, 1..9, 1..9, 1..6, 2, true),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn oracle_case(
        name: &'static str,
        periodic: bool,
        is_nested: bool,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
        specified_zone_width: usize,
        exceptional: bool,
    ) -> OracleCase {
        OracleCase {
            name,
            periodicity: if periodic {
                SpecifiedBoundaryWestEastPeriodicity::Periodic
            } else {
                SpecifiedBoundaryWestEastPeriodicity::Nonperiodic
            },
            is_nested,
            west_east_tile,
            south_north_tile,
            bottom_top_tile,
            specified_zone_width,
            exceptional,
        }
    }

    impl BoundarySet {
        fn references(&self) -> SpecifiedBoundaryTendencies<'_, CpuField<f32>> {
            SpecifiedBoundaryTendencies::new(
                &self.fields[0],
                &self.fields[1],
                &self.fields[2],
                &self.fields[3],
            )
        }
    }

    impl Fixture {
        fn named_outputs(&self) -> [(&'static str, &CpuField<f32>); 6] {
            [
                ("u", &self.outputs[0]),
                ("v", &self.outputs[1]),
                ("ph", &self.outputs[2]),
                ("t", &self.outputs[3]),
                ("w", &self.outputs[4]),
                ("mu", &self.column_mass_output),
            ]
        }
    }
}
