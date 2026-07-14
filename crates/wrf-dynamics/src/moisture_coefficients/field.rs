use std::fmt;

/// Mutable field role produced by WRF moisture-coefficient calculation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoistureCoefficientField {
    /// West-east momentum coefficient (`cqu`).
    WestEastOutput,
    /// South-north momentum coefficient (`cqv`).
    SouthNorthOutput,
    /// Vertical momentum coefficient (`cqw`).
    VerticalOutput,
}

impl fmt::Display for MoistureCoefficientField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WestEastOutput => formatter.write_str("west-east moisture coefficient output"),
            Self::SouthNorthOutput => {
                formatter.write_str("south-north moisture coefficient output")
            }
            Self::VerticalOutput => formatter.write_str("vertical moisture coefficient output"),
        }
    }
}
