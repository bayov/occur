use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub};

use derive_more::Display;

/// The sequence number of a committed event.
///
/// A committed event is assigned a commit number indicating its position within
/// the event stream.
///
/// The very first event in a stream is assigned commit number 0.
#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Display,
)]
pub struct CommitNumber(pub u32);

impl From<usize> for CommitNumber {
    fn from(value: usize) -> Self {
        Self(u32::try_from(value).expect("commit number overflow"))
    }
}

impl From<CommitNumber> for usize {
    fn from(commit_number: CommitNumber) -> Self {
        Self::try_from(commit_number.0).expect("commit number overflow")
    }
}

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

pub trait CommittedEvent {
    type Event;
    type Time;

    fn event(&self) -> &Self::Event;
    fn commit_number(&self) -> CommitNumber;
    fn time(&self) -> &Self::Time;
}

// #[allow(clippy::missing_fields_in_debug)] // false positive
// impl<D: stream::Description> Debug for CommittedEvent<D>
// where
//     D::Id: Debug,
//     D::Time: Debug,
//     D::Event: Debug,
// {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         f.debug_struct(&format!(r#"CommittedEvent<"{}">"#, D::NAME))
//             .field_with("id", |f| write!(f, "{:?}", self.id))
//             .field("commit_number", &self.commit_number.0)
//             .field_with("time", |f| write!(f, "{:?}", self.time))
//             .field("event", &self.event)
//             .finish()
//     }
// }
