use std::fmt;

/// Registry-backed horizontal state fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum ArwColumnField {
    /// Previous perturbation dry column mass (`mu_1`).
    PreviousPerturbationMass,
    /// Current perturbation dry column mass (`mu_2`).
    CurrentPerturbationMass,
    /// Base-state dry column mass (`mub`).
    BaseMass,
    /// Accumulated nonconvective precipitation (`RAINNC`).
    AccumulatedPrecipitation,
    /// Current-step nonconvective precipitation (`RAINNCV`).
    StepPrecipitation,
}

impl ArwColumnField {
    pub(crate) const COUNT: usize = 5;
    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::PreviousPerturbationMass,
        Self::CurrentPerturbationMass,
        Self::BaseMass,
        Self::AccumulatedPrecipitation,
        Self::StepPrecipitation,
    ];

    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::PreviousPerturbationMass | Self::CurrentPerturbationMass => "mu",
            Self::BaseMass => "mub",
            Self::AccumulatedPrecipitation => "rainnc",
            Self::StepPrecipitation => "rainncv",
        }
    }

    pub(crate) const fn time_level(self) -> u8 {
        match self {
            Self::PreviousPerturbationMass => 1,
            Self::CurrentPerturbationMass => 2,
            Self::BaseMass | Self::AccumulatedPrecipitation | Self::StepPrecipitation => 1,
        }
    }
}

impl fmt::Display for ArwColumnField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}_{level}",
            self.registry_name(),
            level = self.time_level()
        )
    }
}
