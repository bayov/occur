use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;

/// Version is a sequence number of a recorded event.
///
/// A recorded event is assigned a version indicating its position within the
/// event stream. The very first recorded event in a stream is assigned
/// version 0.
#[derive(
    Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Display,
)]
pub struct Version(pub u32);

impl Add<u32> for Version {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output { Self(self.0 + rhs) }
}

impl AddAssign<u32> for Version {
    fn add_assign(&mut self, rhs: u32) { self.0 += rhs; }
}

impl Sub for Version {
    type Output = u32;
    fn sub(self, rhs: Self) -> Self::Output { self.0 - rhs.0 }
}
