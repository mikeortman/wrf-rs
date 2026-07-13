use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use crate::rational::Rational;
use crate::{IntervalParts, ModelTime, TimeIntervalComponents, TimeResult};

const SECONDS_PER_MINUTE: i128 = 60;
const SECONDS_PER_HOUR: i128 = 3_600;
const SECONDS_PER_DAY: i128 = 86_400;

/// A fixed WRF model-time duration with exact rational-second precision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeInterval {
    pub(crate) seconds: Rational,
}

impl TimeInterval {
    /// Constructs and normalizes a fixed duration from possibly mixed-sign components.
    pub fn try_new(components: TimeIntervalComponents) -> TimeResult<Self> {
        let integral_seconds = i128::from(components.days) * SECONDS_PER_DAY
            + i128::from(components.hours) * SECONDS_PER_HOUR
            + i128::from(components.minutes) * SECONDS_PER_MINUTE
            + i128::from(components.seconds);
        let fractional_seconds = Self::create_fraction(
            components.fractional_numerator,
            components.fractional_denominator,
        )?;

        Ok(Self {
            seconds: Rational::from_integer(integral_seconds) + fractional_seconds,
        })
    }

    /// Constructs an integral-second duration.
    pub fn from_seconds(seconds: i64) -> Self {
        Self {
            seconds: Rational::from_integer(i128::from(seconds)),
        }
    }

    /// Divides this interval by another and truncates the result toward zero, matching WRF.
    pub fn calculate_truncating_ratio(self, other: Self) -> TimeResult<i64> {
        self.seconds.calculate_truncating_ratio(other.seconds)
    }

    /// Returns whether the duration is greater than zero.
    pub fn is_positive(self) -> bool {
        self.seconds.numerator() > 0
    }

    /// Returns normalized absolute components and the interval sign.
    pub fn parts(self) -> IntervalParts {
        let is_negative = self.seconds.numerator() < 0;
        let absolute_seconds = Rational::try_new(
            self.seconds.numerator().unsigned_abs() as i128,
            self.seconds.denominator(),
        )
        .unwrap_or_else(|_| unreachable!("existing rational denominator is non-zero"));
        let (whole_seconds, fractional_numerator, fractional_denominator) =
            absolute_seconds.split_floor();

        IntervalParts {
            is_negative,
            days: (whole_seconds / SECONDS_PER_DAY) as i64,
            hours: ((whole_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR) as u8,
            minutes: ((whole_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u8,
            seconds: (whole_seconds % SECONDS_PER_MINUTE) as u8,
            fractional_numerator: fractional_numerator as i64,
            fractional_denominator: fractional_denominator as i64,
        }
    }

    pub(crate) const fn from_rational(seconds: Rational) -> Self {
        Self { seconds }
    }

    fn create_fraction(numerator: i64, denominator: i64) -> TimeResult<Rational> {
        if numerator == 0 && denominator == 0 {
            return Ok(Rational::from_integer(0));
        }

        Rational::try_new(i128::from(numerator), i128::from(denominator))
    }
}

impl Add for TimeInterval {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from_rational(self.seconds + other.seconds)
    }
}

impl Add<ModelTime> for TimeInterval {
    type Output = ModelTime;

    fn add(self, time: ModelTime) -> ModelTime {
        time + self
    }
}

impl Sub for TimeInterval {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::from_rational(self.seconds - other.seconds)
    }
}

impl Mul<i64> for TimeInterval {
    type Output = Self;

    fn mul(self, multiplier: i64) -> Self {
        let seconds = self
            .seconds
            .try_multiply(multiplier)
            .unwrap_or_else(|_| unreachable!("integer multiplication preserves denominator"));
        Self::from_rational(seconds)
    }
}

impl Div<i64> for TimeInterval {
    type Output = TimeResult<Self>;

    fn div(self, divisor: i64) -> Self::Output {
        Ok(Self::from_rational(self.seconds.try_divide(divisor)?))
    }
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts = self.parts();
        let sign = if parts.is_negative() { "-" } else { "" };
        write!(
            formatter,
            "{sign}{:010}_{:03}:{:03}:{:03}",
            parts.days(),
            parts.hours(),
            parts.minutes(),
            parts.seconds()
        )?;
        if parts.fractional_numerator() != 0 {
            let fraction_sign = if parts.is_negative() { '-' } else { '+' };
            write!(
                formatter,
                "{fraction_sign}{:02}/{:02}",
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

    struct IntervalBinaryCase {
        name: &'static str,
        left: TimeInterval,
        right: TimeInterval,
        expected: TimeInterval,
    }

    struct IntervalScaleCase {
        name: &'static str,
        interval: TimeInterval,
        scalar: i64,
        expected: TimeInterval,
    }

    struct IntervalRatioCase {
        name: &'static str,
        numerator: TimeInterval,
        denominator: TimeInterval,
        expected: i64,
    }

    fn create_interval(days: i64, hours: i64, minutes: i64, seconds: i64) -> TimeInterval {
        TimeInterval::try_new(TimeIntervalComponents::new(days, hours, minutes, seconds)).unwrap()
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
            TimeIntervalComponents::new(days, hours, minutes, seconds)
                .with_fraction(numerator, denominator),
        )
        .unwrap()
    }

    #[test]
    fn display_matches_wrf_integral_interval_cases() {
        // WRF Test1.F90: printTI_1, printTI_2, printTI_N1, and printTI_M1.
        let cases = [
            (
                "printTI_1",
                create_interval(0, 0, 0, 0),
                "0000000000_000:000:000",
            ),
            (
                "printTI_2",
                create_interval(500, 0, 0, 7270),
                "0000000500_002:001:010",
            ),
            (
                "printTI_D1",
                create_interval(0, 0, 0, 0),
                "0000000000_000:000:000",
            ),
            (
                "printTI_D2",
                create_interval(0, 0, 0, 0),
                "0000000000_000:000:000",
            ),
            (
                "printTI_N1",
                create_interval(-3, -1, -20, -10),
                "-0000000003_001:020:010",
            ),
            (
                "printTI_M1",
                create_interval(-3, 1, 20, 10),
                "-0000000002_022:039:050",
            ),
        ];

        for (name, actual, expected) in cases {
            assert_eq!(actual.to_string(), expected, "{name}");
        }
    }

    #[test]
    fn display_matches_wrf_signed_fractional_interval_cases() {
        // WRF Test1.F90: printTI_F1 through printTI_F4.
        let cases = [
            (
                "printTI_F1",
                (3, 1, 20, 10, 1, 3),
                "0000000003_001:020:010+01/03",
            ),
            (
                "printTI_F2",
                (3, 1, 20, 10, 5, 3),
                "0000000003_001:020:011+02/03",
            ),
            (
                "printTI_F3",
                (-3, -1, -20, -10, -1, 3),
                "-0000000003_001:020:010-01/03",
            ),
            (
                "printTI_F4",
                (-3, -1, -20, -10, 1, 3),
                "-0000000003_001:020:009-02/03",
            ),
        ];

        for (name, (days, hours, minutes, seconds, numerator, denominator), expected) in cases {
            let components = TimeIntervalComponents::new(days, hours, minutes, seconds)
                .with_fraction(numerator, denominator);
            let actual = TimeInterval::try_new(components).unwrap();
            assert_eq!(actual.to_string(), expected, "{name}");
        }
    }

    #[test]
    fn addition_and_subtraction_match_every_active_wrf_interval_case() {
        let addition_cases = [
            IntervalBinaryCase {
                name: "AddTI_TI_TI1",
                left: create_interval(3, 1, 20, 10),
                right: create_interval(1, 1, 10, 10),
                expected: create_interval(4, 2, 30, 20),
            },
            IntervalBinaryCase {
                name: "AddTI_TI_TI2",
                left: create_interval(-3, -1, -20, -10),
                right: create_interval(1, 1, 10, 10),
                expected: create_interval(-2, 0, -10, 0),
            },
            IntervalBinaryCase {
                name: "AddTI_TI_TI3",
                left: create_interval(-3, -1, -20, -10),
                right: create_interval(-1, -1, -10, -10),
                expected: create_interval(-4, -2, -30, -20),
            },
        ];
        for case in addition_cases {
            assert_eq!(case.left + case.right, case.expected, "{}", case.name);
        }

        let subtraction_cases = [
            IntervalBinaryCase {
                name: "SubtractTI_TI_TI1",
                left: create_interval(3, 1, 20, 10),
                right: create_interval(1, 1, 10, 10),
                expected: create_interval(2, 0, 10, 0),
            },
            IntervalBinaryCase {
                name: "SubtractTI_TI_TI2",
                left: create_interval(3, 1, 20, 10),
                right: create_interval(-1, -1, -10, -10),
                expected: create_interval(4, 2, 30, 20),
            },
            IntervalBinaryCase {
                name: "SubtractTI_TI_TI3",
                left: create_interval(-1, -1, -10, -10),
                right: create_interval(-3, -1, -20, -10),
                expected: create_interval(2, 0, 10, 0),
            },
            IntervalBinaryCase {
                name: "SubtractTI_TI_TIN1",
                left: create_interval(1, 1, 10, 10),
                right: create_interval(3, 1, 20, 10),
                expected: create_interval(-2, 0, -10, 0),
            },
            IntervalBinaryCase {
                name: "SubtractTI_TI_TIN2",
                left: create_interval(-1, -1, -10, -10),
                right: create_interval(3, 1, 20, 10),
                expected: create_interval(-4, -2, -30, -20),
            },
        ];
        for case in subtraction_cases {
            assert_eq!(case.left - case.right, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn multiplication_matches_every_active_wrf_interval_case() {
        let cases = [
            IntervalScaleCase {
                name: "MultiplyTI_TI_INT1",
                interval: create_interval(3, 12, 18, 33),
                scalar: 2,
                expected: create_interval(6, 24, 37, 6),
            },
            IntervalScaleCase {
                name: "MultiplyTI_TI_INT2",
                interval: create_interval(350, 23, 50, 50),
                scalar: 2,
                expected: create_interval(701, 23, 41, 40),
            },
            IntervalScaleCase {
                name: "MultiplyTI_TI_INT3",
                interval: create_fractional_interval(0, 0, 0, 1, 3, 4),
                scalar: 8,
                expected: create_interval(0, 0, 0, 14),
            },
        ];

        for case in cases {
            assert_eq!(case.interval * case.scalar, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn integer_division_matches_every_active_wrf_interval_case() {
        let cases = [
            IntervalScaleCase {
                name: "DivideTI_TI_INT1",
                interval: create_interval(3, 12, 18, 33),
                scalar: 3,
                expected: create_interval(1, 4, 6, 11),
            },
            IntervalScaleCase {
                name: "DivideTI_TI_INT2",
                interval: create_interval(3, 12, 18, 33),
                scalar: 4,
                expected: create_fractional_interval(0, 21, 4, 38, 1, 4),
            },
            IntervalScaleCase {
                name: "DivideTI_TI_INT3",
                interval: create_fractional_interval(0, 0, 0, 1, 3, 4),
                scalar: 5,
                expected: create_fractional_interval(0, 0, 0, 0, 7, 20),
            },
        ];

        for case in cases {
            assert_eq!(
                case.interval / case.scalar,
                Ok(case.expected),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn calculate_truncating_ratio_matches_every_active_wrf_case() {
        let base = create_interval(3, 12, 18, 33);
        let cases = [
            IntervalRatioCase {
                name: "DivideINT_TI_TI1",
                numerator: base,
                denominator: base,
                expected: 1,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TI2",
                numerator: create_interval(6, 24, 36, 66),
                denominator: base,
                expected: 2,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TI3",
                numerator: create_interval(0, 0, 0, 0),
                denominator: base,
                expected: 0,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TI4",
                numerator: create_interval(1, 0, 0, 0),
                denominator: create_interval(0, 1, 0, 0),
                expected: 24,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TI5",
                numerator: create_interval(1, 0, 0, 0),
                denominator: create_interval(0, 0, 1, 0),
                expected: 1_440,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TI6",
                numerator: create_interval(1, 0, 0, 0),
                denominator: create_interval(0, 0, 0, 1),
                expected: 86_400,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIR1",
                numerator: create_interval(0, 0, 0, 3),
                denominator: create_interval(0, 0, 0, 2),
                expected: 1,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIR2",
                numerator: create_interval(1, 0, 0, 2),
                denominator: create_interval(1, 0, 0, 3),
                expected: 0,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIF1",
                numerator: create_fractional_interval(0, 0, 0, 0, 3, 4),
                denominator: create_fractional_interval(0, 0, 0, 0, 3, 4),
                expected: 1,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIF2",
                numerator: create_fractional_interval(0, 0, 0, 0, 6, 8),
                denominator: create_fractional_interval(0, 0, 0, 0, 3, 4),
                expected: 1,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIF3",
                numerator: create_fractional_interval(0, 0, 0, 0, 3, 4),
                denominator: create_fractional_interval(0, 0, 0, 0, 4, 3),
                expected: 0,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIF4",
                numerator: create_fractional_interval(0, 0, 0, 2, 3, 4),
                denominator: create_fractional_interval(0, 0, 0, 1, 1, 3),
                expected: 2,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIN1",
                numerator: create_interval(-6, -24, -36, -66),
                denominator: base,
                expected: -2,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIN2",
                numerator: create_interval(6, 24, 36, 66),
                denominator: create_interval(-3, -12, -18, -33),
                expected: -2,
            },
            IntervalRatioCase {
                name: "DivideINT_TI_TIN3",
                numerator: create_interval(-6, -24, -36, -66),
                denominator: create_interval(-3, -12, -18, -33),
                expected: 2,
            },
        ];

        for case in cases {
            assert_eq!(
                case.numerator.calculate_truncating_ratio(case.denominator),
                Ok(case.expected),
                "{}",
                case.name
            );
        }
    }
}
