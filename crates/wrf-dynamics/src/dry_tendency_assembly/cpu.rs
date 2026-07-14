use std::convert::Infallible;

use wrf_compute::{CpuBackend, CpuField, FieldStorage, GridShape, ParallelExecutionError};

use crate::{
    DryTendencyAssemblyCoefficient, DryTendencyAssemblyCoefficients, DryTendencyAssemblyError,
    DryTendencyAssemblyField, DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyKernels,
    DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase, DryTendencyAssemblyRegion,
    DryTendencyAssemblyResult, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
};

impl DryTendencyAssemblyKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn assemble_dry_tendencies(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        forward: DryTendencyAssemblyForwardTendencies<'_, Self::Field>,
        saved: DryTendencyAssemblySavedTendencies<'_, Self::Field>,
        thermodynamics: DryTendencyAssemblyThermodynamics<'_, Self::Field>,
        map_factors: DryTendencyAssemblyMapFactors<'_, Self::Field>,
        coefficients: DryTendencyAssemblyCoefficients<'_>,
        phase: DryTendencyAssemblyPhase,
        region: &DryTendencyAssemblyRegion,
    ) -> DryTendencyAssemblyResult<()> {
        validate_cpu_dry_tendency_assembly(
            &runge_kutta,
            &forward,
            &saved,
            &thermodynamics,
            &map_factors,
            coefficients,
            region,
        )?;

        let DryTendencyAssemblyRungeKuttaTendencies {
            west_east_momentum: west_east_runge_kutta,
            south_north_momentum: south_north_runge_kutta,
            vertical_momentum: vertical_runge_kutta,
            geopotential: geopotential_runge_kutta,
            potential_temperature: potential_temperature_runge_kutta,
            column_mass: column_mass_runge_kutta,
        } = runge_kutta;
        let DryTendencyAssemblyForwardTendencies {
            west_east_momentum: west_east_forward,
            south_north_momentum: south_north_forward,
            vertical_momentum: vertical_forward,
            geopotential: geopotential_forward,
            potential_temperature: potential_temperature_forward,
            column_mass: column_mass_forward,
        } = forward;

        assemble_west_east_momentum(
            self,
            west_east_runge_kutta,
            west_east_forward,
            saved.west_east_momentum,
            map_factors.west_east_momentum_south_north,
            phase,
            region,
        )?;
        assemble_south_north_momentum(
            self,
            south_north_runge_kutta,
            south_north_forward,
            saved.south_north_momentum,
            map_factors.south_north_momentum_west_east,
            map_factors.inverse_south_north_momentum_west_east,
            phase,
            region,
        )?;
        assemble_vertical_momentum(
            self,
            vertical_runge_kutta,
            vertical_forward,
            saved.vertical_momentum,
            map_factors.mass_point_south_north,
            phase,
            region,
        )?;
        assemble_geopotential(
            self,
            geopotential_runge_kutta,
            geopotential_forward,
            saved.geopotential,
            map_factors.mass_point_south_north,
            phase,
            region,
        )?;
        assemble_potential_temperature(
            self,
            potential_temperature_runge_kutta,
            potential_temperature_forward,
            saved.potential_temperature,
            thermodynamics,
            map_factors.mass_point_south_north,
            coefficients,
            phase,
            region,
        )?;
        assemble_column_mass(self, column_mass_runge_kutta, column_mass_forward, region)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_cpu_dry_tendency_assembly(
    runge_kutta: &DryTendencyAssemblyRungeKuttaTendencies<'_, CpuField<f32>>,
    forward: &DryTendencyAssemblyForwardTendencies<'_, CpuField<f32>>,
    saved: &DryTendencyAssemblySavedTendencies<'_, CpuField<f32>>,
    thermodynamics: &DryTendencyAssemblyThermodynamics<'_, CpuField<f32>>,
    map_factors: &DryTendencyAssemblyMapFactors<'_, CpuField<f32>>,
    coefficients: DryTendencyAssemblyCoefficients<'_>,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let volume_shape = region.shape();
    let horizontal_shape = volume_shape.horizontal_shape();
    for (field, role) in [
        (
            &*runge_kutta.west_east_momentum,
            DryTendencyAssemblyField::WestEastRungeKuttaMomentum,
        ),
        (
            &*runge_kutta.south_north_momentum,
            DryTendencyAssemblyField::SouthNorthRungeKuttaMomentum,
        ),
        (
            &*runge_kutta.vertical_momentum,
            DryTendencyAssemblyField::VerticalRungeKuttaMomentum,
        ),
        (
            &*runge_kutta.geopotential,
            DryTendencyAssemblyField::RungeKuttaGeopotential,
        ),
        (
            &*runge_kutta.potential_temperature,
            DryTendencyAssemblyField::RungeKuttaPotentialTemperature,
        ),
        (
            &*forward.west_east_momentum,
            DryTendencyAssemblyField::WestEastForwardMomentum,
        ),
        (
            &*forward.south_north_momentum,
            DryTendencyAssemblyField::SouthNorthForwardMomentum,
        ),
        (
            &*forward.vertical_momentum,
            DryTendencyAssemblyField::VerticalForwardMomentum,
        ),
        (
            &*forward.geopotential,
            DryTendencyAssemblyField::ForwardGeopotential,
        ),
        (
            &*forward.potential_temperature,
            DryTendencyAssemblyField::ForwardPotentialTemperature,
        ),
        (
            saved.west_east_momentum,
            DryTendencyAssemblyField::SavedWestEastMomentum,
        ),
        (
            saved.south_north_momentum,
            DryTendencyAssemblyField::SavedSouthNorthMomentum,
        ),
        (
            saved.vertical_momentum,
            DryTendencyAssemblyField::SavedVerticalMomentum,
        ),
        (
            saved.geopotential,
            DryTendencyAssemblyField::SavedGeopotential,
        ),
        (
            saved.potential_temperature,
            DryTendencyAssemblyField::SavedPotentialTemperature,
        ),
        (
            thermodynamics.diabatic_heating,
            DryTendencyAssemblyField::DiabaticHeating,
        ),
    ] {
        validate_shape(field, role, volume_shape)?;
    }
    for (field, role) in [
        (
            &*runge_kutta.column_mass,
            DryTendencyAssemblyField::RungeKuttaColumnMass,
        ),
        (
            forward.column_mass,
            DryTendencyAssemblyField::ForwardColumnMass,
        ),
        (
            thermodynamics.full_column_mass,
            DryTendencyAssemblyField::FullColumnMass,
        ),
        (
            map_factors.west_east_momentum_south_north,
            DryTendencyAssemblyField::WestEastMomentumSouthNorthMapFactor,
        ),
        (
            map_factors.south_north_momentum_west_east,
            DryTendencyAssemblyField::SouthNorthMomentumWestEastMapFactor,
        ),
        (
            map_factors.inverse_south_north_momentum_west_east,
            DryTendencyAssemblyField::InverseSouthNorthMomentumWestEastMapFactor,
        ),
        (
            map_factors.mass_point_south_north,
            DryTendencyAssemblyField::MassPointSouthNorthMapFactor,
        ),
    ] {
        validate_shape(field, role, horizontal_shape)?;
    }
    for (values, coefficient) in [
        (
            coefficients.full_mass_multiplier,
            DryTendencyAssemblyCoefficient::FullMassMultiplier,
        ),
        (
            coefficients.vertical_offset,
            DryTendencyAssemblyCoefficient::VerticalOffset,
        ),
    ] {
        if values.len() != volume_shape.bottom_top_points() {
            return Err(DryTendencyAssemblyError::CoefficientLengthMismatch {
                coefficient,
                expected: volume_shape.bottom_top_points(),
                actual: values.len(),
            });
        }
    }
    Ok(())
}

fn validate_shape(
    field: &CpuField<f32>,
    role: DryTendencyAssemblyField,
    expected: GridShape,
) -> DryTendencyAssemblyResult<()> {
    let actual = field.shape();
    if actual != expected {
        return Err(DryTendencyAssemblyError::FieldShapeMismatch {
            field: role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn assemble_west_east_momentum(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    phase: DryTendencyAssemblyPhase,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let saved = saved.values();
    let map_factor = map_factor.values();
    for_each_volume_pair(
        backend,
        runge_kutta,
        forward,
        region.west_east_momentum_ranges(),
        |index, horizontal, _, rk, persistent| {
            let factor = map_factor[horizontal];
            if phase.adds_saved_tendencies() {
                *persistent += saved[index] * factor;
            }
            *rk += *persistent / factor;
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn assemble_south_north_momentum(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    inverse_map_factor: &CpuField<f32>,
    phase: DryTendencyAssemblyPhase,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let saved = saved.values();
    let map_factor = map_factor.values();
    let inverse_map_factor = inverse_map_factor.values();
    for_each_volume_pair(
        backend,
        runge_kutta,
        forward,
        region.south_north_momentum_ranges(),
        |index, horizontal, _, rk, persistent| {
            if phase.adds_saved_tendencies() {
                *persistent += saved[index] * map_factor[horizontal];
            }
            *rk += *persistent * inverse_map_factor[horizontal];
        },
    )
}

fn assemble_vertical_momentum(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    phase: DryTendencyAssemblyPhase,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let saved = saved.values();
    let map_factor = map_factor.values();
    for_each_volume_pair(
        backend,
        runge_kutta,
        forward,
        region.vertical_ranges(),
        |index, horizontal, _, rk, persistent| {
            let factor = map_factor[horizontal];
            if phase.adds_saved_tendencies() {
                *persistent += saved[index] * factor;
            }
            *rk += *persistent / factor;
        },
    )
}

fn assemble_geopotential(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    map_factor: &CpuField<f32>,
    phase: DryTendencyAssemblyPhase,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let saved = saved.values();
    let map_factor = map_factor.values();
    for_each_volume_pair(
        backend,
        runge_kutta,
        forward,
        region.vertical_ranges(),
        |index, horizontal, _, rk, persistent| {
            if phase.adds_saved_tendencies() {
                *persistent += saved[index];
            }
            *rk += *persistent / map_factor[horizontal];
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn assemble_potential_temperature(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &mut CpuField<f32>,
    saved: &CpuField<f32>,
    thermodynamics: DryTendencyAssemblyThermodynamics<'_, CpuField<f32>>,
    map_factor: &CpuField<f32>,
    coefficients: DryTendencyAssemblyCoefficients<'_>,
    phase: DryTendencyAssemblyPhase,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let saved = saved.values();
    let heating = thermodynamics.diabatic_heating.values();
    let full_mass = thermodynamics.full_column_mass.values();
    let map_factor = map_factor.values();
    for_each_volume_pair(
        backend,
        runge_kutta,
        forward,
        region.mass_ranges(),
        |index, horizontal, vertical, rk, persistent| {
            if phase.adds_saved_tendencies() {
                *persistent += saved[index];
            }
            let factor = map_factor[horizontal];
            *rk = *rk
                + *persistent / factor
                + (coefficients.full_mass_multiplier[vertical] * full_mass[horizontal]
                    + coefficients.vertical_offset[vertical])
                    * heating[index]
                    / factor;
        },
    )
}

fn assemble_column_mass(
    backend: &CpuBackend,
    runge_kutta: &mut CpuField<f32>,
    forward: &CpuField<f32>,
    region: &DryTendencyAssemblyRegion,
) -> DryTendencyAssemblyResult<()> {
    let shape = region.shape();
    let west_east_points = shape.west_east_points();
    let ranges = region.mass_ranges();
    let forward = forward.values();
    backend
        .try_for_each_output_block(
            runge_kutta.values_mut(),
            west_east_points,
            |south_north, row| {
                if ranges.south_north.contains(&south_north) {
                    let row_start = south_north * west_east_points;
                    for west_east in ranges.west_east.clone() {
                        row[west_east] += forward[row_start + west_east];
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn for_each_volume_pair<Operation>(
    backend: &CpuBackend,
    first: &mut CpuField<f32>,
    second: &mut CpuField<f32>,
    ranges: super::region::DryTendencyAssemblyActiveRanges,
    operation: Operation,
) -> DryTendencyAssemblyResult<()>
where
    Operation: Fn(usize, usize, usize, &mut f32, &mut f32) + Send + Sync,
{
    let shape = first.shape();
    let west_east_points = shape.west_east_points();
    let bottom_top_points = shape.bottom_top_points();
    backend
        .try_for_each_output_pair_block(
            first.values_mut(),
            second.values_mut(),
            west_east_points,
            |line_index, first_row, second_row| {
                let south_north = line_index / bottom_top_points;
                let bottom_top = line_index % bottom_top_points;
                if ranges.south_north.contains(&south_north)
                    && ranges.bottom_top.contains(&bottom_top)
                {
                    let row_start = line_index * west_east_points;
                    let horizontal_row_start = south_north * west_east_points;
                    for west_east in ranges.west_east.clone() {
                        operation(
                            row_start + west_east,
                            horizontal_row_start + west_east,
                            bottom_top,
                            &mut first_row[west_east],
                            &mut second_row[west_east],
                        );
                    }
                }
                Ok::<(), Infallible>(())
            },
        )
        .map_err(map_parallel_error)
}

fn map_parallel_error(error: ParallelExecutionError<Infallible>) -> DryTendencyAssemblyError {
    match error {
        ParallelExecutionError::WorkerPanicked => DryTendencyAssemblyError::WorkerPanicked,
        ParallelExecutionError::ZeroBlockLength
        | ParallelExecutionError::IncompleteOutputBlock { .. }
        | ParallelExecutionError::PairedOutputLengthMismatch { .. } => {
            DryTendencyAssemblyError::SchedulerContractViolated
        }
        ParallelExecutionError::Kernel(unreachable) => match unreachable {},
    }
}
