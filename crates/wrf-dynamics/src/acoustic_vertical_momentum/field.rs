use std::fmt;

/// Scientific role of a field in acoustic vertical advancement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalField {
    /// Coupled vertical momentum (`w`).
    VerticalMomentum,
    /// Perturbation geopotential (`ph`).
    PerturbationGeopotential,
    /// Normalized time-averaged thermodynamic term (`t_2ave`).
    TimeAveragedThermodynamics,
    /// Caller-owned implicit geopotential workspace (`rhs`).
    GeopotentialRightHandSide,
    /// Large-step vertical momentum tendency (`rw_tend`).
    VerticalMomentumTendency,
    /// Contravariant vertical mass flux (`ww`).
    VerticalMassFlux,
    /// Saved vertical momentum used by upper damping (`w_save`).
    SavedVerticalMomentum,
    /// Coupled west-east momentum (`u`).
    WestEastMomentum,
    /// Coupled south-north momentum (`v`).
    SouthNorthMomentum,
    /// Full column mass (`mut`).
    FullColumnMass,
    /// Time-centered perturbation column mass (`muave`).
    TimeCenteredColumnMass,
    /// Coupled full column mass (`muts`).
    CoupledColumnMass,
    /// Current potential temperature (`t_2`).
    PotentialTemperature,
    /// Saved potential temperature passed as source `t_1`.
    SavedPotentialTemperature,
    /// Saved perturbation geopotential (`ph_1`).
    SavedPerturbationGeopotential,
    /// Base-state geopotential (`phb`).
    BaseGeopotential,
    /// Large-step geopotential tendency (`ph_tend`).
    PerturbationGeopotentialTendency,
    /// Terrain height (`ht`).
    TerrainHeight,
    /// Mass-point west-east map factor (`msftx`).
    MassPointWestEastMapFactor,
    /// Mass-point south-north map factor (`msfty`).
    MassPointSouthNorthMapFactor,
    /// Pressure-gradient coefficient (`c2a`).
    PressureCoefficient,
    /// Moisture coefficient (`cqw`).
    MoistureCoefficient,
    /// Inverse density (`alt`).
    InverseDensity,
    /// Tridiagonal lower diagonal (`a`).
    LowerDiagonal,
    /// Reciprocal eliminated diagonal (`alpha`).
    InverseEliminatedDiagonal,
    /// Upper elimination factor (`gamma`).
    UpperEliminationFactor,
}

impl fmt::Display for AcousticVerticalField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
