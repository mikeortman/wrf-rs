/// Components used to construct a fixed-duration [`crate::TimeInterval`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeIntervalComponents {
    pub(crate) days: i64,
    pub(crate) hours: i64,
    pub(crate) minutes: i64,
    pub(crate) seconds: i64,
    pub(crate) fractional_numerator: i64,
    pub(crate) fractional_denominator: i64,
}

impl TimeIntervalComponents {
    /// Creates fixed-duration components; values may be mixed-sign or unnormalized.
    pub const fn new(days: i64, hours: i64, minutes: i64, seconds: i64) -> Self {
        Self {
            days,
            hours,
            minutes,
            seconds,
            fractional_numerator: 0,
            fractional_denominator: 0,
        }
    }

    /// Adds a rational fraction of one second to these components.
    pub const fn with_fraction(mut self, numerator: i64, denominator: i64) -> Self {
        self.fractional_numerator = numerator;
        self.fractional_denominator = denominator;
        self
    }
}
