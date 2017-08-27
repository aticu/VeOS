//! Handles time related functionality.

use arch::get_current_timestamp;
use core::fmt;

/// Represents a unit of time.
pub enum Time {
    /// A microsecond (µ-second) is 1/1,000,000 of a second.
    Microseconds(i64),
    /// A millisecond is 1/1,000 of a second.
    Milliseconds(i64),
    /// A second is 1/60 of a minute.
    Seconds(i64),
    /// A minute is 60 seconds.
    Minutes(i64),
    /// An hour is 60 minutes.
    Hours(i64),
    /// A day is 24 hours.
    Days(i64)
}

/// The number of µ-seconds in a millisecond.
const MILLISECOND_MULTIPLIER: i64 = 1000;

/// The number of µ-seconds in a second.
const SECOND_MULTIPLIER: i64 = MILLISECOND_MULTIPLIER * 1000;

/// The number of µ-seconds in a minute.
const MINUTE_MULTIPLIER: i64 = SECOND_MULTIPLIER * 60;

/// The number of µ-seconds in an hour.
const HOUR_MULTIPLIER: i64 = MINUTE_MULTIPLIER * 60;

/// The number of µ-seconds in a day.
const DAY_MULTIPLIER: i64 = HOUR_MULTIPLIER * 24;

impl Time {
    /// Returns the µ-second representation of the time.
    pub fn as_microseconds(self) -> Time {
        match self {
            Time::Microseconds(time) => Time::Microseconds(time),
            Time::Milliseconds(time) => Time::Microseconds(time.saturating_mul(MILLISECOND_MULTIPLIER)),
            Time::Seconds(time) => Time::Microseconds(time.saturating_mul(SECOND_MULTIPLIER)),
            Time::Minutes(time) => Time::Microseconds(time.saturating_mul(MINUTE_MULTIPLIER)),
            Time::Hours(time) => Time::Microseconds(time.saturating_mul(HOUR_MULTIPLIER)),
            Time::Days(time) => Time::Microseconds(time.saturating_mul(DAY_MULTIPLIER))
        }
    }
    
    /// Returns the millisecond representation of the time.
    pub fn as_milliseconds(self) -> Time {
        match self.as_microseconds() {
            Time::Microseconds(time) => Time::Milliseconds(time / MILLISECOND_MULTIPLIER),
            _ => unreachable!()
        }
    }
}

/// Represents a timestamp within the kernel.
///
/// Currently that is roughly the number of µ-seconds since boot.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Timestamp(u64);

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}ms after boot>", self.0 / 1000)
    }
}

impl Timestamp {
    /// Returns a new time stamp with the offset of the given amount of microseconds.
    pub fn from_microseconds(time: u64) -> Timestamp {
        Timestamp(time)
    }

    /// Returns the current time stamp.
    pub fn get_current() -> Timestamp {
        get_current_timestamp()
    }

    /// Offsets the time stamp by the given amount.
    pub fn offset(&mut self, time: Time) {
        if let Time::Microseconds(microseconds) = time.as_microseconds() {
            let signed_stamp = self.0 as i64;

            assert!(signed_stamp >= 0);

            self.0 = (signed_stamp.saturating_add(microseconds)) as u64;
        } else {
            unreachable!();
        }
    }
}
