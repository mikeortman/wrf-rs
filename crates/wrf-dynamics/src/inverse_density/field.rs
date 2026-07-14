use std::fmt;

/// Field role used by full inverse-density shape validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InverseDensityField {
    /// Full inverse-density output (`alt`).
    FullOutput,
    /// Perturbation inverse density (`al`).
    Perturbation,
    /// Base-state inverse density (`alb`).
    BaseState,
}

impl fmt::Display for InverseDensityField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullOutput => formatter.write_str("full inverse-density output"),
            Self::Perturbation => formatter.write_str("perturbation inverse density"),
            Self::BaseState => formatter.write_str("base-state inverse density"),
        }
    }
}
