use std::convert::Infallible;
use std::ops::Range;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    AcousticStepFinalizationCoefficient, AcousticStepFinalizationCoefficients,
    AcousticStepFinalizationControls, AcousticStepFinalizationError, AcousticStepFinalizationField,
    AcousticStepFinalizationKernels, AcousticStepFinalizationMapFactors,
    AcousticStepFinalizationMasses, AcousticStepFinalizationPhase, AcousticStepFinalizationRegion,
    AcousticStepFinalizationResult, AcousticStepFinalizationSavedState,
    AcousticStepFinalizationState,
};

impl AcousticStepFinalizationKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn finalize_acoustic_step(
        &self,
        state: AcousticStepFinalizationState<'_, Self::Field>,
        masses: AcousticStepFinalizationMasses<'_, Self::Field>,
        saved: AcousticStepFinalizationSavedState<'_, Self::Field>,
        map_factors: AcousticStepFinalizationMapFactors<'_, Self::Field>,
        coefficients: AcousticStepFinalizationCoefficients<'_>,
        controls: AcousticStepFinalizationControls,
        region: &AcousticStepFinalizationRegion,
    ) -> AcousticStepFinalizationResult<()> {
        validate_operation(&state, &masses, &saved, &map_factors, coefficients, region)?;

        finalize_south_north_velocity(
            self,
            state.south_north_velocity,
            &masses,
            &saved,
            &map_factors,
            coefficients,
            region,
        )?;
        finalize_west_east_velocity(
            self,
            state.west_east_velocity,
            &masses,
            &saved,
            &map_factors,
            coefficients,
            region,
        )?;
        finalize_vertical_velocity(
            self,
            state.vertical_velocity,
            &masses,
            &saved,
            &map_factors,
            coefficients,
            region,
        )?;
        add_saved_volume(
            self,
            state.perturbation_geopotential,
            saved.perturbation_geopotential,
            region,
        )?;
        add_saved_volume(
            self,
            state.vertical_mass_flux,
            saved.vertical_mass_flux,
            region,
        )?;
        finalize_potential_temperature(
            self,
            state.potential_temperature,
            &masses,
            &saved,
            coefficients,
            controls,
            region,
        )?;
        add_saved_column_mass(
            self,
            state.perturbation_column_mass,
            saved.perturbation_column_mass,
            region,
        )
    }
}

fn validate_operation(
    state: &AcousticStepFinalizationState<'_, CpuField<f32>>,
    masses: &AcousticStepFinalizationMasses<'_, CpuField<f32>>,
    saved: &AcousticStepFinalizationSavedState<'_, CpuField<f32>>,
    map_factors: &AcousticStepFinalizationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepFinalizationCoefficients<'_>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let volume_shape = region.shape();
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (
            &*state.west_east_velocity,
            AcousticStepFinalizationField::WestEastVelocity,
        ),
        (
            &*state.south_north_velocity,
            AcousticStepFinalizationField::SouthNorthVelocity,
        ),
        (
            &*state.vertical_velocity,
            AcousticStepFinalizationField::VerticalVelocity,
        ),
        (
            &*state.potential_temperature,
            AcousticStepFinalizationField::PotentialTemperature,
        ),
        (
            &*state.perturbation_geopotential,
            AcousticStepFinalizationField::PerturbationGeopotential,
        ),
        (
            &*state.vertical_mass_flux,
            AcousticStepFinalizationField::VerticalMassFlux,
        ),
        (
            saved.west_east_velocity,
            AcousticStepFinalizationField::SavedWestEastVelocity,
        ),
        (
            saved.south_north_velocity,
            AcousticStepFinalizationField::SavedSouthNorthVelocity,
        ),
        (
            saved.vertical_velocity,
            AcousticStepFinalizationField::SavedVerticalVelocity,
        ),
        (
            saved.potential_temperature,
            AcousticStepFinalizationField::SavedPotentialTemperature,
        ),
        (
            saved.perturbation_geopotential,
            AcousticStepFinalizationField::SavedPerturbationGeopotential,
        ),
        (
            saved.vertical_mass_flux,
            AcousticStepFinalizationField::SavedVerticalMassFlux,
        ),
        (
            saved.diabatic_heating,
            AcousticStepFinalizationField::DiabaticHeating,
        ),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            &*state.perturbation_column_mass,
            AcousticStepFinalizationField::PerturbationColumnMass,
        ),
        (
            masses.large_step_full,
            AcousticStepFinalizationField::LargeStepFullColumnMass,
        ),
        (
            masses.final_full,
            AcousticStepFinalizationField::FinalFullColumnMass,
        ),
        (
            masses.large_step_west_east,
            AcousticStepFinalizationField::LargeStepWestEastColumnMass,
        ),
        (
            masses.final_west_east,
            AcousticStepFinalizationField::FinalWestEastColumnMass,
        ),
        (
            masses.large_step_south_north,
            AcousticStepFinalizationField::LargeStepSouthNorthColumnMass,
        ),
        (
            masses.final_south_north,
            AcousticStepFinalizationField::FinalSouthNorthColumnMass,
        ),
        (
            saved.perturbation_column_mass,
            AcousticStepFinalizationField::SavedPerturbationColumnMass,
        ),
        (
            map_factors.west_east_velocity_y,
            AcousticStepFinalizationField::WestEastVelocityMapFactor,
        ),
        (
            map_factors.south_north_velocity_x,
            AcousticStepFinalizationField::SouthNorthVelocityMapFactor,
        ),
        (
            map_factors.mass_point_y,
            AcousticStepFinalizationField::MassPointSouthNorthMapFactor,
        ),
    ] {
        validate_shape(field, role, horizontal_shape)?;
    }
    let coefficient_length = volume_shape.bottom_top_points();
    for (values, coefficient) in [
        (
            coefficients.half_level_mass_multiplier,
            AcousticStepFinalizationCoefficient::HalfLevelMassMultiplier,
        ),
        (
            coefficients.half_level_mass_offset,
            AcousticStepFinalizationCoefficient::HalfLevelMassOffset,
        ),
        (
            coefficients.full_level_mass_multiplier,
            AcousticStepFinalizationCoefficient::FullLevelMassMultiplier,
        ),
        (
            coefficients.full_level_mass_offset,
            AcousticStepFinalizationCoefficient::FullLevelMassOffset,
        ),
    ] {
        if values.len() != coefficient_length {
            return Err(AcousticStepFinalizationError::CoefficientLengthMismatch {
                coefficient,
                expected: coefficient_length,
                actual: values.len(),
            });
        }
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: AcousticStepFinalizationField,
    expected: GridShape,
) -> AcousticStepFinalizationResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(AcousticStepFinalizationError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn finalize_south_north_velocity(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    masses: &AcousticStepFinalizationMasses<'_, CpuField<f32>>,
    saved: &AcousticStepFinalizationSavedState<'_, CpuField<f32>>,
    map_factors: &AcousticStepFinalizationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepFinalizationCoefficients<'_>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let x_range = region.mass_west_east_range();
    let y_range = region.south_north_velocity_range();
    let z_range = region.mass_bottom_top_range();
    let saved_values = saved.south_north_velocity.values();
    let map_values = map_factors.south_north_velocity_x.values();
    let old_mass = masses.large_step_south_north.values();
    let new_mass = masses.final_south_north.values();
    for_each_volume_row(
        backend,
        output,
        &x_range,
        &y_range,
        &z_range,
        |index, x, z, y, value| {
            let horizontal_index = y * west_east_points + x;
            let old_weight = coefficients.half_level_mass_multiplier[z]
                * old_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            let new_weight = coefficients.half_level_mass_multiplier[z]
                * new_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            *value = (map_values[horizontal_index] * *value + saved_values[index] * old_weight)
                / new_weight;
        },
    )
}

fn finalize_west_east_velocity(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    masses: &AcousticStepFinalizationMasses<'_, CpuField<f32>>,
    saved: &AcousticStepFinalizationSavedState<'_, CpuField<f32>>,
    map_factors: &AcousticStepFinalizationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepFinalizationCoefficients<'_>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let x_range = region.west_east_velocity_range();
    let y_range = region.mass_south_north_range();
    let z_range = region.mass_bottom_top_range();
    let saved_values = saved.west_east_velocity.values();
    let map_values = map_factors.west_east_velocity_y.values();
    let old_mass = masses.large_step_west_east.values();
    let new_mass = masses.final_west_east.values();
    for_each_volume_row(
        backend,
        output,
        &x_range,
        &y_range,
        &z_range,
        |index, x, z, y, value| {
            let horizontal_index = y * west_east_points + x;
            let old_weight = coefficients.half_level_mass_multiplier[z]
                * old_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            let new_weight = coefficients.half_level_mass_multiplier[z]
                * new_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            *value = (map_values[horizontal_index] * *value + saved_values[index] * old_weight)
                / new_weight;
        },
    )
}

fn finalize_vertical_velocity(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    masses: &AcousticStepFinalizationMasses<'_, CpuField<f32>>,
    saved: &AcousticStepFinalizationSavedState<'_, CpuField<f32>>,
    map_factors: &AcousticStepFinalizationMapFactors<'_, CpuField<f32>>,
    coefficients: AcousticStepFinalizationCoefficients<'_>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let x_range = region.mass_west_east_range();
    let y_range = region.mass_south_north_range();
    let z_range = region.full_bottom_top_range();
    let saved_values = saved.vertical_velocity.values();
    let map_values = map_factors.mass_point_y.values();
    let old_mass = masses.large_step_full.values();
    let new_mass = masses.final_full.values();
    for_each_volume_row(
        backend,
        output,
        &x_range,
        &y_range,
        &z_range,
        |index, x, z, y, value| {
            let horizontal_index = y * west_east_points + x;
            let old_weight = coefficients.full_level_mass_multiplier[z]
                * old_mass[horizontal_index]
                + coefficients.full_level_mass_offset[z];
            let new_weight = coefficients.full_level_mass_multiplier[z]
                * new_mass[horizontal_index]
                + coefficients.full_level_mass_offset[z];
            *value = (map_values[horizontal_index] * *value + saved_values[index] * old_weight)
                / new_weight;
        },
    )
}

fn add_saved_volume(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let x_range = region.mass_west_east_range();
    let y_range = region.mass_south_north_range();
    let z_range = region.full_bottom_top_range();
    let saved_values = saved.values();
    for_each_volume_row(
        backend,
        output,
        &x_range,
        &y_range,
        &z_range,
        |index, _, _, _, value| {
            *value += saved_values[index];
        },
    )
}

fn finalize_potential_temperature(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    masses: &AcousticStepFinalizationMasses<'_, CpuField<f32>>,
    saved: &AcousticStepFinalizationSavedState<'_, CpuField<f32>>,
    coefficients: AcousticStepFinalizationCoefficients<'_>,
    controls: AcousticStepFinalizationControls,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let x_range = region.mass_west_east_range();
    let y_range = region.mass_south_north_range();
    let z_range = region.mass_bottom_top_range();
    let saved_values = saved.potential_temperature.values();
    let heating = saved.diabatic_heating.values();
    let old_mass = masses.large_step_full.values();
    let new_mass = masses.final_full.values();
    for_each_volume_row(
        backend,
        output,
        &x_range,
        &y_range,
        &z_range,
        |index, x, z, y, value| {
            let horizontal_index = y * west_east_points + x;
            let old_weight = coefficients.half_level_mass_multiplier[z]
                * old_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            let new_weight = coefficients.half_level_mass_multiplier[z]
                * new_mass[horizontal_index]
                + coefficients.half_level_mass_offset[z];
            let coupled_value = match controls.phase {
                AcousticStepFinalizationPhase::Intermediate => *value,
                AcousticStepFinalizationPhase::Final => {
                    *value
                        - controls.acoustic_time_step
                            * controls.acoustic_substep_count as f32
                            * old_weight
                            * heating[index]
                }
            };
            *value = (coupled_value + saved_values[index] * old_weight) / new_weight;
        },
    )
}

fn add_saved_column_mass(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    region: &AcousticStepFinalizationRegion,
) -> AcousticStepFinalizationResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let x_range = region.mass_west_east_range();
    let y_range = region.mass_south_north_range();
    let saved_values = saved.values();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |south_north_index, output_row| {
                if y_range.contains(&south_north_index) {
                    let row_start = south_north_index * west_east_points;
                    for west_east_index in x_range.clone() {
                        output_row[west_east_index] += saved_values[row_start + west_east_index];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn for_each_volume_row<Operation>(
    backend: &CpuBackend,
    output: &mut CpuField<f32>,
    west_east_range: &Range<usize>,
    south_north_range: &Range<usize>,
    bottom_top_range: &Range<usize>,
    operation: Operation,
) -> AcousticStepFinalizationResult<()>
where
    Operation: Fn(usize, usize, usize, usize, &mut f32) + Send + Sync,
{
    let shape = output.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    backend
        .try_for_each_output_block(
            output.values_mut(),
            west_east_points,
            |row_index, output_row| {
                let south_north_index = row_index / bottom_top_points;
                let bottom_top_index = row_index % bottom_top_points;
                if south_north_range.contains(&south_north_index)
                    && bottom_top_range.contains(&bottom_top_index)
                {
                    let row_start = row_index * west_east_points;
                    for west_east_index in west_east_range.clone() {
                        let index = row_start + west_east_index;
                        operation(
                            index,
                            west_east_index,
                            bottom_top_index,
                            south_north_index,
                            &mut output_row[west_east_index],
                        );
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> AcousticStepFinalizationError {
    match error {
        ParallelExecutionError::Kernel(never) => match never {},
        ParallelExecutionError::WorkerPanicked => AcousticStepFinalizationError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            unreachable!("validated fields form complete nonempty rows")
        }
    }
}

#[cfg(test)]
mod tests {
    use wrf_compute::{ComputeBackend, GridShape};

    use super::*;

    #[test]
    fn reconstructs_finite_fields_with_exact_expected_bits() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::constant(&backend, GridShape::try_new(3, 3, 2).unwrap());

        fixture
            .execute(
                &backend,
                controls(AcousticStepFinalizationPhase::Intermediate),
            )
            .unwrap();

        assert_volume_bits(
            &fixture.west_east_velocity,
            &fixture.region.west_east_velocity_range(),
            &fixture.region.mass_south_north_range(),
            &fixture.region.mass_bottom_top_range(),
            4.0,
        );
        assert_volume_bits(
            &fixture.south_north_velocity,
            &fixture.region.mass_west_east_range(),
            &fixture.region.south_north_velocity_range(),
            &fixture.region.mass_bottom_top_range(),
            4.0,
        );
        for (field, expected) in [
            (&fixture.vertical_velocity, 4.0),
            (&fixture.perturbation_geopotential, 7.0),
            (&fixture.vertical_mass_flux, 9.0),
        ] {
            assert_volume_bits(
                field,
                &fixture.region.mass_west_east_range(),
                &fixture.region.mass_south_north_range(),
                &fixture.region.full_bottom_top_range(),
                expected,
            );
        }
        assert_volume_bits(
            &fixture.potential_temperature,
            &fixture.region.mass_west_east_range(),
            &fixture.region.mass_south_north_range(),
            &fixture.region.mass_bottom_top_range(),
            5.0,
        );
        assert_horizontal_bits(
            &fixture.perturbation_column_mass,
            &fixture.region.mass_west_east_range(),
            &fixture.region.mass_south_north_range(),
            7.0,
        );
        assert_eq!(
            fixture.potential_temperature.values()[index(3, 2, 2, 0, 0)].to_bits(),
            4.0_f32.to_bits(),
            "mass-point x stagger remains untouched"
        );
    }

    #[test]
    fn final_phase_applies_heating_in_pinned_fortran_order() {
        let backend = CpuBackend::try_with_worker_count(1).unwrap();
        let mut fixture = Fixture::constant(&backend, GridShape::try_new(3, 3, 2).unwrap());

        fixture
            .execute(&backend, controls(AcousticStepFinalizationPhase::Final))
            .unwrap();

        assert_eq!(
            fixture.potential_temperature.values()[0].to_bits(),
            4.75_f32.to_bits()
        );
    }

    #[test]
    fn minimum_domain_updates_each_upper_stagger() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::constant(&backend, GridShape::try_new(2, 2, 2).unwrap());

        fixture
            .execute(
                &backend,
                controls(AcousticStepFinalizationPhase::Intermediate),
            )
            .unwrap();

        assert_eq!(
            fixture.west_east_velocity.values()[index(2, 2, 1, 0, 0)].to_bits(),
            4.0_f32.to_bits()
        );
        assert_eq!(
            fixture.south_north_velocity.values()[index(2, 2, 0, 0, 1)].to_bits(),
            4.0_f32.to_bits()
        );
        assert_eq!(
            fixture.vertical_velocity.values()[index(2, 2, 0, 1, 0)].to_bits(),
            4.0_f32.to_bits()
        );
    }

    #[test]
    fn late_shape_failure_is_atomic_across_all_mutable_fields() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::constant(&backend, GridShape::try_new(3, 3, 2).unwrap());
        let invalid_heating = backend
            .create_field(GridShape::try_new(2, 3, 2).unwrap(), 0.5)
            .unwrap();
        let before = fixture.mutable_bits();

        let result = fixture.execute_with_heating(
            &backend,
            controls(AcousticStepFinalizationPhase::Final),
            &invalid_heating,
        );

        assert_eq!(
            result,
            Err(AcousticStepFinalizationError::FieldShapeMismatch {
                field: AcousticStepFinalizationField::DiabaticHeating,
                expected: GridShape::try_new(3, 3, 2).unwrap(),
                actual: GridShape::try_new(2, 3, 2).unwrap(),
            })
        );
        assert_eq!(fixture.mutable_bits(), before);
    }

    #[test]
    fn one_and_four_workers_produce_identical_complete_bits() {
        let one_worker = CpuBackend::try_with_worker_count(1).unwrap();
        let four_workers = CpuBackend::try_with_worker_count(4).unwrap();
        let mut serial = Fixture::patterned(&one_worker, GridShape::try_new(9, 8, 7).unwrap());
        let mut parallel = serial.clone();
        let controls = AcousticStepFinalizationControls::try_new(
            3,
            0.125,
            AcousticStepFinalizationPhase::Final,
        )
        .unwrap();

        serial.execute(&one_worker, controls).unwrap();
        parallel.execute(&four_workers, controls).unwrap();

        assert_eq!(serial.mutable_bits(), parallel.mutable_bits());
    }

    #[test]
    fn coefficient_failure_precedes_mutation() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut fixture = Fixture::constant(&backend, GridShape::try_new(3, 3, 2).unwrap());
        fixture.coefficients[3].pop();
        let before = fixture.mutable_bits();

        let result = fixture.execute(
            &backend,
            controls(AcousticStepFinalizationPhase::Intermediate),
        );

        assert_eq!(
            result,
            Err(AcousticStepFinalizationError::CoefficientLengthMismatch {
                coefficient: AcousticStepFinalizationCoefficient::FullLevelMassOffset,
                expected: 2,
                actual: 1,
            })
        );
        assert_eq!(fixture.mutable_bits(), before);
    }

    fn controls(phase: AcousticStepFinalizationPhase) -> AcousticStepFinalizationControls {
        AcousticStepFinalizationControls::try_new(2, 0.25, phase).unwrap()
    }

    #[derive(Clone)]
    struct Fixture {
        west_east_velocity: CpuField<f32>,
        south_north_velocity: CpuField<f32>,
        vertical_velocity: CpuField<f32>,
        potential_temperature: CpuField<f32>,
        perturbation_geopotential: CpuField<f32>,
        vertical_mass_flux: CpuField<f32>,
        perturbation_column_mass: CpuField<f32>,
        masses: [CpuField<f32>; 6],
        saved_volume: [CpuField<f32>; 6],
        saved_column_mass: CpuField<f32>,
        diabatic_heating: CpuField<f32>,
        map_factors: [CpuField<f32>; 3],
        coefficients: [Vec<f32>; 4],
        region: AcousticStepFinalizationRegion,
    }

    impl Fixture {
        fn constant(backend: &CpuBackend, shape: GridShape) -> Self {
            let volume = |value| backend.create_field(shape, value).unwrap();
            let horizontal = |value| {
                backend
                    .create_field(shape.horizontal_shape(), value)
                    .unwrap()
            };
            let region = AcousticStepFinalizationRegion::try_new(
                shape,
                0..shape.west_east_points() - 1,
                0..shape.south_north_points() - 1,
                0..shape.bottom_top_points() - 1,
                0..shape.west_east_points(),
                0..shape.south_north_points(),
            )
            .unwrap();
            Self {
                west_east_velocity: volume(3.0),
                south_north_velocity: volume(3.0),
                vertical_velocity: volume(3.0),
                potential_temperature: volume(4.0),
                perturbation_geopotential: volume(5.0),
                vertical_mass_flux: volume(8.0),
                perturbation_column_mass: horizontal(4.0),
                masses: std::array::from_fn(|_| horizontal(2.0)),
                saved_volume: [
                    volume(1.0),
                    volume(1.0),
                    volume(1.0),
                    volume(3.0),
                    volume(2.0),
                    volume(1.0),
                ],
                saved_column_mass: horizontal(3.0),
                diabatic_heating: volume(0.5),
                map_factors: std::array::from_fn(|_| horizontal(2.0)),
                coefficients: [
                    vec![0.5; shape.bottom_top_points()],
                    vec![1.0; shape.bottom_top_points()],
                    vec![0.5; shape.bottom_top_points()],
                    vec![1.0; shape.bottom_top_points()],
                ],
                region,
            }
        }

        fn patterned(backend: &CpuBackend, shape: GridShape) -> Self {
            let mut fixture = Self::constant(backend, shape);
            for (field_number, field) in [
                &mut fixture.west_east_velocity,
                &mut fixture.south_north_velocity,
                &mut fixture.vertical_velocity,
                &mut fixture.potential_temperature,
                &mut fixture.perturbation_geopotential,
                &mut fixture.vertical_mass_flux,
            ]
            .into_iter()
            .enumerate()
            {
                for (index, value) in field.values_mut().iter_mut().enumerate() {
                    *value += field_number as f32 * 0.03125 + index as f32 * 0.000_244_140_63;
                }
            }
            fixture
        }

        fn execute(
            &mut self,
            backend: &CpuBackend,
            controls: AcousticStepFinalizationControls,
        ) -> AcousticStepFinalizationResult<()> {
            let heating = self.diabatic_heating.clone();
            self.execute_with_heating(backend, controls, &heating)
        }

        fn execute_with_heating(
            &mut self,
            backend: &CpuBackend,
            controls: AcousticStepFinalizationControls,
            heating: &CpuField<f32>,
        ) -> AcousticStepFinalizationResult<()> {
            backend.finalize_acoustic_step(
                AcousticStepFinalizationState::new(
                    &mut self.west_east_velocity,
                    &mut self.south_north_velocity,
                    &mut self.vertical_velocity,
                    &mut self.potential_temperature,
                    &mut self.perturbation_geopotential,
                    &mut self.vertical_mass_flux,
                    &mut self.perturbation_column_mass,
                ),
                AcousticStepFinalizationMasses::new(
                    &self.masses[0],
                    &self.masses[1],
                    &self.masses[2],
                    &self.masses[3],
                    &self.masses[4],
                    &self.masses[5],
                ),
                AcousticStepFinalizationSavedState::new(
                    &self.saved_volume[0],
                    &self.saved_volume[1],
                    &self.saved_volume[2],
                    &self.saved_volume[3],
                    &self.saved_volume[4],
                    &self.saved_column_mass,
                    &self.saved_volume[5],
                    heating,
                ),
                AcousticStepFinalizationMapFactors::new(
                    &self.map_factors[0],
                    &self.map_factors[1],
                    &self.map_factors[2],
                ),
                AcousticStepFinalizationCoefficients::new(
                    &self.coefficients[0],
                    &self.coefficients[1],
                    &self.coefficients[2],
                    &self.coefficients[3],
                ),
                controls,
                &self.region,
            )
        }

        fn mutable_bits(&self) -> Vec<u32> {
            [
                &self.west_east_velocity,
                &self.south_north_velocity,
                &self.vertical_velocity,
                &self.potential_temperature,
                &self.perturbation_geopotential,
                &self.vertical_mass_flux,
                &self.perturbation_column_mass,
            ]
            .into_iter()
            .flat_map(|field| field.values().iter().map(|value| value.to_bits()))
            .collect()
        }
    }

    const fn index(
        west_east_points: usize,
        bottom_top_points: usize,
        west_east: usize,
        bottom_top: usize,
        south_north: usize,
    ) -> usize {
        (south_north * bottom_top_points + bottom_top) * west_east_points + west_east
    }

    fn assert_volume_bits(
        field: &CpuField<f32>,
        west_east: &Range<usize>,
        south_north: &Range<usize>,
        bottom_top: &Range<usize>,
        expected: f32,
    ) {
        let shape = field.shape();
        for y in south_north.clone() {
            for z in bottom_top.clone() {
                for x in west_east.clone() {
                    assert_eq!(
                        field.values()
                            [index(shape.west_east_points(), shape.bottom_top_points(), x, z, y,)]
                        .to_bits(),
                        expected.to_bits()
                    );
                }
            }
        }
    }

    fn assert_horizontal_bits(
        field: &CpuField<f32>,
        west_east: &Range<usize>,
        south_north: &Range<usize>,
        expected: f32,
    ) {
        let west_east_points = field.shape().west_east_points();
        for y in south_north.clone() {
            for x in west_east.clone() {
                assert_eq!(
                    field.values()[y * west_east_points + x].to_bits(),
                    expected.to_bits()
                );
            }
        }
    }
}
