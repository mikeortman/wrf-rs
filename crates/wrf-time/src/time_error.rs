use std::fmt;

/// A failure to construct or operate on WRF model time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeError {
    /// A month fell outside the Gregorian range `1..=12`.
    InvalidMonth(u8),
    /// A day does not exist in the specified month and year.
    InvalidDay {
        /// Proleptic Gregorian year supplied by the caller.
        year: i32,
        /// One-based Gregorian month supplied by the caller.
        month: u8,
        /// One-based day of month that is invalid for the year and month.
        day: u8,
    },
    /// An hour fell outside `0..=23`.
    InvalidHour(u8),
    /// A minute fell outside `0..=59`.
    InvalidMinute(u8),
    /// A second fell outside `0..=59`.
    InvalidSecond(u8),
    /// A non-zero fraction was supplied with a zero denominator.
    ZeroDenominator,
    /// Interval division used a zero divisor.
    DivisionByZero,
    /// A clock stop precedes its start.
    ClockStopBeforeStart,
    /// A clock step is zero or negative.
    NonPositiveTimeStep,
}

/// The typed result returned by the model-time boundary.
pub type TimeResult<T> = Result<T, TimeError>;

impl fmt::Display for TimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMonth(month) => write!(formatter, "month {month} is outside 1..=12"),
            Self::InvalidDay { year, month, day } => {
                write!(
                    formatter,
                    "day {day} does not exist in {year:04}-{month:02}"
                )
            }
            Self::InvalidHour(hour) => write!(formatter, "hour {hour} is outside 0..=23"),
            Self::InvalidMinute(minute) => {
                write!(formatter, "minute {minute} is outside 0..=59")
            }
            Self::InvalidSecond(second) => {
                write!(formatter, "second {second} is outside 0..=59")
            }
            Self::ZeroDenominator => formatter.write_str("fraction denominator cannot be zero"),
            Self::DivisionByZero => formatter.write_str("time interval divisor cannot be zero"),
            Self::ClockStopBeforeStart => {
                formatter.write_str("clock stop time cannot precede its start time")
            }
            Self::NonPositiveTimeStep => {
                formatter.write_str("clock time step must be greater than zero")
            }
        }
    }
}

impl std::error::Error for TimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_explains_invalid_day_with_its_date_context() {
        let error = TimeError::InvalidDay {
            year: 2003,
            month: 2,
            day: 29,
        };

        assert_eq!(error.to_string(), "day 29 does not exist in 2003-02");
    }
}
