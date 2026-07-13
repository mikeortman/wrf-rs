/// Civil date and time components used to construct [`crate::ModelTime`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTimeComponents {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) fractional_numerator: i64,
    pub(crate) fractional_denominator: i64,
}

impl DateTimeComponents {
    /// Creates integral-second civil date and time components.
    pub const fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
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
