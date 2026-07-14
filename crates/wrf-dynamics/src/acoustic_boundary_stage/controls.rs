use crate::{
    AcousticHorizontalBoundaryPolicy, AcousticMassThetaBoundaryPolicy,
    AcousticMassThetaLateralDomain, AcousticMassThetaWestEastPeriodicity, AcousticRelaxationZone,
    AcousticSouthNorthBoundary, AcousticTrajectoryControls, AcousticVerticalBoundaryPolicy,
    AcousticVerticalLateralDomain, AcousticVerticalWestEastPeriodicity, AcousticWestEastBoundary,
    AcousticWestEastPeriodicity, PhysicalBoundaryConditions,
};

/// Scalar and lateral-boundary controls for the complete acoustic stage.
#[derive(Clone, Copy, Debug)]
pub struct AcousticBoundaryStageControls {
    pub(crate) trajectory: AcousticTrajectoryControls,
    pub(crate) physical_boundaries: PhysicalBoundaryConditions,
    pub(crate) specified_zone_width: usize,
}

impl AcousticBoundaryStageControls {
    /// Groups the verified trajectory controls with WRF lateral configuration.
    ///
    /// A zero specified-zone width preserves the underlying routines' exact
    /// no-op behavior. Polar configurations are rejected by the stage preflight
    /// because `pxft` is deliberately outside this slice.
    pub const fn new(
        trajectory: AcousticTrajectoryControls,
        physical_boundaries: PhysicalBoundaryConditions,
        specified_zone_width: usize,
    ) -> Self {
        Self {
            trajectory,
            physical_boundaries,
            specified_zone_width,
        }
    }

    pub(crate) const fn has_specified_updates(self) -> bool {
        self.physical_boundaries.specified || self.physical_boundaries.nested
    }

    pub(crate) fn trajectory_controls(self) -> AcousticTrajectoryControls {
        let mut trajectory = self.trajectory;
        let lateral_domain = if self.has_specified_updates() {
            AcousticMassThetaLateralDomain::SpecifiedOrNested
        } else {
            AcousticMassThetaLateralDomain::Global
        };
        let vertical_lateral_domain = if self.has_specified_updates() {
            AcousticVerticalLateralDomain::SpecifiedOrNested
        } else {
            AcousticVerticalLateralDomain::Global
        };
        let relaxation_zone = if self.has_specified_updates() {
            AcousticRelaxationZone::Active {
                width: self.specified_zone_width,
            }
        } else {
            AcousticRelaxationZone::Disabled
        };
        let horizontal_periodicity = if self.physical_boundaries.periodic_x {
            AcousticWestEastPeriodicity::Periodic
        } else {
            AcousticWestEastPeriodicity::Nonperiodic
        };
        let mass_periodicity = if self.physical_boundaries.periodic_x {
            AcousticMassThetaWestEastPeriodicity::Periodic
        } else {
            AcousticMassThetaWestEastPeriodicity::Nonperiodic
        };
        let vertical_periodicity = if self.physical_boundaries.periodic_x {
            AcousticVerticalWestEastPeriodicity::Periodic
        } else {
            AcousticVerticalWestEastPeriodicity::Nonperiodic
        };
        trajectory.horizontal_boundary_policy = AcousticHorizontalBoundaryPolicy::new(
            relaxation_zone,
            horizontal_periodicity,
            west_east_boundary(
                self.physical_boundaries.open_xs,
                self.physical_boundaries.symmetric_xs,
            ),
            west_east_boundary(
                self.physical_boundaries.open_xe,
                self.physical_boundaries.symmetric_xe,
            ),
            south_north_boundary(
                self.physical_boundaries.open_ys,
                self.physical_boundaries.symmetric_ys,
            ),
            south_north_boundary(
                self.physical_boundaries.open_ye,
                self.physical_boundaries.symmetric_ye,
            ),
        );
        trajectory.mass_theta_boundary_policy =
            AcousticMassThetaBoundaryPolicy::new(lateral_domain, mass_periodicity);
        trajectory.vertical_boundary_policy =
            AcousticVerticalBoundaryPolicy::new(vertical_lateral_domain, vertical_periodicity);
        trajectory
    }
}

const fn west_east_boundary(is_open: bool, is_symmetric: bool) -> AcousticWestEastBoundary {
    if is_symmetric {
        AcousticWestEastBoundary::Symmetric
    } else if is_open {
        AcousticWestEastBoundary::Open
    } else {
        AcousticWestEastBoundary::Closed
    }
}

const fn south_north_boundary(is_open: bool, is_symmetric: bool) -> AcousticSouthNorthBoundary {
    if is_symmetric {
        AcousticSouthNorthBoundary::Symmetric
    } else if is_open {
        AcousticSouthNorthBoundary::Open
    } else {
        AcousticSouthNorthBoundary::Closed
    }
}
