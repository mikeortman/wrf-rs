use wrf_compute::{CpuBackend, CpuField, FieldStorage};

use crate::{
    PhysicalBoundaryConditions, PhysicalBoundaryError, PhysicalBoundaryKernels,
    PhysicalBoundaryRegion, PhysicalBoundaryResult, PhysicalBoundaryVariable,
};

mod geometry;
mod kernel;

use geometry::PhysicalBoundaryGeometry;

impl PhysicalBoundaryKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn apply_physical_boundary(
        &self,
        field: &mut Self::Field,
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> PhysicalBoundaryResult<()> {
        validate_volume_boundary(field, region)?;
        let geometry = PhysicalBoundaryGeometry::for_volume(variable, conditions, region);
        let shape = region.shape();
        let plane_length = shape.west_east_points() * shape.bottom_top_points();
        for (south_north, output_plane) in field
            .values_mut()
            .chunks_exact_mut(plane_length)
            .enumerate()
        {
            kernel::apply_volume_west_east(output_plane, south_north as isize, &geometry);
        }
        kernel::apply_volume_south_north(field.values_mut(), &geometry);
        Ok(())
    }

    fn apply_physical_boundary_horizontal(
        &self,
        field: &mut Self::Field,
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        region: &PhysicalBoundaryRegion,
    ) -> PhysicalBoundaryResult<()> {
        validate_horizontal_boundary(field, region)?;
        let geometry = PhysicalBoundaryGeometry::for_horizontal(variable, conditions, region);
        let row_length = region.horizontal_shape().west_east_points();
        for (south_north, output_row) in field.values_mut().chunks_exact_mut(row_length).enumerate()
        {
            kernel::apply_horizontal_west_east(output_row, south_north as isize, &geometry);
        }
        kernel::apply_horizontal_south_north(field.values_mut(), &geometry);
        Ok(())
    }
}

pub(crate) fn validate_volume_boundary(
    field: &CpuField<f32>,
    region: &PhysicalBoundaryRegion,
) -> PhysicalBoundaryResult<()> {
    validate_shape(field, region.shape())
}

pub(crate) fn validate_horizontal_boundary(
    field: &CpuField<f32>,
    region: &PhysicalBoundaryRegion,
) -> PhysicalBoundaryResult<()> {
    validate_shape(field, region.horizontal_shape())
}

fn validate_shape(
    field: &CpuField<f32>,
    expected: wrf_compute::GridShape,
) -> PhysicalBoundaryResult<()> {
    if field.shape() != expected {
        return Err(PhysicalBoundaryError::ShapeMismatch {
            expected,
            actual: field.shape(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    fn shape() -> GridShape {
        GridShape::try_new(15, 15, 7).unwrap()
    }

    fn full_region() -> PhysicalBoundaryRegion {
        PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..10, 1..6, 4..11, 4..11, 1..7).unwrap()
    }

    fn initialized_volume(backend: &CpuBackend) -> CpuField<f32> {
        let mut field = backend.create_field(shape(), 0.0).unwrap();
        let west_east_points = shape().west_east_points();
        let bottom_top_points = shape().bottom_top_points();
        for south_north in 0..shape().south_north_points() {
            for bottom_top in 0..bottom_top_points {
                for west_east in 0..west_east_points {
                    let index = south_north * west_east_points * bottom_top_points
                        + bottom_top * west_east_points
                        + west_east;
                    field.values_mut()[index] =
                        -20.0 + west_east as f32 * 0.7 + bottom_top as f32 * 0.11
                            - south_north as f32 * 0.3;
                }
            }
        }
        field
    }

    fn initialized_horizontal(backend: &CpuBackend) -> CpuField<f32> {
        let horizontal_shape = shape().horizontal_shape();
        let mut field = backend.create_field(horizontal_shape, 0.0).unwrap();
        let west_east_points = horizontal_shape.west_east_points();
        for south_north in 0..horizontal_shape.south_north_points() {
            for west_east in 0..west_east_points {
                let index = south_north * west_east_points + west_east;
                field.values_mut()[index] =
                    -10.0 + west_east as f32 * 0.5 - south_north as f32 * 0.2;
            }
        }
        let index =
            |west_east: usize, south_north: usize| south_north * west_east_points + west_east;
        field.values_mut()[index(4, 5)] = f32::from_bits(0x7fc0_002a);
        field.values_mut()[index(9, 6)] = f32::INFINITY;
        field.values_mut()[index(5, 9)] = f32::NEG_INFINITY;
        field.values_mut()[index(6, 4)] = f32::from_bits(0x8000_0000);
        field
    }

    #[derive(Clone)]
    struct VolumeOracleCase {
        name: &'static str,
        variable: PhysicalBoundaryVariable,
        conditions: PhysicalBoundaryConditions,
        west_east_tile: Range<usize>,
        south_north_tile: Range<usize>,
        bottom_top_tile: Range<usize>,
    }

    fn volume_oracle_cases() -> [VolumeOracleCase; 6] {
        [
            VolumeOracleCase {
                name: "periodic_p",
                variable: PhysicalBoundaryVariable::MassHalfLevel,
                conditions: PhysicalBoundaryConditions::periodic_xy(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                bottom_top_tile: 1..7,
            },
            VolumeOracleCase {
                name: "specified_u",
                variable: PhysicalBoundaryVariable::WestEastFace,
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                bottom_top_tile: 1..7,
            },
            VolumeOracleCase {
                name: "nested_v",
                variable: PhysicalBoundaryVariable::SouthNorthFace,
                conditions: PhysicalBoundaryConditions::nested_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                bottom_top_tile: 1..7,
            },
            VolumeOracleCase {
                name: "specified_w",
                variable: PhysicalBoundaryVariable::FullLevel,
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..11,
                south_north_tile: 4..11,
                bottom_top_tile: 1..7,
            },
            VolumeOracleCase {
                name: "partial_t",
                variable: PhysicalBoundaryVariable::MassHalfLevel,
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 4..8,
                south_north_tile: 4..8,
                bottom_top_tile: 2..5,
            },
            VolumeOracleCase {
                name: "inactive_p",
                variable: PhysicalBoundaryVariable::MassHalfLevel,
                conditions: PhysicalBoundaryConditions::specified_lateral(),
                west_east_tile: 5..8,
                south_north_tile: 5..8,
                bottom_top_tile: 2..5,
            },
        ]
    }

    #[test]
    fn volume_cases_match_direct_pinned_fortran_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        for case in volume_oracle_cases() {
            let mut field = initialized_volume(&backend);
            install_volume_special_values(&mut field);
            backend
                .apply_physical_boundary(
                    &mut field,
                    case.variable,
                    case.conditions,
                    &PhysicalBoundaryRegion::try_new(
                        shape(),
                        4..10,
                        4..10,
                        1..6,
                        case.west_east_tile,
                        case.south_north_tile,
                        case.bottom_top_tile,
                    )
                    .unwrap(),
                )
                .unwrap();

            assert_matches_oracle(case.name, field.values());
        }
    }

    #[test]
    fn horizontal_cases_match_direct_pinned_fortran_bits() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        for (name, conditions) in [
            (
                "horizontal_periodic_t",
                PhysicalBoundaryConditions::periodic_xy(),
            ),
            (
                "horizontal_specified_t",
                PhysicalBoundaryConditions::specified_lateral(),
            ),
            (
                "horizontal_nested_t",
                PhysicalBoundaryConditions::nested_lateral(),
            ),
        ] {
            let mut field = initialized_horizontal(&backend);
            backend
                .apply_physical_boundary_horizontal(
                    &mut field,
                    PhysicalBoundaryVariable::MassHalfLevel,
                    conditions,
                    &full_region(),
                )
                .unwrap();

            assert_matches_oracle(name, field.values());
        }
    }

    #[test]
    fn periodic_volume_is_bitwise_deterministic_across_worker_counts() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let mut one_worker_field = initialized_volume(&one_worker);
        let mut four_worker_field = initialized_volume(&four_workers);

        one_worker
            .apply_physical_boundary(
                &mut one_worker_field,
                PhysicalBoundaryVariable::MassHalfLevel,
                PhysicalBoundaryConditions::periodic_xy(),
                &full_region(),
            )
            .unwrap();
        four_workers
            .apply_physical_boundary(
                &mut four_worker_field,
                PhysicalBoundaryVariable::MassHalfLevel,
                PhysicalBoundaryConditions::periodic_xy(),
                &full_region(),
            )
            .unwrap();

        assert_eq!(one_worker_field, four_worker_field);
    }

    #[test]
    fn specified_volume_copies_edge_bits_including_non_finite_values() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut field = initialized_volume(&backend);
        let shape = shape();
        let index = |west_east: usize, bottom_top: usize, south_north: usize| {
            south_north * shape.west_east_points() * shape.bottom_top_points()
                + bottom_top * shape.west_east_points()
                + west_east
        };
        field.values_mut()[index(4, 2, 5)] = f32::from_bits(0x7fc0_002a);

        backend
            .apply_physical_boundary(
                &mut field,
                PhysicalBoundaryVariable::MassHalfLevel,
                PhysicalBoundaryConditions::specified_lateral(),
                &full_region(),
            )
            .unwrap();

        assert_eq!(
            field.values()[index(1, 2, 5)].to_bits(),
            f32::from_bits(0x7fc0_002a).to_bits()
        );
    }

    fn install_volume_special_values(field: &mut CpuField<f32>) {
        let shape = shape();
        let index = |west_east: usize, bottom_top: usize, south_north: usize| {
            south_north * shape.west_east_points() * shape.bottom_top_points()
                + bottom_top * shape.west_east_points()
                + west_east
        };
        field.values_mut()[index(4, 2, 5)] = f32::from_bits(0x7fc0_002a);
        field.values_mut()[index(9, 3, 6)] = f32::INFINITY;
        field.values_mut()[index(5, 4, 9)] = f32::NEG_INFINITY;
        field.values_mut()[index(6, 1, 4)] = f32::from_bits(0x8000_0000);
    }

    fn assert_matches_oracle(case_name: &str, actual: &[f32]) {
        let expected = include_str!("../../../test-data/physical_boundary.out.correct")
            .lines()
            .filter_map(|line| {
                let mut columns = line.split_whitespace();
                (columns.next() == Some(case_name)).then(|| {
                    u32::from_str_radix(line.split_whitespace().last().unwrap(), 16).unwrap()
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected.len(), "case {case_name}");
        for (index, (&actual_value, &expected_bits)) in
            actual.iter().zip(expected.iter()).enumerate()
        {
            let expected_value = f32::from_bits(expected_bits);
            if expected_value.is_nan() {
                assert!(actual_value.is_nan(), "case {case_name}, index {index}");
            } else {
                assert_eq!(
                    actual_value.to_bits(),
                    expected_bits,
                    "case {case_name}, index {index}"
                );
            }
        }
    }

    #[test]
    fn interior_tile_is_an_exact_no_op() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut field = initialized_volume(&backend);
        let before = field.clone();
        let region =
            PhysicalBoundaryRegion::try_new(shape(), 4..10, 4..10, 1..6, 5..8, 5..8, 2..5).unwrap();

        backend
            .apply_physical_boundary(
                &mut field,
                PhysicalBoundaryVariable::MassHalfLevel,
                PhysicalBoundaryConditions::nested_lateral(),
                &region,
            )
            .unwrap();

        assert_eq!(field, before);
    }

    #[test]
    fn malformed_shape_fails_before_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let wrong_shape = GridShape::try_new(14, 15, 7).unwrap();
        let mut field = backend.create_field(wrong_shape, -17.0).unwrap();
        let before = field.clone();

        let result = backend.apply_physical_boundary(
            &mut field,
            PhysicalBoundaryVariable::FullLevel,
            PhysicalBoundaryConditions::specified_lateral(),
            &full_region(),
        );

        assert!(matches!(
            result,
            Err(PhysicalBoundaryError::ShapeMismatch { .. })
        ));
        assert_eq!(field, before);
    }
}
