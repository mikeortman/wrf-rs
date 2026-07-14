use std::fmt;

/// Vertical coefficient used by the diabatic-heating term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DryTendencyAssemblyCoefficient {
    /// Multiplier applied to full column mass (`c1`).
    FullMassMultiplier,
    /// Additive vertical offset (`c2`).
    VerticalOffset,
}

impl fmt::Display for DryTendencyAssemblyCoefficient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FullMassMultiplier => "full-mass multiplier",
            Self::VerticalOffset => "vertical offset",
        })
    }
}
