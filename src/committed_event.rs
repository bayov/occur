use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;
use impl_tools::autoimpl;

use crate::StreamDescription;

/// The sequence number of a committed event.
///
/// A committed event is assigned a commit number indicating its position within
/// the event stream.
///
/// The very first event in a stream is assigned commit number 0.
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

#[autoimpl(Clone where T::Event: Clone)]
#[autoimpl(PartialEq where T::Event: PartialEq)]
#[autoimpl(Eq where T::Event: Eq)]
pub struct CommittedEvent<T: StreamDescription> {
    pub id: T::Id,
    pub commit_number: CommitNumber,
    pub time: T::Time,
    pub event: T::Event,
}

#[allow(clippy::missing_fields_in_debug)] // false positive
impl<T: StreamDescription> Debug for CommittedEvent<T>
where
    T::Id: Debug,
    T::Time: Debug,
    T::Event: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct(&format!(r#"CommittedEvent<"{}">"#, T::NAME))
            .field_with("id", |f| write!(f, "{:?}", self.id))
            .field("commit_number", &self.commit_number.0)
            .field_with("time", |f| write!(f, "{:?}", self.time))
            .field("event", &self.event)
            .finish()
    }
}
