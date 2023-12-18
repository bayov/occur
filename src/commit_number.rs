use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;

/// CommitNumber is the sequence number of a recorded event.
///
/// A recorded event is assigned a commit number indicating its position within
/// the event stream. The very first recorded event in a stream is assigned
/// commit number 0.
#[derive(
    Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Display,
)]
pub struct CommitNumber(pub u32);

impl Add<u32> for CommitNumber {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output { Self(self.0 + rhs) }
}

impl AddAssign<u32> for CommitNumber {
    fn add_assign(&mut self, rhs: u32) { self.0 += rhs; }
}

impl Sub for CommitNumber {
    type Output = u32;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}
