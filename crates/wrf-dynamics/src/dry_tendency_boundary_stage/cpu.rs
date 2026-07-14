use wrf_compute::{CpuBackend, CpuField};

use super::{
    DryTendencyBoundaryStageControls, DryTendencyBoundaryStageInputs,
    DryTendencyBoundaryStageKernels, DryTendencyBoundaryStageRegions,
    DryTendencyBoundaryStageResult, DryTendencyBoundaryStageVertical,
};
use crate::dry_tendency_assembly::validate_cpu_dry_tendency_assembly;
use crate::specified_boundary_update::validate_cpu_dry_boundary_tendency_assignment;
use crate::{
    DryBoundaryTendencies, DryBoundaryTendencyKernels, DryBoundaryVerticalTendency,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyKernels,
    DryTendencyAssemblyRungeKuttaTendencies,
};

impl DryTendencyBoundaryStageKernels for CpuBackend {
    type Field = CpuField<f32>;

    fn advance_dry_tendency_boundary_stage(
        &self,
        runge_kutta: DryTendencyAssemblyRungeKuttaTendencies<'_, Self::Field>,
        inputs: DryTendencyBoundaryStageInputs<'_, Self::Field>,
        vertical: DryTendencyBoundaryStageVertical<'_, Self::Field>,
        controls: DryTendencyBoundaryStageControls,
        regions: &DryTendencyBoundaryStageRegions,
    ) -> DryTendencyBoundaryStageResult<()> {
        let DryTendencyAssemblyRungeKuttaTendencies {
            west_east_momentum,
            south_north_momentum,
            vertical_momentum,
            geopotential,
            potential_temperature,
            column_mass,
        } = runge_kutta;
        let DryTendencyBoundaryStageInputs {
            forward,
            saved,
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
            &saved,
            &thermodynamics,
            &map_factors,
            coefficients,
            &regions.assembly,
        )?;

        let boundary_tendencies = DryBoundaryTendencies::new(
            &mut *west_east_momentum,
            &mut *south_north_momentum,
            &mut *geopotential,
            &mut *potential_temperature,
            &mut *column_mass,
        );
        let mut vertical = vertical;
        match &mut vertical {
            DryTendencyBoundaryStageVertical::Global => {
                validate_cpu_dry_boundary_tendency_assignment(
                    &boundary_tendencies,
                    boundaries,
                    &DryBoundaryVerticalTendency::Disabled,
                    controls.boundary_parameters,
                    &regions.boundary_assignment,
                )?;
            }
            DryTendencyBoundaryStageVertical::Nested {
                boundaries: vertical_boundaries,
            } => {
                validate_cpu_dry_boundary_tendency_assignment(
                    &boundary_tendencies,
                    boundaries,
                    &DryBoundaryVerticalTendency::Nested {
                        tendency: &mut *vertical_momentum,
                        boundaries: *vertical_boundaries,
                    },
                    controls.boundary_parameters,
                    &regions.boundary_assignment,
                )?;
            }
        }

        self.assemble_dry_tendencies(
            DryTendencyAssemblyRungeKuttaTendencies::new(
                west_east_momentum,
                south_north_momentum,
                vertical_momentum,
                geopotential,
                potential_temperature,
                column_mass,
            ),
            DryTendencyAssemblyForwardTendencies::new(
                west_east_forward,
                south_north_forward,
                vertical_forward,
                geopotential_forward,
                potential_temperature_forward,
                column_mass_forward,
            ),
            saved,
            thermodynamics,
            map_factors,
            coefficients,
            controls.phase,
            &regions.assembly,
        )?;

        let vertical = match vertical {
            DryTendencyBoundaryStageVertical::Global => DryBoundaryVerticalTendency::Disabled,
            DryTendencyBoundaryStageVertical::Nested { boundaries } => {
                DryBoundaryVerticalTendency::Nested {
                    tendency: vertical_momentum,
                    boundaries,
                }
            }
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
