//! Model-time primitives translated from WRF's bundled ESMF time manager.
//!
//! The compatibility reference is WRF v4.7.1 under
//! `external/esmf_time_f90`. Model time is deliberately independent of civil
//! time zones and leap seconds.

#![forbid(unsafe_code)]

mod clock;
mod date_time_components;
mod date_time_parts;
mod gregorian_calendar;
mod interval_parts;
mod model_time;
mod rational;
mod time_error;
mod time_interval;
mod time_interval_components;

pub use clock::Clock;
pub use date_time_components::DateTimeComponents;
pub use date_time_parts::DateTimeParts;
pub use gregorian_calendar::GregorianCalendar;
pub use interval_parts::IntervalParts;
pub use model_time::ModelTime;
pub use time_error::{TimeError, TimeResult};
pub use time_interval::TimeInterval;
pub use time_interval_components::TimeIntervalComponents;
