use wrf_compute::{CpuBackend, CpuField};

use super::super::{
    DryBoundaryTendencies, DryBoundaryTendencyBoundaryFields, DryBoundaryTendencyError,
    DryBoundaryTendencyRegion, DryBoundaryTendencyResult, DryBoundaryTendencyTarget,
    DryBoundaryVerticalTendency,
};
use super::validation::validate_cpu_dry_boundary_tendency_assignment;
use crate::{
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyKernels,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryWestEastPeriodicity,
};

pub(super) struct DryBoundaryTendencyCpuExecution<'a, 'region> {
    backend: &'a CpuBackend,
    tendencies: DryBoundaryTendencies<'a, CpuField<f32>>,
    boundaries: DryBoundaryTendencyBoundaryFields<'a, CpuField<f32>>,
    vertical: DryBoundaryVerticalTendency<'a, CpuField<f32>>,
    parameters: SpecifiedBoundaryTendencyParameters,
    west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
    region: &'region DryBoundaryTendencyRegion,
}

impl<'a, 'region> DryBoundaryTendencyCpuExecution<'a, 'region> {
    pub(super) fn new(
        backend: &'a CpuBackend,
        tendencies: DryBoundaryTendencies<'a, CpuField<f32>>,
        boundaries: DryBoundaryTendencyBoundaryFields<'a, CpuField<f32>>,
        vertical: DryBoundaryVerticalTendency<'a, CpuField<f32>>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &'region DryBoundaryTendencyRegion,
    ) -> Self {
        Self {
            backend,
            tendencies,
            boundaries,
            vertical,
            parameters,
            west_east_periodicity,
            region,
        }
    }

    pub(super) fn run(self) -> DryBoundaryTendencyResult<()> {
        validate_cpu_dry_boundary_tendency_assignment(
            &self.tendencies,
            self.boundaries,
            &self.vertical,
            self.parameters,
            self.region,
        )?;

        let Self {
            backend,
            tendencies,
            boundaries,
            vertical,
            parameters,
            west_east_periodicity,
            region,
        } = self;
        let DryBoundaryTendencies {
            west_east_momentum,
            south_north_momentum,
            perturbation_geopotential,
            potential_temperature,
            perturbation_column_mass,
        } = tendencies;

        Self::assign(
            backend,
            DryBoundaryTendencyTarget::WestEastMomentum,
            west_east_momentum,
            boundaries.west_east_momentum,
            parameters,
            west_east_periodicity,
            region.west_east_momentum(),
        )?;
        Self::assign(
            backend,
            DryBoundaryTendencyTarget::SouthNorthMomentum,
            south_north_momentum,
            boundaries.south_north_momentum,
            parameters,
            west_east_periodicity,
            region.south_north_momentum(),
        )?;
        Self::assign(
            backend,
            DryBoundaryTendencyTarget::PerturbationGeopotential,
            perturbation_geopotential,
            boundaries.perturbation_geopotential,
            parameters,
            west_east_periodicity,
            region.perturbation_geopotential(),
        )?;
        Self::assign(
            backend,
            DryBoundaryTendencyTarget::PotentialTemperature,
            potential_temperature,
            boundaries.potential_temperature,
            parameters,
            west_east_periodicity,
            region.potential_temperature(),
        )?;
        Self::assign(
            backend,
            DryBoundaryTendencyTarget::PerturbationColumnMass,
            perturbation_column_mass,
            boundaries.perturbation_column_mass,
            parameters,
            west_east_periodicity,
            region.perturbation_column_mass(),
        )?;
        if let DryBoundaryVerticalTendency::Nested {
            tendency,
            boundaries,
        } = vertical
        {
            Self::assign(
                backend,
                DryBoundaryTendencyTarget::VerticalMomentum,
                tendency,
                boundaries,
                parameters,
                west_east_periodicity,
                region.vertical_momentum(),
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        backend: &CpuBackend,
        target: DryBoundaryTendencyTarget,
        tendency: &mut CpuField<f32>,
        boundaries: SpecifiedBoundaryTendencies<'_, CpuField<f32>>,
        parameters: SpecifiedBoundaryTendencyParameters,
        west_east_periodicity: SpecifiedBoundaryWestEastPeriodicity,
        region: &SpecifiedBoundaryUpdateRegion,
    ) -> DryBoundaryTendencyResult<()> {
        backend
            .assign_specified_boundary_tendencies(
                tendency,
                boundaries,
                parameters,
                west_east_periodicity,
                region,
            )
            .map_err(|source| DryBoundaryTendencyError::SpecifiedTendency { target, source })
    }
}
