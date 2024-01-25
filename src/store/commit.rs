use std::future::Future;

use derive_more::Display;

use crate::error::ErrorWithKind;
use crate::{revision, Event};

/// Sequence number of a committed event.
///
/// Whenever an event is committed to an event stream it is assigned a
/// sequentially increasing commit number, starting from 0.
pub type Number = u32;

/// Specifies the condition for a commit request to succeed.
pub enum Condition {
    /// No conditions; a commit request is expected to always succeed.
    None,

    /// The commit request must result in the committed event number being
    /// equal to the one provided.
    ///
    /// Used for committing events optimistically ([OCC]).
    ///
    /// When committing many events at once, the commit number of the first
    /// event is used as the commit number.
    ///
    /// [OCC]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control
    WantCommitNumber(Number),
}

/// A request to commit an event to a stream.
pub trait Request<T: Event> {
    /// The event to be committed.
    fn event(&self) -> revision::OldOrNewRef<'_, T>;

    /// The condition that must be satisfied to successfully commit.
    fn condition(&self) -> Condition;
}

impl<T: Event> Request<T> for &T {
    fn event(&self) -> revision::OldOrNewRef<'_, T> {
        revision::OldOrNewRef::New(self)
    }
    fn condition(&self) -> Condition { Condition::None }
}

impl<T: Event> Request<T> for &revision::OldOrNew<T> {
    fn event(&self) -> revision::OldOrNewRef<'_, T> { self.borrow() }
    fn condition(&self) -> Condition { Condition::None }
}

/// Errors that might occur when committing an event to a stream.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
pub enum ErrorKind {
    /// The stream is full, and cannot accept anymore events.
    #[display("stream full")]
    StreamFull,
    /// The condition specified by the request was not met.
    #[display("condition not met")]
    ConditionNotMet,
    /// An unexpected error occurred.
    #[display("unexpected error")]
    Other,
}

/// Represents an event stream to which events can be committed.
///
/// This is the write side of an event stream. See [`crate::store::Read`] for
/// the read side.
pub trait Commit: Send {
    /// The type of events held within the stream.
    type Event: Event;
    type Error: ErrorWithKind<Kind = ErrorKind>;

    /// Commits an event to the stream.
    ///
    /// Returns the commit number of the event.
    fn commit(
        &mut self,
        request: impl Request<Self::Event>,
    ) -> impl Future<Output = Result<Number, Self::Error>> + Send;
}
