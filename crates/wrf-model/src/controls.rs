use wrf_dynamics::{
    AcousticHorizontalBoundaryPolicy, AcousticMassThetaBoundaryPolicy,
    AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity, AcousticPressureMode,
    AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticStepFinalizationControls,
    AcousticStepFinalizationPhase, AcousticStepPreparationPhase, AcousticTrajectoryControls,
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalLateralDomain,
    AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary, AcousticWestEastPeriodicity,
    VerticalAcousticTopBoundary,
};
use wrf_physics::ArwMicrophysicsControls;

use crate::{ArwModelError, ArwModelResult};

/// Validated controls for the accepted dynamics-to-Kessler projection.
#[derive(Clone, Copy, Debug)]
pub struct ArwModelControls {
    pub(crate) acoustic: AcousticTrajectoryControls,
    pub(crate) finalization: AcousticStepFinalizationControls,
    pub(crate) microphysics: ArwMicrophysicsControls,
}

impl ArwModelControls {
    /// Groups independently validated component controls without changing them.
    pub const fn new(
        acoustic: AcousticTrajectoryControls,
        finalization: AcousticStepFinalizationControls,
        microphysics: ArwMicrophysicsControls,
    ) -> Self {
        Self {
            acoustic,
            finalization,
            microphysics,
        }
    }

    pub(crate) fn validate_accepted_projection(self) -> ArwModelResult<()> {
        if self.acoustic.preparation_phase() != AcousticStepPreparationPhase::FirstSubstep {
            return Err(incompatible("first Runge-Kutta substep"));
        }
        if self.finalization.phase() != AcousticStepFinalizationPhase::Intermediate {
            return Err(incompatible("intermediate Runge-Kutta finalization"));
        }
        if self.acoustic.substep_count() != self.finalization.acoustic_substep_count()
            || self.acoustic.acoustic_time_step().to_bits()
                != self.finalization.acoustic_time_step().to_bits()
        {
            return Err(incompatible("acoustic substep count and timestep"));
        }
        let horizontal = AcousticHorizontalBoundaryPolicy::new(
            AcousticRelaxationZone::Disabled,
            AcousticWestEastPeriodicity::Nonperiodic,
            AcousticWestEastBoundary::Closed,
            AcousticWestEastBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
            AcousticSouthNorthBoundary::Closed,
        );
        let mass_theta = AcousticMassThetaBoundaryPolicy::new(
            AcousticMassThetaLateralDomain::Global,
            AcousticMassThetaWestEastPeriodicity::Nonperiodic,
        );
        let vertical = AcousticVerticalBoundaryPolicy::new(
            AcousticVerticalLateralDomain::Global,
            AcousticVerticalWestEastPeriodicity::Nonperiodic,
        );
        if self.acoustic.horizontal_boundary_policy() != horizontal
            || self.acoustic.mass_theta_boundary_policy() != mass_theta
            || self.acoustic.vertical_boundary_policy() != vertical
            || self.acoustic.pressure_mode() != AcousticPressureMode::Nonhydrostatic
            || self.acoustic.top_boundary() != VerticalAcousticTopBoundary::Nonrigid
            || self.acoustic.vertical_advection()
                != AcousticVerticalAdvection::StaggeredGeopotentialGradient
            || !self.acoustic.is_vertical_damping_disabled()
        {
            return Err(incompatible("nonperiodic closed local acoustic stage"));
        }
        Ok(())
    }

    pub(crate) const fn inverse_west_east_grid_spacing(self) -> f32 {
        self.acoustic.inverse_west_east_grid_spacing()
    }

    pub(crate) const fn inverse_south_north_grid_spacing(self) -> f32 {
        self.acoustic.inverse_south_north_grid_spacing()
    }
}

const fn incompatible(component: &'static str) -> ArwModelError {
    ArwModelError::IncompatibleControls { component }
}
