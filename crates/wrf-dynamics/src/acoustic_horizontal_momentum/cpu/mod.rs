use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, ParallelExecutionError};

use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMomentumError,
    AcousticHorizontalMomentumInputs, AcousticHorizontalMomentumKernels,
    AcousticHorizontalMomentumParameters, AcousticHorizontalMomentumRegion,
    AcousticHorizontalMomentumResult, AcousticHorizontalMomentumState,
};

mod pressure_interpolation;
mod south_north;
mod validation;
mod west_east;

impl AcousticHorizontalMomentumKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_acoustic_horizontal_momentum(
        &self,
        state: AcousticHorizontalMomentumState<'_, Self::Field>,
        inputs: AcousticHorizontalMomentumInputs<'_, Self::Field>,
        parameters: AcousticHorizontalMomentumParameters,
        boundary_policy: AcousticHorizontalBoundaryPolicy,
        region: &AcousticHorizontalMomentumRegion,
    ) -> AcousticHorizontalMomentumResult<()> {
        let ranges =
            validation::validate_operation(&state, inputs, parameters, boundary_policy, region)?;
        let AcousticHorizontalMomentumState {
            west_east,
            south_north,
        } = state;

        west_east::advance(self, west_east, inputs, parameters, region, &ranges)?;
        south_north::advance(self, south_north, inputs, parameters, region, &ranges)
    }
}

fn map_parallel_error(
    error: ParallelExecutionError<Infallible>,
) -> AcousticHorizontalMomentumError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => AcousticHorizontalMomentumError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            AcousticHorizontalMomentumError::SchedulerContractViolated
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;
    use crate::{
        AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
        AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumTendencies,
        AcousticHorizontalPressureFields, AcousticHorizontalVerticalCoefficients,
        AcousticPressureMode, AcousticRelaxationZone, AcousticSouthNorthBoundary,
        AcousticWestEastBoundary, AcousticWestEastPeriodicity, VerticalAcousticTopBoundary,
    };

    struct Fixture {
        west_east: CpuField<f32>,
        south_north: CpuField<f32>,
        west_east_tendency: CpuField<f32>,
        south_north_tendency: CpuField<f32>,
        pressure: CpuField<f32>,
        base_pressure: CpuField<f32>,
        geopotential: CpuField<f32>,
        pressure_point_geopotential: CpuField<f32>,
        full_inverse_density: CpuField<f32>,
        inverse_density_perturbation: CpuField<f32>,
        perturbation_mass: CpuField<f32>,
        west_east_mass: CpuField<f32>,
        south_north_mass: CpuField<f32>,
        damping_mass: CpuField<f32>,
        west_east_moisture: CpuField<f32>,
        south_north_moisture: CpuField<f32>,
        west_east_x_map: CpuField<f32>,
        west_east_y_map: CpuField<f32>,
        south_north_x_map: CpuField<f32>,
        inverse_south_north_x_map: CpuField<f32>,
        south_north_y_map: CpuField<f32>,
        half_level_mass_multiplier: Vec<f32>,
        half_level_mass_offset: Vec<f32>,
        lower_interpolation_weight: Vec<f32>,
        upper_interpolation_weight: Vec<f32>,
        inverse_half_level_spacing: Vec<f32>,
    }

    #[test]
    fn advances_tendencies_and_damping_on_every_c_grid_range() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut fixture = fixture(&backend, false);
        let region = region();

        apply(
            &backend,
            &mut fixture,
            AcousticPressureMode::Hydrostatic,
            VerticalAcousticTopBoundary::Nonrigid,
            closed_policy(),
            &region,
        )
        .unwrap();

        for south_north in 1..5 {
            for bottom_top in 1..5 {
                for west_east in 1..6 {
                    assert_eq!(
                        fixture.west_east.values()[index(west_east, bottom_top, south_north)],
                        1.975
                    );
                }
            }
        }
        for south_north in 1..6 {
            for bottom_top in 1..5 {
                for west_east in 1..5 {
                    assert_eq!(
                        fixture.south_north.values()[index(west_east, bottom_top, south_north)],
                        1.975
                    );
                }
            }
        }
    }

    #[test]
    fn nonhydrostatic_and_rigid_lid_branches_are_active_and_deterministic() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let region = region();
        let mut nonrigid_one = fixture(&one_worker, true);
        let mut nonrigid_four = fixture(&four_workers, true);
        let mut rigid = fixture(&one_worker, true);

        apply(
            &one_worker,
            &mut nonrigid_one,
            AcousticPressureMode::Nonhydrostatic,
            VerticalAcousticTopBoundary::Nonrigid,
            closed_policy(),
            &region,
        )
        .unwrap();
        apply(
            &four_workers,
            &mut nonrigid_four,
            AcousticPressureMode::Nonhydrostatic,
            VerticalAcousticTopBoundary::Nonrigid,
            closed_policy(),
            &region,
        )
        .unwrap();
        apply(
            &one_worker,
            &mut rigid,
            AcousticPressureMode::Nonhydrostatic,
            VerticalAcousticTopBoundary::RigidLid,
            closed_policy(),
            &region,
        )
        .unwrap();

        assert_eq!(nonrigid_one.west_east, nonrigid_four.west_east);
        assert_eq!(nonrigid_one.south_north, nonrigid_four.south_north);
        assert_ne!(nonrigid_one.west_east, rigid.west_east);
        assert_ne!(nonrigid_one.south_north, rigid.south_north);
    }

    #[test]
    fn matches_the_direct_pinned_fortran_rigid_nonhydrostatic_oracle() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();
        let mut actual = fixture(&backend, true);
        apply(
            &backend,
            &mut actual,
            AcousticPressureMode::Nonhydrostatic,
            VerticalAcousticTopBoundary::RigidLid,
            closed_policy(),
            &region(),
        )
        .unwrap();
        let mut expected_west_east = Vec::new();
        let mut expected_south_north = Vec::new();
        for line in
            include_str!("../../../test-data/acoustic_horizontal_momentum.out.correct").lines()
        {
            let mut parts = line.split_whitespace();
            let field = parts.next().unwrap();
            let _coordinates = [parts.next(), parts.next(), parts.next()];
            let bits = u32::from_str_radix(parts.next().unwrap(), 16).unwrap();
            match field {
                "u" => expected_west_east.push(bits),
                "v" => expected_south_north.push(bits),
                _ => panic!("unknown oracle field {field}"),
            }
        }
        assert_eq!(field_bits(&actual.west_east), expected_west_east);
        assert_eq!(field_bits(&actual.south_north), expected_south_north);
    }

    #[test]
    fn polar_rows_are_zero_and_validation_is_failure_atomic() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let region = region();
        let mut polar = fixture(&backend, false);
        let polar_policy = AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Disabled,
            AcousticWestEastPeriodicity::Nonperiodic,
            AcousticWestEastBoundary::Closed,
            AcousticWestEastBoundary::Closed,
            AcousticSouthNorthBoundary::Polar,
            AcousticSouthNorthBoundary::Polar,
        );
        apply(
            &backend,
            &mut polar,
            AcousticPressureMode::Hydrostatic,
            VerticalAcousticTopBoundary::Nonrigid,
            polar_policy,
            &region,
        )
        .unwrap();
        for south_north in [1, 5] {
            for bottom_top in 1..5 {
                for west_east in 1..5 {
                    assert_eq!(
                        polar.south_north.values()[index(west_east, bottom_top, south_north)]
                            .to_bits(),
                        0
                    );
                }
            }
        }

        let mut invalid = fixture(&backend, false);
        let before_west_east = invalid.west_east.clone();
        let before_south_north = invalid.south_north.clone();
        invalid.inverse_half_level_spacing.pop();
        assert_eq!(
            apply(
                &backend,
                &mut invalid,
                AcousticPressureMode::Hydrostatic,
                VerticalAcousticTopBoundary::Nonrigid,
                closed_policy(),
                &region
            ),
            Err(AcousticHorizontalMomentumError::CoefficientLengthMismatch {
                coefficient:
                    crate::AcousticHorizontalMomentumCoefficient::InverseHalfLevelEtaSpacing,
                expected: 6,
                actual: 5,
            })
        );
        assert_eq!(invalid.west_east, before_west_east);
        assert_eq!(invalid.south_north, before_south_north);
    }

    fn fixture(backend: &CpuBackend, varying_pressure: bool) -> Fixture {
        let shape = GridShape::try_new(6, 6, 6).unwrap();
        let horizontal_shape = shape.horizontal_shape();
        let volume = |initial| backend.create_field(shape, initial).unwrap();
        let horizontal = |initial| backend.create_field(horizontal_shape, initial).unwrap();
        let mut pressure = volume(3.0);
        let mut pressure_point_geopotential = volume(2.0);
        if varying_pressure {
            for south_north in 0..6 {
                for bottom_top in 0..6 {
                    for west_east in 0..6 {
                        let location = index(west_east, bottom_top, south_north);
                        pressure.values_mut()[location] = 1.0
                            + west_east as f32 * 0.2
                            + south_north as f32 * 0.3
                            + bottom_top as f32 * 0.4;
                        pressure_point_geopotential.values_mut()[location] =
                            2.0 + west_east as f32 * 0.15 + south_north as f32 * 0.25;
                    }
                }
            }
        }
        let mut damping_mass = horizontal(0.0);
        for south_north in 0..6 {
            for west_east in 0..6 {
                damping_mass.values_mut()[south_north * 6 + west_east] =
                    west_east as f32 + 2.0 * south_north as f32;
            }
        }
        Fixture {
            west_east: volume(1.0),
            south_north: volume(1.0),
            west_east_tendency: volume(2.0),
            south_north_tendency: volume(2.0),
            pressure,
            base_pressure: volume(4.0),
            geopotential: volume(5.0),
            pressure_point_geopotential,
            full_inverse_density: volume(0.8),
            inverse_density_perturbation: volume(0.1),
            perturbation_mass: horizontal(1.0),
            west_east_mass: horizontal(2.0),
            south_north_mass: horizontal(2.0),
            damping_mass,
            west_east_moisture: volume(1.0),
            south_north_moisture: volume(1.0),
            west_east_x_map: horizontal(1.0),
            west_east_y_map: horizontal(1.0),
            south_north_x_map: horizontal(1.0),
            inverse_south_north_x_map: horizontal(1.0),
            south_north_y_map: horizontal(1.0),
            half_level_mass_multiplier: vec![0.5; 6],
            half_level_mass_offset: vec![0.25; 6],
            lower_interpolation_weight: vec![0.6; 6],
            upper_interpolation_weight: vec![0.4; 6],
            inverse_half_level_spacing: vec![1.2; 6],
        }
    }

    fn region() -> AcousticHorizontalMomentumRegion {
        AcousticHorizontalMomentumRegion::try_new(
            GridShape::try_new(6, 6, 6).unwrap(),
            1..5,
            1..5,
            1..5,
            1..6,
            1..6,
            1..6,
        )
        .unwrap()
    }

    fn closed_policy() -> AcousticHorizontalBoundaryPolicy {
        AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Disabled,
            AcousticWestEastPeriodicity::Nonperiodic,
            AcousticWestEastBoundary::Closed,
            AcousticWestEastBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
        )
    }

    fn apply(
        backend: &CpuBackend,
        fixture: &mut Fixture,
        mode: AcousticPressureMode,
        top_boundary: VerticalAcousticTopBoundary,
        policy: AcousticHorizontalBoundaryPolicy,
        region: &AcousticHorizontalMomentumRegion,
    ) -> AcousticHorizontalMomentumResult<()> {
        let inputs = AcousticHorizontalMomentumInputs::new(
            AcousticHorizontalMomentumTendencies::new(
                &fixture.west_east_tendency,
                &fixture.south_north_tendency,
            ),
            AcousticHorizontalPressureFields::new(
                &fixture.pressure,
                &fixture.base_pressure,
                &fixture.geopotential,
                &fixture.pressure_point_geopotential,
                &fixture.full_inverse_density,
                &fixture.inverse_density_perturbation,
            ),
            AcousticHorizontalMassFields::new(
                &fixture.perturbation_mass,
                &fixture.west_east_mass,
                &fixture.south_north_mass,
                &fixture.damping_mass,
            ),
            AcousticHorizontalMoistureCoefficients::new(
                &fixture.west_east_moisture,
                &fixture.south_north_moisture,
            ),
            AcousticHorizontalMapFactors::new(
                &fixture.west_east_x_map,
                &fixture.west_east_y_map,
                &fixture.south_north_x_map,
                &fixture.inverse_south_north_x_map,
                &fixture.south_north_y_map,
            ),
            AcousticHorizontalVerticalCoefficients::new(
                &fixture.half_level_mass_multiplier,
                &fixture.half_level_mass_offset,
                &fixture.lower_interpolation_weight,
                &fixture.upper_interpolation_weight,
                &fixture.inverse_half_level_spacing,
            ),
        );
        backend.advance_acoustic_horizontal_momentum(
            AcousticHorizontalMomentumState::new(&mut fixture.west_east, &mut fixture.south_north),
            inputs,
            AcousticHorizontalMomentumParameters::new(
                2.0,
                4.0,
                0.5,
                [0.7, 0.2, 0.1],
                0.1,
                mode,
                top_boundary,
            ),
            policy,
            region,
        )
    }

    const fn index(west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        (south_north * 6 + bottom_top) * 6 + west_east
    }

    fn field_bits(field: &CpuField<f32>) -> Vec<u32> {
        field.values().iter().map(|value| value.to_bits()).collect()
    }
}
