use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;

/// The time that an event was recorded at.
pub type Time = std::time::SystemTime;

/// The sequence number of a recorded event.
///
/// A recorded event is assigned a sequence number indicating its position
/// within the event stream. The very first recorded event in a stream is
/// assigned the sequence number 0.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, Default,
)]
pub struct SequenceNumber(pub usize);

/// Adds a number to sequence number.
impl Add<usize> for SequenceNumber {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output { Self(self.0 + rhs) }
}

/// Adds a number to sequence number.
impl AddAssign<usize> for SequenceNumber {
    fn add_assign(&mut self, rhs: usize) { self.0 += rhs; }
}

/// Subtracts two sequence numbers, returning the difference between them.
impl Sub for SequenceNumber {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}
