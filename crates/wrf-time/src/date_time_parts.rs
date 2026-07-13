/// Normalized components returned from a [`crate::ModelTime`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTimeParts {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) fractional_numerator: i64,
    pub(crate) fractional_denominator: i64,
}

impl DateTimeParts {
    /// Returns the signed proleptic Gregorian year.
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Returns the one-based month.
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Returns the one-based day of month.
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Returns the zero-based hour of day.
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// Returns the zero-based minute of hour.
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// Returns the integral second of minute.
    pub const fn second(self) -> u8 {
        self.second
    }

    /// Returns the normalized fractional-second numerator.
    pub const fn fractional_numerator(self) -> i64 {
        self.fractional_numerator
    }

    /// Returns the positive fractional-second denominator.
    pub const fn fractional_denominator(self) -> i64 {
        self.fractional_denominator
    }
}
