/// Horizontal diagnostics and tendencies owned by the runner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ArwWorkspaceColumnField {
    /// Full column mass `mut`.
    FullColumnMass,
    /// West-east staggered column mass `muu`.
    WestEastColumnMass,
    /// South-north staggered column mass `muv`.
    SouthNorthColumnMass,
    /// Runge-Kutta column-mass tendency `mu_tend`.
    ColumnMassTendency,
    /// Persistent column-mass tendency `mu_tendf`.
    ForwardColumnMassTendency,
    /// Final west-east staggered mass `muus`.
    FinalWestEastColumnMass,
    /// Final south-north staggered mass `muvs`.
    FinalSouthNorthColumnMass,
    /// Final full column mass `muts`.
    FinalFullColumnMass,
    /// Divergence-damping mass `mudf`.
    DivergenceDampingColumnMass,
    /// Time-centered mass `muave`.
    TimeCenteredColumnMass,
    /// Saved perturbation mass `mu_save`.
    SavedPerturbationColumnMass,
}

impl ArwWorkspaceColumnField {
    pub(crate) const COUNT: usize = 11;

    /// All typed fields in storage order.
    pub const ALL: [Self; Self::COUNT] = [
        Self::FullColumnMass,
        Self::WestEastColumnMass,
        Self::SouthNorthColumnMass,
        Self::ColumnMassTendency,
        Self::ForwardColumnMassTendency,
        Self::FinalWestEastColumnMass,
        Self::FinalSouthNorthColumnMass,
        Self::FinalFullColumnMass,
        Self::DivergenceDampingColumnMass,
        Self::TimeCenteredColumnMass,
        Self::SavedPerturbationColumnMass,
    ];
}
