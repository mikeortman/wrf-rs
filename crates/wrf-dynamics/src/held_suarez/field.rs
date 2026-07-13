use std::fmt;

/// A field participating in Held-Suarez momentum damping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeldSuarezDampingField {
    /// West-east momentum tendency updated in place.
    WestEastMomentumTendency,
    /// South-north momentum tendency updated in place.
    SouthNorthMomentumTendency,
    /// West-east momentum read by the damping operator.
    WestEastMomentum,
    /// South-north momentum read by the damping operator.
    SouthNorthMomentum,
    /// Perturbation pressure.
    PerturbationPressure,
    /// Base-state pressure.
    BasePressure,
}

impl fmt::Display for HeldSuarezDampingField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEastMomentumTendency => formatter.write_str("west-east momentum tendency"),
            Self::SouthNorthMomentumTendency => {
                formatter.write_str("south-north momentum tendency")
            }
            Self::WestEastMomentum => formatter.write_str("west-east momentum"),
            Self::SouthNorthMomentum => formatter.write_str("south-north momentum"),
            Self::PerturbationPressure => formatter.write_str("perturbation pressure"),
            Self::BasePressure => formatter.write_str("base pressure"),
        }
    }
}
