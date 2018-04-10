//! Handles time related functionality.

use arch::get_current_timestamp;
use core::fmt;

/// Represents a unit of time.
pub enum Time {
    /// A microsecond (µ-second) is 1/1,000,000 of a second.
    Microseconds(i64),
    /// A millisecond is 1/1,000 of a second.
    Milliseconds(i64)
}

/// The number of µ-seconds in a millisecond.
const MILLISECOND_MULTIPLIER: i64 = 1000;

impl Time {
    /// Returns the µ-second representation of the time.
    pub fn as_microseconds(self) -> Time {
        match self {
            Time::Microseconds(time) => Time::Microseconds(time),
            Time::Milliseconds(time) => {
                Time::Microseconds(time.saturating_mul(MILLISECOND_MULTIPLIER))
            },
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
    /// Returns a new time stamp with the offset of the given amount of
    /// microseconds.
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
