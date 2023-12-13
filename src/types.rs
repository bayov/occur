use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;

/// The time that an event was recorded at.
pub type Time = std::time::SystemTime;

/// Version is the sequence number of a recorded event.
///
/// A recorded event is assigned a version indicating its position within the
/// event stream. The very first recorded event in a stream is assigned
/// version 0.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, Default,
)]
pub struct Version(pub usize);

impl Add<usize> for Version {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output { Self(self.0 + rhs) }
}

impl AddAssign<usize> for Version {
    fn add_assign(&mut self, rhs: usize) { self.0 += rhs; }
}

impl Sub for Version {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}
