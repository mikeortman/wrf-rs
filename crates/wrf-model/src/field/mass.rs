use std::fmt;

/// Registry-backed three-dimensional mass-grid state fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum ArwMassField {
    /// Previous west-east velocity (`u_1`).
    PreviousWestEastVelocity,
    /// Current west-east velocity (`u_2`).
    CurrentWestEastVelocity,
    /// Previous south-north velocity (`v_1`).
    PreviousSouthNorthVelocity,
    /// Current south-north velocity (`v_2`).
    CurrentSouthNorthVelocity,
    /// Previous vertical velocity (`w_1`).
    PreviousVerticalVelocity,
    /// Current vertical velocity (`w_2`).
    CurrentVerticalVelocity,
    /// Previous perturbation potential temperature (`t_1`).
    PreviousPotentialTemperature,
    /// Current perturbation potential temperature (`t_2`).
    CurrentPotentialTemperature,
    /// Perturbation pressure (`p`).
    PerturbationPressure,
    /// Perturbation inverse density (`al`).
    PerturbationInverseDensity,
    /// Base-state pressure (`pb`).
    BasePressure,
    /// Base-state inverse density (`alb`).
    BaseInverseDensity,
}

impl ArwMassField {
    pub(crate) const COUNT: usize = 12;
    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::PreviousWestEastVelocity,
        Self::CurrentWestEastVelocity,
        Self::PreviousSouthNorthVelocity,
        Self::CurrentSouthNorthVelocity,
        Self::PreviousVerticalVelocity,
        Self::CurrentVerticalVelocity,
        Self::PreviousPotentialTemperature,
        Self::CurrentPotentialTemperature,
        Self::PerturbationPressure,
        Self::PerturbationInverseDensity,
        Self::BasePressure,
        Self::BaseInverseDensity,
    ];

    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::PreviousWestEastVelocity | Self::CurrentWestEastVelocity => "u",
            Self::PreviousSouthNorthVelocity | Self::CurrentSouthNorthVelocity => "v",
            Self::PreviousVerticalVelocity | Self::CurrentVerticalVelocity => "w",
            Self::PreviousPotentialTemperature | Self::CurrentPotentialTemperature => "t",
            Self::PerturbationPressure => "p",
            Self::PerturbationInverseDensity => "al",
            Self::BasePressure => "pb",
            Self::BaseInverseDensity => "alb",
        }
    }

    pub(crate) const fn time_level(self) -> u8 {
        match self {
            Self::PreviousWestEastVelocity
            | Self::PreviousSouthNorthVelocity
            | Self::PreviousVerticalVelocity
            | Self::PreviousPotentialTemperature => 1,
            Self::CurrentWestEastVelocity
            | Self::CurrentSouthNorthVelocity
            | Self::CurrentVerticalVelocity
            | Self::CurrentPotentialTemperature => 2,
            Self::PerturbationPressure
            | Self::PerturbationInverseDensity
            | Self::BasePressure
            | Self::BaseInverseDensity => 1,
        }
    }
}

impl fmt::Display for ArwMassField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}_{level}",
            self.registry_name(),
            level = self.time_level()
        )
    }
}
