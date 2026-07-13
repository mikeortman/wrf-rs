use std::fmt;
use std::ops::{Add, Sub};

use crate::rational::Rational;
use crate::{DateTimeComponents, DateTimeParts, GregorianCalendar, TimeInterval, TimeResult};

const SECONDS_PER_MINUTE: i128 = 60;
const SECONDS_PER_HOUR: i128 = 3_600;
const SECONDS_PER_DAY: i128 = 86_400;

/// A WRF model timestamp on the proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModelTime {
    pub(crate) seconds: Rational,
}

impl ModelTime {
    /// Validates and constructs a timestamp from civil components.
    pub fn try_new(components: DateTimeComponents) -> TimeResult<Self> {
        GregorianCalendar::validate(components)?;

        let days = GregorianCalendar::calculate_days_from_civil(
            components.year,
            components.month,
            components.day,
        );
        let integral_seconds = days * SECONDS_PER_DAY
            + i128::from(components.hour) * SECONDS_PER_HOUR
            + i128::from(components.minute) * SECONDS_PER_MINUTE
            + i128::from(components.second);
        let fractional_seconds = Self::create_fraction(
            components.fractional_numerator,
            components.fractional_denominator,
        )?;

        Ok(Self {
            seconds: Rational::from_integer(integral_seconds) + fractional_seconds,
        })
    }

    /// Returns normalized civil and fractional-second components.
    pub fn parts(self) -> DateTimeParts {
        let (whole_seconds, fractional_numerator, fractional_denominator) =
            self.seconds.split_floor();
        let days = whole_seconds.div_euclid(SECONDS_PER_DAY);
        let seconds_of_day = whole_seconds.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = GregorianCalendar::calculate_civil_from_days(days);

        DateTimeParts {
            year,
            month,
            day,
            hour: (seconds_of_day / SECONDS_PER_HOUR) as u8,
            minute: ((seconds_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u8,
            second: (seconds_of_day % SECONDS_PER_MINUTE) as u8,
            fractional_numerator: fractional_numerator as i64,
            fractional_denominator: fractional_denominator as i64,
        }
    }

    /// Returns the one-based day within the current year.
    pub fn day_of_year(self) -> u16 {
        let parts = self.parts();
        let year_start = GregorianCalendar::calculate_days_from_civil(parts.year(), 1, 1);
        let current_day =
            GregorianCalendar::calculate_days_from_civil(parts.year(), parts.month(), parts.day());
        (current_day - year_start + 1) as u16
    }

    /// Returns WRF's one-based day of year with the elapsed day fraction.
    pub fn day_of_year_fraction(self) -> f64 {
        let parts = self.parts();
        let elapsed_seconds = f64::from(parts.hour()) * 3_600.0
            + f64::from(parts.minute()) * 60.0
            + f64::from(parts.second())
            + parts.fractional_numerator() as f64 / parts.fractional_denominator() as f64;
        f64::from(self.day_of_year()) + elapsed_seconds / 86_400.0
    }

    fn create_fraction(numerator: i64, denominator: i64) -> TimeResult<Rational> {
        if numerator == 0 && denominator == 0 {
            return Ok(Rational::from_integer(0));
        }

        Rational::try_new(i128::from(numerator), i128::from(denominator))
    }
}

impl Add<TimeInterval> for ModelTime {
    type Output = Self;

    fn add(self, interval: TimeInterval) -> Self {
        Self {
            seconds: self.seconds + interval.seconds,
        }
    }
}

impl Sub<TimeInterval> for ModelTime {
    type Output = Self;

    fn sub(self, interval: TimeInterval) -> Self {
        Self {
            seconds: self.seconds - interval.seconds,
        }
    }
}

impl Sub for ModelTime {
    type Output = TimeInterval;

    fn sub(self, other: Self) -> TimeInterval {
        TimeInterval::from_rational(self.seconds - other.seconds)
    }
}

impl fmt::Display for ModelTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = self.parts();
        write!(
            formatter,
            "{:04}-{:02}-{:02}_{:02}:{:02}:{:02}",
            parts.year(),
            parts.month(),
            parts.day(),
            parts.hour(),
            parts.minute(),
            parts.second()
        )?;
        if parts.fractional_numerator() != 0 {
            write!(
                formatter,
                "+{:02}/{:02}",
                parts.fractional_numerator(),
                parts.fractional_denominator()
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TimeIntervalCase {
        name: &'static str,
        time: ModelTime,
        interval: TimeInterval,
        expected: ModelTime,
    }

    struct TimeDifferenceCase {
        name: &'static str,
        left: ModelTime,
        right: ModelTime,
        expected: TimeInterval,
    }

    fn create_time(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> ModelTime {
        ModelTime::try_new(DateTimeComponents::new(
            year, month, day, hour, minute, second,
        ))
        .unwrap()
    }

    fn create_interval(days: i64, hours: i64, minutes: i64, seconds: i64) -> TimeInterval {
        TimeInterval::try_new(crate::TimeIntervalComponents::new(
            days, hours, minutes, seconds,
        ))
        .unwrap()
    }

    fn create_fractional_time(
        components: DateTimeComponents,
        numerator: i64,
        denominator: i64,
    ) -> ModelTime {
        ModelTime::try_new(components.with_fraction(numerator, denominator)).unwrap()
    }

    fn create_fractional_interval(
        days: i64,
        hours: i64,
        minutes: i64,
        seconds: i64,
        numerator: i64,
        denominator: i64,
    ) -> TimeInterval {
        TimeInterval::try_new(
            crate::TimeIntervalComponents::new(days, hours, minutes, seconds)
                .with_fraction(numerator, denominator),
        )
        .unwrap()
    }

    #[test]
    fn display_matches_wrf_print_time_cases() {
        // WRF Test1.F90: printT_1 through printT_6.
        let cases = [
            (
                "printT_1",
                create_time(2001, 12, 3, 1, 20, 10),
                "2001-12-03_01:20:10",
            ),
            (
                "printT_2",
                create_time(0, 1, 1, 0, 0, 0),
                "0000-01-01_00:00:00",
            ),
            (
                "printT_3",
                create_time(2003, 12, 30, 23, 59, 50),
                "2003-12-30_23:59:50",
            ),
            (
                "printT_4",
                create_time(2003, 12, 31, 23, 59, 50),
                "2003-12-31_23:59:50",
            ),
            (
                "printT_5",
                create_time(2004, 12, 30, 23, 59, 50),
                "2004-12-30_23:59:50",
            ),
            (
                "printT_6",
                create_time(2004, 12, 31, 23, 59, 50),
                "2004-12-31_23:59:50",
            ),
            (
                "printT_D1",
                create_time(0, 1, 1, 0, 0, 0),
                "0000-01-01_00:00:00",
            ),
            (
                "printT_D2",
                create_time(0, 1, 1, 0, 0, 0),
                "0000-01-01_00:00:00",
            ),
        ];

        for (name, actual, expected) in cases {
            assert_eq!(actual.to_string(), expected, "{name}");
        }
    }

    #[test]
    fn display_normalizes_wrf_fractional_time_cases() {
        // WRF Test1.F90: printT_F1 through printT_F4.
        let cases = [
            ("printT_F1", (1, 3), "2001-12-03_01:20:10+01/03"),
            ("printT_F2", (4, 3), "2001-12-03_01:20:11+01/03"),
            ("printT_F3", (12, 3), "2001-12-03_01:20:14"),
            ("printT_F4", (-1, 3), "2001-12-03_01:20:09+02/03"),
        ];

        for (name, (numerator, denominator), expected) in cases {
            let components = DateTimeComponents::new(2001, 12, 3, 1, 20, 10)
                .with_fraction(numerator, denominator);
            let actual = ModelTime::try_new(components).unwrap();
            assert_eq!(actual.to_string(), expected, "{name}");
        }
    }

    #[test]
    fn addition_matches_every_active_wrf_time_interval_case() {
        let cases = [
            TimeIntervalCase {
                name: "AddT_T_TI1",
                time: create_time(2001, 12, 3, 1, 20, 10),
                interval: create_interval(0, 3, 10, 10),
                expected: create_time(2001, 12, 3, 4, 30, 20),
            },
            TimeIntervalCase {
                name: "AddT_T_TI2",
                time: create_time(2001, 12, 31, 22, 30, 0),
                interval: create_interval(0, 4, 10, 10),
                expected: create_time(2002, 1, 1, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI3",
                time: create_time(2003, 12, 31, 22, 30, 0),
                interval: create_interval(0, 4, 10, 10),
                expected: create_time(2004, 1, 1, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI4",
                time: create_time(2004, 12, 31, 22, 30, 0),
                interval: create_interval(0, 4, 10, 10),
                expected: create_time(2005, 1, 1, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI5",
                time: create_time(2004, 12, 30, 22, 30, 0),
                interval: create_interval(0, 4, 10, 10),
                expected: create_time(2004, 12, 31, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI7",
                time: create_time(2004, 12, 30, 4, 30, 0),
                interval: create_interval(365, 4, 10, 10),
                expected: create_time(2005, 12, 30, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI8",
                time: create_time(2004, 12, 30, 4, 30, 0),
                interval: create_interval(367, 4, 10, 10),
                expected: create_time(2006, 1, 1, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI9",
                time: create_time(2003, 12, 30, 4, 30, 0),
                interval: create_interval(365, 4, 10, 10),
                expected: create_time(2004, 12, 29, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI10",
                time: create_time(2003, 12, 30, 4, 30, 0),
                interval: create_interval(366, 4, 10, 10),
                expected: create_time(2004, 12, 30, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI11",
                time: create_time(2003, 12, 30, 4, 30, 0),
                interval: create_interval(367, 4, 10, 10),
                expected: create_time(2004, 12, 31, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI12",
                time: create_time(2003, 12, 30, 4, 30, 0),
                interval: create_interval(368, 4, 10, 10),
                expected: create_time(2005, 1, 1, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI13",
                time: create_time(2004, 3, 30, 4, 30, 0),
                interval: create_interval(365, 4, 10, 10),
                expected: create_time(2005, 3, 30, 8, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI14",
                time: create_time(2004, 3, 30, 4, 30, 0),
                interval: create_interval(365, 22, 10, 10),
                expected: create_time(2005, 3, 31, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI15",
                time: create_time(2004, 3, 30, 4, 30, 0),
                interval: create_interval(366, 22, 10, 10),
                expected: create_time(2005, 4, 1, 2, 40, 10),
            },
            TimeIntervalCase {
                name: "AddT_T_TI_F1",
                time: create_fractional_time(
                    DateTimeComponents::new(2004, 12, 31, 22, 30, 0),
                    1,
                    3,
                ),
                interval: create_fractional_interval(0, 4, 10, 10, 1, 3),
                expected: create_fractional_time(
                    DateTimeComponents::new(2005, 1, 1, 2, 40, 10),
                    2,
                    3,
                ),
            },
        ];

        for case in cases {
            assert_eq!(case.time + case.interval, case.expected, "{}", case.name);
        }

        let reverse_cases = [
            TimeIntervalCase {
                name: "AddT_TI_T1",
                time: create_time(2001, 12, 3, 1, 20, 10),
                interval: create_interval(0, 3, 10, 10),
                expected: create_time(2001, 12, 3, 4, 30, 20),
            },
            TimeIntervalCase {
                name: "AddT_TI_T2",
                time: create_time(2001, 12, 31, 22, 30, 0),
                interval: create_interval(0, 4, 10, 10),
                expected: create_time(2002, 1, 1, 2, 40, 10),
            },
        ];

        for case in reverse_cases {
            assert_eq!(case.interval + case.time, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn subtraction_matches_every_active_wrf_time_interval_case() {
        let cases = [
            TimeIntervalCase {
                name: "SubtractT_T_TI1",
                time: create_time(2001, 12, 3, 1, 20, 10),
                interval: create_interval(0, 3, 10, 10),
                expected: create_time(2001, 12, 2, 22, 10, 0),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI2",
                time: create_time(2005, 1, 1, 0, 0, 0),
                interval: create_interval(0, 0, 0, 10),
                expected: create_time(2004, 12, 31, 23, 59, 50),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI3",
                time: create_time(2004, 1, 1, 0, 0, 0),
                interval: create_interval(0, 0, 0, 10),
                expected: create_time(2003, 12, 31, 23, 59, 50),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI4",
                time: create_time(2003, 1, 1, 0, 0, 0),
                interval: create_interval(0, 0, 0, 10),
                expected: create_time(2002, 12, 31, 23, 59, 50),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI5",
                time: create_time(2005, 4, 1, 2, 40, 10),
                interval: create_interval(366, 22, 10, 10),
                expected: create_time(2004, 3, 30, 4, 30, 0),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI6",
                time: create_time(2006, 1, 1, 8, 40, 10),
                interval: create_interval(367, 4, 10, 10),
                expected: create_time(2004, 12, 30, 4, 30, 0),
            },
            TimeIntervalCase {
                name: "SubtractT_T_TI_F1",
                time: create_time(2005, 1, 1, 0, 0, 0),
                interval: create_fractional_interval(0, 0, 0, 1, 1, 3),
                expected: create_fractional_time(
                    DateTimeComponents::new(2004, 12, 31, 23, 59, 58),
                    2,
                    3,
                ),
            },
        ];

        for case in cases {
            assert_eq!(case.time - case.interval, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn difference_matches_every_active_wrf_time_case() {
        let cases = [
            TimeDifferenceCase {
                name: "SubtractTI_T_T1",
                left: create_time(2001, 12, 3, 1, 20, 10),
                right: create_time(2001, 12, 1, 1, 10, 10),
                expected: create_interval(2, 0, 10, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T2",
                left: create_time(2002, 1, 1, 0, 0, 0),
                right: create_time(2001, 12, 31, 23, 59, 50),
                expected: create_interval(0, 0, 0, 10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T3",
                left: create_time(2005, 1, 1, 0, 0, 0),
                right: create_time(2004, 12, 31, 23, 59, 50),
                expected: create_interval(0, 0, 0, 10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T4",
                left: create_time(2003, 3, 1, 0, 0, 0),
                right: create_time(2003, 2, 28, 23, 59, 50),
                expected: create_interval(0, 0, 0, 10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T5",
                left: create_time(2004, 3, 1, 0, 0, 0),
                right: create_time(2004, 2, 28, 23, 59, 50),
                expected: create_interval(1, 0, 0, 10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T6",
                left: create_time(2002, 2, 28, 0, 0, 0),
                right: create_time(2002, 2, 28, 0, 0, 0),
                expected: create_interval(0, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T7",
                left: create_time(2003, 2, 28, 0, 0, 0),
                right: create_time(2002, 2, 28, 0, 0, 0),
                expected: create_interval(365, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T8",
                left: create_time(2004, 2, 28, 0, 0, 0),
                right: create_time(2003, 2, 28, 0, 0, 0),
                expected: create_interval(365, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T9",
                left: create_time(2005, 2, 28, 0, 0, 0),
                right: create_time(2004, 2, 28, 0, 0, 0),
                expected: create_interval(366, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T10",
                left: create_time(2003, 3, 1, 0, 0, 0),
                right: create_time(2002, 2, 28, 0, 0, 0),
                expected: create_interval(366, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T11",
                left: create_time(2005, 3, 1, 0, 0, 0),
                right: create_time(2004, 2, 28, 0, 0, 0),
                expected: create_interval(367, 0, 0, 0),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T12",
                left: create_time(2005, 3, 1, 0, 0, 0),
                right: create_time(2004, 2, 28, 23, 59, 50),
                expected: create_interval(366, 0, 0, 10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T13",
                left: create_time(2004, 2, 28, 23, 59, 50),
                right: create_time(2005, 3, 1, 0, 0, 0),
                expected: create_interval(-366, 0, 0, -10),
            },
            TimeDifferenceCase {
                name: "SubtractTI_T_T14",
                left: create_time(-2002, 2, 28, 0, 0, 0),
                right: create_time(-2002, 2, 28, 0, 0, 0),
                expected: create_interval(0, 0, 0, 0),
            },
        ];

        for case in cases {
            assert_eq!(case.left - case.right, case.expected, "{}", case.name);
        }
    }
}
