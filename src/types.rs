use derive_more::Display;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Sub};

/// An ID that uniquely identifies an event stream.
pub trait Id: Eq + Hash + Clone + Debug + Send + Sync {}
impl<T: Eq + Hash + Clone + Debug + Send + Sync> Id for T {}

/// The default time-zone used by recorded events.
pub type TimeZone = chrono::Utc;

/// The time that an event was recorded at.
pub type Time = chrono::DateTime<TimeZone>;

/// The sequence number of a recorded event.
///
/// A recorded event is assigned a sequence number indicating its position
/// within the event stream. The very first recorded event in a stream is
/// assigned the sequence number 0.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display)]
pub struct SequenceNumber(pub usize);

impl SequenceNumber {
    /// Returns the next sequence number (incremented by 1).
    ///
    /// ```
    /// # use event_sourcing::SequenceNumber;
    /// assert_eq!(SequenceNumber(42), SequenceNumber(41).next());
    /// ```
    #[must_use]
    pub const fn next(self) -> Self { Self(self.0 + 1) }

    /// Returns the previous version number (decremented by 1).
    ///
    /// ```
    /// # use event_sourcing::SequenceNumber;
    /// assert_eq!(SequenceNumber(41), SequenceNumber(42).prev());
    /// ```
    ///
    /// # Panics
    ///
    /// When sequence number is 0:
    ///
    /// ```should_panic
    /// # use event_sourcing::SequenceNumber;
    /// let  _ = SequenceNumber(0).prev();
    /// ```
    #[must_use]
    pub const fn prev(self) -> Self { Self(self.0 - 1) }
}

/// Adds a number to sequence number.
impl Add<usize> for SequenceNumber {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output { Self(self.0 + rhs) }
}

/// Subtracts two sequence numbers, returning the difference between them.
impl Sub for SequenceNumber {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}
