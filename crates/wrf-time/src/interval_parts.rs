/// Normalized absolute components and sign of a [`crate::TimeInterval`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntervalParts {
    pub(crate) is_negative: bool,
    pub(crate) days: i64,
    pub(crate) hours: u8,
    pub(crate) minutes: u8,
    pub(crate) seconds: u8,
    pub(crate) fractional_numerator: i64,
    pub(crate) fractional_denominator: i64,
}

impl IntervalParts {
    /// Returns whether the complete interval is less than zero.
    pub const fn is_negative(self) -> bool {
        self.is_negative
    }

    /// Returns the absolute whole-day component.
    pub const fn days(self) -> i64 {
        self.days
    }

    /// Returns the absolute normalized hour component.
    pub const fn hours(self) -> u8 {
        self.hours
    }

    /// Returns the absolute normalized minute component.
    pub const fn minutes(self) -> u8 {
        self.minutes
    }

    /// Returns the absolute normalized integral-second component.
    pub const fn seconds(self) -> u8 {
        self.seconds
    }

    /// Returns the absolute normalized fractional-second numerator.
    pub const fn fractional_numerator(self) -> i64 {
        self.fractional_numerator
    }

    /// Returns the positive normalized fractional-second denominator.
    pub const fn fractional_denominator(self) -> i64 {
        self.fractional_denominator
    }
}
