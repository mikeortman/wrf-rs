use std::fmt;

/// Registry-backed W-level geopotential fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum ArwGeopotentialField {
    /// Previous perturbation geopotential (`ph_1`).
    PreviousPerturbation,
    /// Current perturbation geopotential (`ph_2`).
    CurrentPerturbation,
    /// Base-state geopotential (`phb`).
    BaseState,
}

impl ArwGeopotentialField {
    pub(crate) const COUNT: usize = 3;
    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::PreviousPerturbation,
        Self::CurrentPerturbation,
        Self::BaseState,
    ];

    pub(crate) const fn registry_name(self) -> &'static str {
        match self {
            Self::PreviousPerturbation | Self::CurrentPerturbation => "ph",
            Self::BaseState => "phb",
        }
    }

    pub(crate) const fn time_level(self) -> u8 {
        match self {
            Self::PreviousPerturbation => 1,
            Self::CurrentPerturbation => 2,
            Self::BaseState => 1,
        }
    }
}

impl fmt::Display for ArwGeopotentialField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}_{level}",
            self.registry_name(),
            level = self.time_level()
        )
    }
}
