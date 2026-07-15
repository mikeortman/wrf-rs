/// Three-dimensional diagnostics, tendencies, and adapters owned by the runner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ArwWorkspaceVolumeField {
    /// Common-shape adapter for `ph_1`.
    PreviousPerturbationGeopotential,
    /// Common-shape adapter for `ph_2`.
    CurrentPerturbationGeopotential,
    /// Common-shape adapter for `phb`.
    BaseGeopotential,
    /// Coupled west-east momentum `ru`.
    CoupledWestEastMomentum,
    /// Coupled south-north momentum `rv`.
    CoupledSouthNorthMomentum,
    /// Coupled vertical momentum `rw`.
    CoupledVerticalMomentum,
    /// West-east moisture coefficient `cqu`.
    WestEastMoistureCoefficient,
    /// South-north moisture coefficient `cqv`.
    SouthNorthMoistureCoefficient,
    /// Vertical moisture coefficient `cqw`.
    VerticalMoistureCoefficient,
    /// Full inverse density `alt`.
    FullInverseDensity,
    /// Saved west-east velocity `u_save`.
    SavedWestEastVelocity,
    /// Saved south-north velocity `v_save`.
    SavedSouthNorthVelocity,
    /// Saved vertical velocity `w_save`.
    SavedVerticalVelocity,
    /// Saved potential temperature `t_save`.
    SavedPotentialTemperature,
    /// Saved perturbation geopotential `ph_save`.
    SavedPerturbationGeopotential,
    /// Saved vertical mass flux `ww1`.
    SavedVerticalMassFlux,
    /// Saved pressure coefficient `c2a`.
    SavedPressureCoefficient,
    /// Previous pressure perturbation `pm1`.
    PreviousPressurePerturbation,
    /// Lower implicit diagonal `a`.
    LowerDiagonal,
    /// Inverse eliminated diagonal `alpha`.
    InverseEliminatedDiagonal,
    /// Upper elimination factor `gamma`.
    UpperEliminationFactor,
    /// Time-averaged thermodynamics `t2save`.
    TimeAveragedThermodynamics,
    /// Accumulated west-east mass flux `ru_m`.
    AverageWestEastMassFlux,
    /// Accumulated south-north mass flux `rv_m`.
    AverageSouthNorthMassFlux,
    /// Runge-Kutta west-east momentum tendency.
    WestEastMomentumTendency,
    /// Runge-Kutta south-north momentum tendency.
    SouthNorthMomentumTendency,
    /// Runge-Kutta vertical momentum tendency.
    VerticalMomentumTendency,
    /// Runge-Kutta geopotential tendency.
    GeopotentialTendency,
    /// Runge-Kutta potential-temperature tendency.
    PotentialTemperatureTendency,
    /// Persistent west-east forward tendency.
    ForwardWestEastMomentumTendency,
    /// Persistent south-north forward tendency.
    ForwardSouthNorthMomentumTendency,
    /// Persistent vertical forward tendency.
    ForwardVerticalMomentumTendency,
    /// Persistent geopotential forward tendency.
    ForwardGeopotentialTendency,
    /// Persistent potential-temperature forward tendency.
    ForwardPotentialTemperatureTendency,
    /// Reusable geopotential right-hand-side scratch.
    GeopotentialRightHandSide,
}

impl ArwWorkspaceVolumeField {
    pub(crate) const COUNT: usize = 35;

    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::PreviousPerturbationGeopotential,
        Self::CurrentPerturbationGeopotential,
        Self::BaseGeopotential,
        Self::CoupledWestEastMomentum,
        Self::CoupledSouthNorthMomentum,
        Self::CoupledVerticalMomentum,
        Self::WestEastMoistureCoefficient,
        Self::SouthNorthMoistureCoefficient,
        Self::VerticalMoistureCoefficient,
        Self::FullInverseDensity,
        Self::SavedWestEastVelocity,
        Self::SavedSouthNorthVelocity,
        Self::SavedVerticalVelocity,
        Self::SavedPotentialTemperature,
        Self::SavedPerturbationGeopotential,
        Self::SavedVerticalMassFlux,
        Self::SavedPressureCoefficient,
        Self::PreviousPressurePerturbation,
        Self::LowerDiagonal,
        Self::InverseEliminatedDiagonal,
        Self::UpperEliminationFactor,
        Self::TimeAveragedThermodynamics,
        Self::AverageWestEastMassFlux,
        Self::AverageSouthNorthMassFlux,
        Self::WestEastMomentumTendency,
        Self::SouthNorthMomentumTendency,
        Self::VerticalMomentumTendency,
        Self::GeopotentialTendency,
        Self::PotentialTemperatureTendency,
        Self::ForwardWestEastMomentumTendency,
        Self::ForwardSouthNorthMomentumTendency,
        Self::ForwardVerticalMomentumTendency,
        Self::ForwardGeopotentialTendency,
        Self::ForwardPotentialTemperatureTendency,
        Self::GeopotentialRightHandSide,
    ];
}
