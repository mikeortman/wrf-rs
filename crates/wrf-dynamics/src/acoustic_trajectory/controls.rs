use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticMassThetaBoundaryPolicy, AcousticPressureMode,
    AcousticStepPreparationPhase, AcousticTrajectoryError, AcousticTrajectoryResult,
    AcousticVerticalAdvection, AcousticVerticalBoundaryPolicy, AcousticVerticalDamping,
    VerticalAcousticTopBoundary,
};

/// Shared scalar, phase, and boundary controls for one acoustic trajectory.
#[derive(Clone, Copy, Debug)]
pub struct AcousticTrajectoryControls {
    pub(crate) preparation_phase: AcousticStepPreparationPhase,
    pub(crate) substep_count: usize,
    pub(crate) inverse_west_east_grid_spacing: f32,
    pub(crate) inverse_south_north_grid_spacing: f32,
    pub(crate) acoustic_time_step: f32,
    pub(crate) gravitational_acceleration: f32,
    pub(crate) base_potential_temperature: f32,
    pub(crate) time_centering: f32,
    pub(crate) pressure_divergence_damping: f32,
    pub(crate) horizontal_divergence_damping: f32,
    pub(crate) boundary_velocity_weights: [f32; 3],
    pub(crate) pressure_mode: AcousticPressureMode,
    pub(crate) top_boundary: VerticalAcousticTopBoundary,
    pub(crate) horizontal_boundary_policy: AcousticHorizontalBoundaryPolicy,
    pub(crate) mass_theta_boundary_policy: AcousticMassThetaBoundaryPolicy,
    pub(crate) vertical_boundary_policy: AcousticVerticalBoundaryPolicy,
    pub(crate) vertical_advection: AcousticVerticalAdvection,
    pub(crate) vertical_damping: AcousticVerticalDamping,
}

impl AcousticTrajectoryControls {
    /// Returns the Runge-Kutta phase used by acoustic preparation.
    pub const fn preparation_phase(self) -> AcousticStepPreparationPhase {
        self.preparation_phase
    }

    /// Returns the number of acoustic substeps.
    pub const fn substep_count(self) -> usize {
        self.substep_count
    }

    /// Returns inverse west-east grid spacing.
    pub const fn inverse_west_east_grid_spacing(self) -> f32 {
        self.inverse_west_east_grid_spacing
    }

    /// Returns inverse south-north grid spacing.
    pub const fn inverse_south_north_grid_spacing(self) -> f32 {
        self.inverse_south_north_grid_spacing
    }

    /// Returns the acoustic timestep in seconds.
    pub const fn acoustic_time_step(self) -> f32 {
        self.acoustic_time_step
    }

    /// Returns the horizontal boundary policy.
    pub const fn horizontal_boundary_policy(self) -> AcousticHorizontalBoundaryPolicy {
        self.horizontal_boundary_policy
    }

    /// Returns the mass/theta boundary policy.
    pub const fn mass_theta_boundary_policy(self) -> AcousticMassThetaBoundaryPolicy {
        self.mass_theta_boundary_policy
    }

    /// Returns the vertical boundary policy.
    pub const fn vertical_boundary_policy(self) -> AcousticVerticalBoundaryPolicy {
        self.vertical_boundary_policy
    }

    /// Returns the pressure mode.
    pub const fn pressure_mode(self) -> AcousticPressureMode {
        self.pressure_mode
    }

    /// Returns the upper boundary mode.
    pub const fn top_boundary(self) -> VerticalAcousticTopBoundary {
        self.top_boundary
    }

    /// Returns the vertical advection discretization.
    pub const fn vertical_advection(self) -> AcousticVerticalAdvection {
        self.vertical_advection
    }

    /// Returns whether upper-level damping is disabled.
    pub const fn is_vertical_damping_disabled(self) -> bool {
        matches!(self.vertical_damping, AcousticVerticalDamping::Disabled)
    }

    /// Validates the substep count and preserves all IEEE scalar inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        preparation_phase: AcousticStepPreparationPhase,
        substep_count: usize,
        inverse_west_east_grid_spacing: f32,
        inverse_south_north_grid_spacing: f32,
        acoustic_time_step: f32,
        gravitational_acceleration: f32,
        base_potential_temperature: f32,
        time_centering: f32,
        pressure_divergence_damping: f32,
        horizontal_divergence_damping: f32,
        boundary_velocity_weights: [f32; 3],
        pressure_mode: AcousticPressureMode,
        top_boundary: VerticalAcousticTopBoundary,
        horizontal_boundary_policy: AcousticHorizontalBoundaryPolicy,
        mass_theta_boundary_policy: AcousticMassThetaBoundaryPolicy,
        vertical_boundary_policy: AcousticVerticalBoundaryPolicy,
        vertical_advection: AcousticVerticalAdvection,
        vertical_damping: AcousticVerticalDamping,
    ) -> AcousticTrajectoryResult<Self> {
        if substep_count == 0 {
            return Err(AcousticTrajectoryError::ZeroSubstepCount);
        }
        Ok(Self {
            preparation_phase,
            substep_count,
            inverse_west_east_grid_spacing,
            inverse_south_north_grid_spacing,
            acoustic_time_step,
            gravitational_acceleration,
            base_potential_temperature,
            time_centering,
            pressure_divergence_damping,
            horizontal_divergence_damping,
            boundary_velocity_weights,
            pressure_mode,
            top_boundary,
            horizontal_boundary_policy,
            mass_theta_boundary_policy,
            vertical_boundary_policy,
            vertical_advection,
            vertical_damping,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AcousticHorizontalBoundaryPolicy, AcousticMassThetaBoundaryPolicy,
        AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity,
        AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticVerticalLateralDomain,
        AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary, AcousticWestEastPeriodicity,
    };

    use super::*;

    #[test]
    fn rejects_an_empty_acoustic_sequence() {
        let result = AcousticTrajectoryControls::try_new(
            AcousticStepPreparationPhase::FirstSubstep,
            0,
            1.0,
            1.0,
            0.1,
            9.81,
            300.0,
            0.1,
            0.17,
            0.1,
            [0.5, 0.3, 0.2],
            AcousticPressureMode::Nonhydrostatic,
            VerticalAcousticTopBoundary::Nonrigid,
            AcousticHorizontalBoundaryPolicy::new(
                AcousticRelaxationZone::Disabled,
                AcousticWestEastPeriodicity::Nonperiodic,
                AcousticWestEastBoundary::Open,
                AcousticWestEastBoundary::Open,
                AcousticSouthNorthBoundary::Open,
                AcousticSouthNorthBoundary::Open,
            ),
            AcousticMassThetaBoundaryPolicy::new(
                AcousticMassThetaLateralDomain::Global,
                AcousticMassThetaWestEastPeriodicity::Nonperiodic,
            ),
            AcousticVerticalBoundaryPolicy::new(
                AcousticVerticalLateralDomain::Global,
                AcousticVerticalWestEastPeriodicity::Nonperiodic,
            ),
            AcousticVerticalAdvection::StaggeredGeopotentialGradient,
            AcousticVerticalDamping::Disabled,
        );

        assert_eq!(
            result.unwrap_err(),
            AcousticTrajectoryError::ZeroSubstepCount
        );
    }
}
