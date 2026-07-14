use std::fmt;

/// Scientific role of a field in acoustic mass-and-theta advancement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticMassThetaField {
    /// Acoustic vertical mass flux (`ww`).
    VerticalMassFlux,
    /// Perturbation column mass (`mu`).
    ColumnMass,
    /// Coupled perturbation potential temperature (`t`).
    PotentialTemperature,
    /// Saved large-step vertical mass flux (`ww_1`).
    SavedVerticalMassFlux,
    /// Current U momentum (`u`).
    WestEastMomentum,
    /// Saved U momentum (`u_1`).
    SavedWestEastMomentum,
    /// Current V momentum (`v`).
    SouthNorthMomentum,
    /// Saved V momentum (`v_1`).
    SavedSouthNorthMomentum,
    /// Saved perturbation potential temperature (`t_1`).
    SavedPotentialTemperature,
    /// Potential-temperature tendency (`ft`).
    PotentialTemperatureTendency,
    /// Base full column mass (`mut`).
    BaseFullColumnMass,
    /// U-staggered full column mass (`muu`).
    WestEastColumnMass,
    /// V-staggered full column mass (`muv`).
    SouthNorthColumnMass,
    /// Column-mass tendency (`mu_tend`).
    ColumnMassTendency,
    /// Time-centered column mass (`muave`).
    TimeCenteredColumnMass,
    /// Coupled full column mass (`muts`).
    CoupledColumnMass,
    /// Saved divergence-damping tendency (`mudf`).
    DivergenceDampingMassTendency,
    /// Previous potential temperature (`t_ave`).
    PreviousPotentialTemperature,
    /// U-point Y map factor (`msfuy`).
    WestEastYMapFactor,
    /// Inverse V-point X map factor (`msfvx_inv`).
    InverseSouthNorthXMapFactor,
    /// Mass-point X map factor (`msftx`).
    MassPointXMapFactor,
    /// Mass-point Y map factor (`msfty`).
    MassPointYMapFactor,
}

impl fmt::Display for AcousticMassThetaField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::VerticalMassFlux => "vertical mass flux",
            Self::ColumnMass => "column mass",
            Self::PotentialTemperature => "potential temperature",
            Self::SavedVerticalMassFlux => "saved vertical mass flux",
            Self::WestEastMomentum => "west-east momentum",
            Self::SavedWestEastMomentum => "saved west-east momentum",
            Self::SouthNorthMomentum => "south-north momentum",
            Self::SavedSouthNorthMomentum => "saved south-north momentum",
            Self::SavedPotentialTemperature => "saved potential temperature",
            Self::PotentialTemperatureTendency => "potential-temperature tendency",
            Self::BaseFullColumnMass => "base full column mass",
            Self::WestEastColumnMass => "west-east column mass",
            Self::SouthNorthColumnMass => "south-north column mass",
            Self::ColumnMassTendency => "column-mass tendency",
            Self::TimeCenteredColumnMass => "time-centered column mass",
            Self::CoupledColumnMass => "coupled column mass",
            Self::DivergenceDampingMassTendency => "divergence-damping mass tendency",
            Self::PreviousPotentialTemperature => "previous potential temperature",
            Self::WestEastYMapFactor => "west-east Y map factor",
            Self::InverseSouthNorthXMapFactor => "inverse south-north X map factor",
            Self::MassPointXMapFactor => "mass-point X map factor",
            Self::MassPointYMapFactor => "mass-point Y map factor",
        })
    }
}
