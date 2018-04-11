//! Handles time related functionality.

use arch::get_current_timestamp;
use core::fmt;
use core::time::Duration;

/// Represents a timestamp within the kernel.
///
/// Currently that is the `Duration` since boot.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Timestamp(Duration);

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>6}.{:06}", self.0.as_secs(), self.0.subsec_micros())
    }
}

impl Timestamp {
    /// Returns a new time stamp with the offset of the given amount of
    /// milliseconds.
    pub fn from_milliseconds(time: u64) -> Timestamp {
        Timestamp(Duration::from_millis(time))
    }

    /// Returns the current time stamp.
    pub fn get_current() -> Timestamp {
        get_current_timestamp()
    }

    /// Offsets the time stamp by the given amount.
    pub fn offset(self, duration: Duration) -> Option<Timestamp> {
        self.0
            .checked_add(duration)
            .map(|new_duration| Timestamp(new_duration))
    }
}
