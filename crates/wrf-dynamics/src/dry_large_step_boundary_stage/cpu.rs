use wrf_compute::{CpuBackend, CpuField};

use super::{
    DryLargeStepBoundaryStageControls, DryLargeStepBoundaryStageInputs,
    DryLargeStepBoundaryStageKernels, DryLargeStepBoundaryStageMode,
    DryLargeStepBoundaryStageRegions, DryLargeStepBoundaryStageResult, DryLargeStepNestedVertical,
    DryLargeStepRelaxationBoundaryValues, DryLargeStepRelaxationInputs,
    DryLargeStepSavedTendencies,
};
use crate::dry_tendency_assembly::validate_cpu_dry_tendency_assembly;
use crate::specified_boundary_update::{
    validate_cpu_dry_boundary_relaxation, validate_cpu_dry_boundary_tendency_assignment,
};
use crate::{
    DryBoundaryRelaxationBoundaryData, DryBoundaryRelaxationBoundaryFields,
    DryBoundaryRelaxationKernels, DryBoundaryRelaxationTendencies, DryBoundaryTendencies,
    DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyKernels, DryBoundaryVerticalRelaxation,
    DryBoundaryVerticalTendency, DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyKernels,
    DryTendencyAssemblyPhase, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies,
};

impl DryLargeStepBoundaryStageKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_dry_large_step_boundary_stage(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        saved: DryLargeStepSavedTendencies<'_, Self::Field>,
        inputs: DryLargeStepBoundaryStageInputs<'_, Self::Field>,
        mode: DryLargeStepBoundaryStageMode<'_, Self::Field>,
        controls: DryLargeStepBoundaryStageControls,
        regions: &DryLargeStepBoundaryStageRegions,
    ) -> DryLargeStepBoundaryStageResult<()> {
        let DryTendencyAssemblyRungeKuttaTendencies {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
            column_mass,
        } = runge_kutta;
        let DryLargeStepSavedTendencies {
            west_east_momentum: west_east_saved,
            south_north_momentum: south_north_saved,
            vertical_momentum: vertical_saved,
            geopotential: geopotential_saved,
            potential_temperature: potential_temperature_saved,
        } = saved;
        let DryLargeStepBoundaryStageInputs {
            forward,
            thermodynamics,
            map_factors,
            coefficients,
            boundaries,
        } = inputs;
        let DryTendencyAssemblyForwardTendencies {
            west_east_momentum: west_east_forward,
            south_north_momentum: south_north_forward,
            vertical_momentum: vertical_forward,
            geopotential: geopotential_forward,
            potential_temperature: potential_temperature_forward,
            column_mass: column_mass_forward,
        } = forward;

        let (phase, relaxation, vertical_relaxation, vertical_boundaries) = match mode {
            DryLargeStepBoundaryStageMode::FirstSubstepGlobal { relaxation } => (
                DryTendencyAssemblyPhase::FirstSubstep,
                Some(relaxation),
                None,
                None,
            ),
            DryLargeStepBoundaryStageMode::FirstSubstepNested {
                relaxation,
                vertical,
            } => {
                let DryLargeStepNestedVertical {
                    velocity,
                    boundary_values,
                    boundary_tendencies,
                } = vertical;
                (
                    DryTendencyAssemblyPhase::FirstSubstep,
                    Some(relaxation),
                    Some((
                        velocity,
                        DryBoundaryRelaxationBoundaryData::new(
                            boundary_values,
                            boundary_tendencies,
                        ),
                    )),
                    Some(boundary_tendencies),
                )
            }
            DryLargeStepBoundaryStageMode::LaterSubstepGlobal => {
                (DryTendencyAssemblyPhase::LaterSubstep, None, None, None)
            }
            DryLargeStepBoundaryStageMode::LaterSubstepNested {
                vertical_boundaries,
            } => (
                DryTendencyAssemblyPhase::LaterSubstep,
                None,
                None,
                Some(vertical_boundaries),
            ),
        };

        if let Some(relaxation) = &relaxation {
            let relaxation_tendencies = DryBoundaryRelaxationTendencies::new(
                &mut *west_east_saved,
                &mut *south_north_saved,
                &mut *geopotential_saved,
                &mut *potential_temperature_saved,
                &mut *column_mass,
            );
            let vertical = match &vertical_relaxation {
                None => DryBoundaryVerticalRelaxation::Disabled,
                Some((velocity, boundary)) => DryBoundaryVerticalRelaxation::Nested {
                    velocity: *velocity,
                    tendency: &mut *vertical_saved,
                    boundary: *boundary,
                },
            };
            validate_cpu_dry_boundary_relaxation(
                &relaxation_tendencies,
                &relaxation.state,
                relaxation_boundary_fields(relaxation.boundary_values, boundaries),
                &vertical,
                &*relaxation.workspace.mass_weighted_field,
                relaxation.mass_coefficients,
                relaxation.relaxation_coefficients,
                relaxation.parameters,
                controls.west_east_periodicity,
                &regions.relaxation,
            )?;
        }

        validate_cpu_dry_tendency_assembly(
            &DryTendencyAssemblyRungeKuttaTendencies::new(
                &mut *west_east_momentum,
                &mut *south_north_momentum,
                &mut *vertical_momentum,
                &mut *geopotential,
                &mut *potential_temperature,
                &mut *column_mass,
            ),
            &DryTendencyAssemblyForwardTendencies::new(
                &mut *west_east_forward,
                &mut *south_north_forward,
                &mut *vertical_forward,
                &mut *geopotential_forward,
                &mut *potential_temperature_forward,
                column_mass_forward,
            ),
            &DryTendencyAssemblySavedTendencies::new(
                &*west_east_saved,
                &*south_north_saved,
                &*vertical_saved,
                &*geopotential_saved,
                &*potential_temperature_saved,
            ),
            &thermodynamics,
            &map_factors,
            coefficients,
            &regions.assembly,
        )?;

        let assignment_tendencies = DryBoundaryTendencies::new(
            &mut *west_east_momentum,
            &mut *south_north_momentum,
            &mut *geopotential,
            &mut *potential_temperature,
            &mut *column_mass,
        );
        match &vertical_boundaries {
            None => validate_cpu_dry_boundary_tendency_assignment(
                &assignment_tendencies,
                boundaries,
                &DryBoundaryVerticalTendency::Disabled,
                controls.boundary_parameters,
                &regions.boundary_assignment,
            )?,
            Some(slabs) => validate_cpu_dry_boundary_tendency_assignment(
                &assignment_tendencies,
                boundaries,
                &DryBoundaryVerticalTendency::Nested {
                    tendency: &mut *vertical_momentum,
                    boundaries: *slabs,
                },
                controls.boundary_parameters,
                &regions.boundary_assignment,
            )?,
        }

        if let Some(relaxation) = relaxation {
            let DryLargeStepRelaxationInputs {
                state,
                boundary_values,
                workspace,
                mass_coefficients,
                relaxation_coefficients,
                parameters,
            } = relaxation;
            let vertical = match vertical_relaxation {
                None => DryBoundaryVerticalRelaxation::Disabled,
                Some((velocity, boundary)) => DryBoundaryVerticalRelaxation::Nested {
                    velocity,
                    tendency: &mut *vertical_saved,
                    boundary,
                },
            };
            self.add_dry_boundary_relaxation_tendencies(
                DryBoundaryRelaxationTendencies::new(
                    &mut *west_east_saved,
                    &mut *south_north_saved,
                    &mut *geopotential_saved,
                    &mut *potential_temperature_saved,
                    &mut *column_mass,
                ),
                state,
                relaxation_boundary_fields(boundary_values, boundaries),
                vertical,
                workspace,
                mass_coefficients,
                relaxation_coefficients,
                parameters,
                controls.west_east_periodicity,
                &regions.relaxation,
            )?;
        }

        self.assemble_dry_tendencies(
            DryTendencyAssemblyRungeKuttaTendencies::new(
                &mut *west_east_momentum,
                &mut *south_north_momentum,
                &mut *vertical_momentum,
                &mut *geopotential,
                &mut *potential_temperature,
                &mut *column_mass,
            ),
            DryTendencyAssemblyForwardTendencies::new(
                west_east_forward,
                south_north_forward,
                vertical_forward,
                geopotential_forward,
                potential_temperature_forward,
                column_mass_forward,
            ),
            DryTendencyAssemblySavedTendencies::new(
                west_east_saved,
                south_north_saved,
                vertical_saved,
                geopotential_saved,
                potential_temperature_saved,
            ),
            thermodynamics,
            map_factors,
            coefficients,
            phase,
            &regions.assembly,
        )?;

        let vertical = match vertical_boundaries {
            None => DryBoundaryVerticalTendency::Disabled,
            Some(slabs) => DryBoundaryVerticalTendency::Nested {
                tendency: vertical_momentum,
                boundaries: slabs,
            },
        };
        self.assign_dry_boundary_tendencies(
            DryBoundaryTendencies::new(
                west_east_momentum,
                south_north_momentum,
                geopotential,
                potential_temperature,
                column_mass,
            ),
            boundaries,
            vertical,
            controls.boundary_parameters,
            controls.west_east_periodicity,
            &regions.boundary_assignment,
        )?;
        Ok(())
    }
}

fn relaxation_boundary_fields<'a>(
    values: DryLargeStepRelaxationBoundaryValues<'a, CpuField<f32>>,
    tendencies: DryBoundaryTendencyBoundaryFields<'a, CpuField<f32>>,
) -> DryBoundaryRelaxationBoundaryFields<'a, CpuField<f32>> {
    DryBoundaryRelaxationBoundaryFields::new(
        DryBoundaryRelaxationBoundaryData::new(
            values.west_east_momentum,
            tendencies.west_east_momentum,
        ),
        DryBoundaryRelaxationBoundaryData::new(
            values.south_north_momentum,
            tendencies.south_north_momentum,
        ),
        DryBoundaryRelaxationBoundaryData::new(
            values.perturbation_geopotential,
            tendencies.perturbation_geopotential,
        ),
        DryBoundaryRelaxationBoundaryData::new(
            values.potential_temperature,
            tendencies.potential_temperature,
        ),
        DryBoundaryRelaxationBoundaryData::new(
            values.perturbation_column_mass,
            tendencies.perturbation_column_mass,
        ),
    )
}
