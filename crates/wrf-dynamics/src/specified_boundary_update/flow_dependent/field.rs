use std::fmt;

/// Field role in a flow-dependent specified-boundary operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifiedBoundaryFlowField {
    /// Unstaggered scalar receiving boundary values.
    Scalar,
    /// West–east face velocity used for west/east inflow tests.
    WestEastVelocity,
    /// South–north face velocity used for south/north inflow tests.
    SouthNorthVelocity,
}

impl fmt::Display for SpecifiedBoundaryFlowField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar => formatter.write_str("scalar"),
            Self::WestEastVelocity => formatter.write_str("west-east velocity"),
            Self::SouthNorthVelocity => formatter.write_str("south-north velocity"),
        }
    }
}
