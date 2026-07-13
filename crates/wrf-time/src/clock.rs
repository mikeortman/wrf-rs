use crate::{ModelTime, TimeError, TimeInterval, TimeResult};

/// A deterministic WRF model clock advanced by an exact fixed interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Clock {
    start: ModelTime,
    stop: ModelTime,
    current: ModelTime,
    time_step: TimeInterval,
    step_count: u64,
}

impl Clock {
    /// Creates a clock with an inclusive stop and a positive fixed time step.
    pub fn try_new(start: ModelTime, stop: ModelTime, time_step: TimeInterval) -> TimeResult<Self> {
        if stop < start {
            return Err(TimeError::ClockStopBeforeStart);
        }
        if !time_step.is_positive() {
            return Err(TimeError::NonPositiveTimeStep);
        }

        Ok(Self {
            start,
            stop,
            current: start,
            time_step,
            step_count: 0,
        })
    }

    /// Advances one exact step and returns false after reaching or passing the stop.
    pub fn advance(&mut self) -> bool {
        if self.current >= self.stop {
            return false;
        }

        self.current = self.current + self.time_step;
        self.step_count += 1;
        true
    }

    /// Returns the immutable clock start.
    pub const fn start(&self) -> ModelTime {
        self.start
    }

    /// Returns the current model time.
    pub const fn current(&self) -> ModelTime {
        self.current
    }

    /// Returns the number of completed advances.
    pub const fn step_count(&self) -> u64 {
        self.step_count
    }
}

#[cfg(test)]
mod tests {
    use crate::{DateTimeComponents, TimeIntervalComponents};

    use super::*;

    fn create_time(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> ModelTime {
        ModelTime::try_new(DateTimeComponents::new(
            year, month, day, hour, minute, second,
        ))
        .unwrap()
    }

    #[test]
    fn advance_matches_wrf_simple_clock_case() {
        let name = "SimpleClockAdvance";
        let start = create_time(2002, 12, 27, 3, 0, 0);
        let stop = create_time(2002, 12, 28, 8, 0, 0);
        let mut clock = Clock::try_new(start, stop, TimeInterval::from_seconds(600)).unwrap();

        assert_eq!(clock.start(), start);
        assert!((clock.current().day_of_year_fraction() - 361.125).abs() < f64::EPSILON);
        while clock.advance() {}

        assert_eq!(clock.current(), stop, "{name}");
        assert_eq!(clock.step_count(), 174, "{name}");
    }

    #[test]
    fn fractional_advance_crosses_wrf_leap_year_boundary_exactly() {
        let name = "LeapYearFractionClockAdvance";
        let start = create_time(2004, 12, 31, 23, 58, 0);
        let stop = create_time(2005, 1, 1, 0, 2, 0);
        let time_step =
            TimeInterval::try_new(TimeIntervalComponents::new(0, 0, 0, 13).with_fraction(1, 3))
                .unwrap();
        let mut clock = Clock::try_new(start, stop, time_step).unwrap();

        while clock.advance() {}

        assert_eq!(clock.current(), stop, "{name}");
        assert_eq!(clock.step_count(), 18, "{name}");
    }

    #[test]
    fn hourly_advance_matches_wrf_standard_and_leap_year_cases() {
        let cases = [
            (
                "StdYearClockAdvance",
                create_time(2003, 12, 29, 9, 0, 0),
                create_time(2004, 1, 2, 9, 0, 0),
            ),
            (
                "LeapYearClockAdvance",
                create_time(2004, 12, 29, 9, 0, 0),
                create_time(2005, 1, 2, 9, 0, 0),
            ),
        ];

        for (name, start, stop) in cases {
            let mut clock = Clock::try_new(start, stop, TimeInterval::from_seconds(3_600)).unwrap();
            while clock.advance() {}

            assert_eq!(clock.current(), stop, "{name}");
            assert_eq!(clock.step_count(), 96, "{name}");
        }
    }

    #[test]
    fn try_new_rejects_non_positive_time_step() {
        let time = create_time(2004, 1, 1, 0, 0, 0);

        assert_eq!(
            Clock::try_new(time, time, TimeInterval::from_seconds(0)),
            Err(TimeError::NonPositiveTimeStep)
        );
    }
}
