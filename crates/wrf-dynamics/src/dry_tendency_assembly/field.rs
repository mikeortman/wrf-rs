use std::fmt;

/// Semantic role of a field supplied to dry-tendency assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryTendencyAssemblyField {
    /// West-east RK momentum tendency.
    WestEastRungeKuttaMomentum,
    /// South-north RK momentum tendency.
    SouthNorthRungeKuttaMomentum,
    /// Vertical RK momentum tendency.
    VerticalRungeKuttaMomentum,
    /// RK geopotential tendency.
    RungeKuttaGeopotential,
    /// RK potential-temperature tendency.
    RungeKuttaPotentialTemperature,
    /// RK column-mass tendency.
    RungeKuttaColumnMass,
    /// Persistent west-east momentum tendency.
    WestEastForwardMomentum,
    /// Persistent south-north momentum tendency.
    SouthNorthForwardMomentum,
    /// Persistent vertical momentum tendency.
    VerticalForwardMomentum,
    /// Persistent geopotential tendency.
    ForwardGeopotential,
    /// Persistent potential-temperature tendency.
    ForwardPotentialTemperature,
    /// Persistent column-mass tendency.
    ForwardColumnMass,
    /// Saved west-east boundary tendency.
    SavedWestEastMomentum,
    /// Saved south-north boundary tendency.
    SavedSouthNorthMomentum,
    /// Saved vertical boundary tendency.
    SavedVerticalMomentum,
    /// Saved geopotential boundary tendency.
    SavedGeopotential,
    /// Saved potential-temperature boundary tendency.
    SavedPotentialTemperature,
    /// Diabatic heating tendency.
    DiabaticHeating,
    /// Full column mass.
    FullColumnMass,
    /// West-east momentum south-north map factor.
    WestEastMomentumSouthNorthMapFactor,
    /// South-north momentum west-east map factor.
    SouthNorthMomentumWestEastMapFactor,
    /// Inverse south-north momentum west-east map factor.
    InverseSouthNorthMomentumWestEastMapFactor,
    /// Mass-point south-north map factor.
    MassPointSouthNorthMapFactor,
}

impl fmt::Display for DryTendencyAssemblyField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WestEastRungeKuttaMomentum => "west-east RK momentum tendency",
            Self::SouthNorthRungeKuttaMomentum => "south-north RK momentum tendency",
            Self::VerticalRungeKuttaMomentum => "vertical RK momentum tendency",
            Self::RungeKuttaGeopotential => "RK geopotential tendency",
            Self::RungeKuttaPotentialTemperature => "RK potential-temperature tendency",
            Self::RungeKuttaColumnMass => "RK column-mass tendency",
            Self::WestEastForwardMomentum => "persistent west-east momentum tendency",
            Self::SouthNorthForwardMomentum => "persistent south-north momentum tendency",
            Self::VerticalForwardMomentum => "persistent vertical momentum tendency",
            Self::ForwardGeopotential => "persistent geopotential tendency",
            Self::ForwardPotentialTemperature => "persistent potential-temperature tendency",
            Self::ForwardColumnMass => "persistent column-mass tendency",
            Self::SavedWestEastMomentum => "saved west-east momentum tendency",
            Self::SavedSouthNorthMomentum => "saved south-north momentum tendency",
            Self::SavedVerticalMomentum => "saved vertical momentum tendency",
            Self::SavedGeopotential => "saved geopotential tendency",
            Self::SavedPotentialTemperature => "saved potential-temperature tendency",
            Self::DiabaticHeating => "diabatic heating",
            Self::FullColumnMass => "full column mass",
            Self::WestEastMomentumSouthNorthMapFactor => "west-east momentum map factor",
            Self::SouthNorthMomentumWestEastMapFactor => "south-north momentum map factor",
            Self::InverseSouthNorthMomentumWestEastMapFactor => {
                "inverse south-north momentum map factor"
            }
            Self::MassPointSouthNorthMapFactor => "mass-point map factor",
        })
    }
}
