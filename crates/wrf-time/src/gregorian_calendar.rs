use crate::{DateTimeComponents, TimeError, TimeResult};

/// Proleptic Gregorian calendar operations used by WRF model time.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GregorianCalendar;

impl GregorianCalendar {
    /// Returns whether `year` contains a Gregorian leap day.
    pub const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Returns the number of days in `month`, or zero for an invalid month.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    pub(crate) fn validate(components: DateTimeComponents) -> TimeResult<()> {
        if !(1..=12).contains(&components.month) {
            return Err(TimeError::InvalidMonth(components.month));
        }
        if components.day == 0
            || components.day > Self::days_in_month(components.year, components.month)
        {
            return Err(TimeError::InvalidDay {
                year: components.year,
                month: components.month,
                day: components.day,
            });
        }
        if components.hour > 23 {
            return Err(TimeError::InvalidHour(components.hour));
        }
        if components.minute > 59 {
            return Err(TimeError::InvalidMinute(components.minute));
        }
        if components.second > 59 {
            return Err(TimeError::InvalidSecond(components.second));
        }

        Ok(())
    }

    pub(crate) fn calculate_days_from_civil(year: i32, month: u8, day: u8) -> i128 {
        let adjusted_year = i128::from(year) - i128::from(month <= 2);
        let era = adjusted_year.div_euclid(400);
        let year_of_era = adjusted_year - era * 400;
        let shifted_month = i128::from(month) + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + i128::from(day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    pub(crate) fn calculate_civil_from_days(days: i128) -> (i32, u8, u8) {
        let shifted_days = days + 719_468;
        let era = shifted_days.div_euclid(146_097);
        let day_of_era = shifted_days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_prime = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
        let month = month_prime + if month_prime < 10 { 3 } else { -9 };
        year += i128::from(month <= 2);
        (year as i32, month as u8, day as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_leap_year_applies_century_rule() {
        assert!(!GregorianCalendar::is_leap_year(1900));
        assert!(GregorianCalendar::is_leap_year(2000));
        assert!(GregorianCalendar::is_leap_year(2004));
    }

    #[test]
    fn civil_day_conversion_round_trips_year_zero_and_negative_year() {
        for expected in [(0, 1, 1), (-2002, 2, 28), (2004, 2, 29)] {
            let days =
                GregorianCalendar::calculate_days_from_civil(expected.0, expected.1, expected.2);
            assert_eq!(GregorianCalendar::calculate_civil_from_days(days), expected);
        }
    }
}
