//! Handles time related functionality.

use arch::{self, Architecture};
use core::fmt;
use core::ops;
use core::time::Duration;

/// Represents a timestamp within the kernel.
///
/// Currently that is the `Duration` since boot.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Timestamp(Duration);

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:06}", self.0.as_secs(), self.0.subsec_micros())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:>6}.{:06}]", self.0.as_secs(), self.0.subsec_micros())
    }
}

impl ops::Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Timestamp {
    /// Returns a new `Timestamp` corresponding to the `duration` since boot.
    pub fn from_duration(duration: Duration) -> Timestamp {
        Timestamp(duration)
    }

    /// Returns the current time stamp.
    pub fn get_current() -> Timestamp {
        arch::Current::get_current_timestamp()
    }

    /// Offsets the time stamp by the given amount.
    pub fn offset(self, duration: Duration) -> Option<Timestamp> {
        self.0
            .checked_add(duration)
            .map(|new_duration| Timestamp(new_duration))
    }

    /// Tries to perform a subtraction and returns the result if successful.
    pub fn checked_sub(self, other: Timestamp) -> Option<Duration> {
        self.0.checked_sub(other.0)
    }
}
