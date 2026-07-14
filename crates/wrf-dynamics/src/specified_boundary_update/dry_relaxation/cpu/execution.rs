use wrf_compute::{CpuBackend, CpuField};

use super::super::{
    DryBoundaryRelaxationBoundaryFields, DryBoundaryRelaxationError,
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationRegion,
    DryBoundaryRelaxationResult, DryBoundaryRelaxationState, DryBoundaryRelaxationTarget,
    DryBoundaryRelaxationTendencies, DryBoundaryRelaxationWorkspace, DryBoundaryVerticalRelaxation,
};
use super::inputs::{full_field_inputs, workspace_inputs};
use super::mass_weighting::{DryBoundaryMassWeightingCpuKernel, DryBoundaryMassWeightingInputs};
use super::validation::validate_operation;
use crate::specified_boundary_update::relaxation::has_relaxation_updates;
use crate::{
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationInputs,
    SpecifiedBoundaryRelaxationKernels, SpecifiedBoundaryRelaxationParameters,
    SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct DryBoundaryRelaxationCpuExecution<'a, 'coefficients, 'region> {
    backend: &'a CpuBackend,
    tendencies: DryBoundaryRelaxationTendencies<'a, CpuField<f32>>,
    state: DryBoundaryRelaxationState<'a, CpuField<f32>>,
    boundaries: DryBoundaryRelaxationBoundaryFields<'a, CpuField<f32>>,
    vertical: DryBoundaryVerticalRelaxation<'a, CpuField<f32>>,
    workspace: DryBoundaryRelaxationWorkspace<'a, CpuField<f32>>,
    mass_coefficients: DryBoundaryRelaxationMassCoefficients<'coefficients>,
    relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'coefficients>,
    parameters: SpecifiedBoundaryRelaxationParameters,
    west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    region: &'region DryBoundaryRelaxationRegion,
}

impl<'a, 'coefficients, 'region> DryBoundaryRelaxationCpuExecution<'a, 'coefficients, 'region> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        backend: &'a CpuBackend,
        tendencies: DryBoundaryRelaxationTendencies<'a, CpuField<f32>>,
        state: DryBoundaryRelaxationState<'a, CpuField<f32>>,
        boundaries: DryBoundaryRelaxationBoundaryFields<'a, CpuField<f32>>,
        vertical: DryBoundaryVerticalRelaxation<'a, CpuField<f32>>,
        workspace: DryBoundaryRelaxationWorkspace<'a, CpuField<f32>>,
        mass_coefficients: DryBoundaryRelaxationMassCoefficients<'coefficients>,
        relaxation_coefficients: SpecifiedBoundaryRelaxationCoefficients<'coefficients>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &'region DryBoundaryRelaxationRegion,
    ) -> Self {
        Self {
            backend,
            tendencies,
            state,
            boundaries,
            vertical,
            workspace,
            mass_coefficients,
            relaxation_coefficients,
            parameters,
            west_east_periodicity,
            region,
        }
    }

    pub(super) fn run(self) -> DryBoundaryRelaxationResult<()> {
        validate_operation(
            &self.tendencies,
            &self.state,
            self.boundaries,
            &self.vertical,
            self.workspace.mass_weighted_field,
            self.mass_coefficients,
            self.relaxation_coefficients,
            self.parameters,
            self.west_east_periodicity,
            self.region,
        )?;

        let Self {
            backend,
            tendencies,
            state,
            boundaries,
            vertical,
            workspace,
            mass_coefficients,
            relaxation_coefficients,
            parameters,
            west_east_periodicity,
            region,
        } = self;
        let DryBoundaryRelaxationTendencies {
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
        } = tendencies;
        let scratch = workspace.mass_weighted_field;

        Self::relax(
            backend,
            DryBoundaryRelaxationTarget::WestEastMomentum,
            west_east_momentum,
            full_field_inputs(
                state.west_east_momentum,
                boundaries.west_east_momentum,
                relaxation_coefficients,
            ),
            parameters,
            west_east_periodicity,
            region.west_east_momentum(),
        )?;
        Self::relax(
            backend,
            DryBoundaryRelaxationTarget::SouthNorthMomentum,
            south_north_momentum,
            full_field_inputs(
                state.south_north_momentum,
                boundaries.south_north_momentum,
                relaxation_coefficients,
            ),
            parameters,
            west_east_periodicity,
            region.south_north_momentum(),
        )?;

        let full_level_end = region.workspace_ranges().2.end;
        if has_relaxation_updates(
            parameters,
            west_east_periodicity,
            region.perturbation_geopotential(),
        ) {
            DryBoundaryMassWeightingCpuKernel::execute(
                backend,
                scratch,
                DryBoundaryMassWeightingInputs {
                    field: state.perturbation_geopotential,
                    full_column_mass: state.full_column_mass,
                    multiplier: mass_coefficients.full_level_multiplier,
                    offset: mass_coefficients.full_level_offset,
                    bottom_top_end: full_level_end,
                    region,
                },
            )?;
            Self::relax(
                backend,
                DryBoundaryRelaxationTarget::PerturbationGeopotential,
                perturbation_geopotential,
                workspace_inputs(
                    scratch,
                    boundaries.perturbation_geopotential,
                    relaxation_coefficients,
                    region,
                ),
                parameters,
                west_east_periodicity,
                region.perturbation_geopotential(),
            )?;
        }

        let half_level_end = region.potential_temperature().mass_domains().2.end;
        if has_relaxation_updates(
            parameters,
            west_east_periodicity,
            region.potential_temperature(),
        ) {
            DryBoundaryMassWeightingCpuKernel::execute(
                backend,
                scratch,
                DryBoundaryMassWeightingInputs {
                    field: state.potential_temperature,
                    full_column_mass: state.full_column_mass,
                    multiplier: mass_coefficients.half_level_multiplier,
                    offset: mass_coefficients.half_level_offset,
                    bottom_top_end: half_level_end,
                    region,
                },
            )?;
            Self::relax(
                backend,
                DryBoundaryRelaxationTarget::PotentialTemperature,
                potential_temperature,
                workspace_inputs(
                    scratch,
                    boundaries.potential_temperature,
                    relaxation_coefficients,
                    region,
                ),
                parameters,
                west_east_periodicity,
                region.potential_temperature(),
            )?;
        }

        Self::relax(
            backend,
            DryBoundaryRelaxationTarget::PerturbationColumnMass,
            perturbation_column_mass,
            full_field_inputs(
                state.perturbation_column_mass,
                boundaries.perturbation_column_mass,
                relaxation_coefficients,
            ),
            parameters,
            west_east_periodicity,
            region.perturbation_column_mass(),
        )?;

        if let DryBoundaryVerticalRelaxation::Nested {
            velocity,
            tendency,
            boundary,
        } = vertical
        {
            if !has_relaxation_updates(
                parameters,
                west_east_periodicity,
                region.vertical_momentum(),
            ) {
                return Ok(());
            }
            DryBoundaryMassWeightingCpuKernel::execute(
                backend,
                scratch,
                DryBoundaryMassWeightingInputs {
                    field: velocity,
                    full_column_mass: state.full_column_mass,
                    multiplier: mass_coefficients.full_level_multiplier,
                    offset: mass_coefficients.full_level_offset,
                    bottom_top_end: full_level_end,
                    region,
                },
            )?;
            Self::relax(
                backend,
                DryBoundaryRelaxationTarget::VerticalMomentum,
                tendency,
                workspace_inputs(scratch, boundary, relaxation_coefficients, region),
                parameters,
                west_east_periodicity,
                region.vertical_momentum(),
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn relax(
        backend: &CpuBackend,
        target: DryBoundaryRelaxationTarget,
        tendency: &mut CpuField<f32>,
        inputs: SpecifiedBoundaryRelaxationInputs<'_, CpuField<f32>>,
        parameters: SpecifiedBoundaryRelaxationParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> DryBoundaryRelaxationResult<()> {
        backend
            .add_specified_boundary_relaxation_tendencies(
                tendency,
                inputs,
                parameters,
                west_east_periodicity,
                region,
            )
            .map_err(|source| DryBoundaryRelaxationError::SpecifiedRelaxation { target, source })
    }
}
