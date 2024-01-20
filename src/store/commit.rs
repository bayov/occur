use std::future::Future;

use crate::{revision, store, Event};

/// Sequence number of a committed event.
///
/// Whenever an event is committed to an event stream it is assigned a
/// sequentially increasing commit number, starting from 0.
pub type Number = u32;

/// Specifies the condition for a commit request to succeed.
pub enum Condition {
    /// There are no conditions, events should be committed regardless of the
    /// stream's state.
    None,

    /// The stream's latest commit number should be equal to the number
    /// provided.
    ///
    /// Used for committing events optimistically ([OCC]).
    ///
    /// [OCC]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control
    Number(Number),
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

/// Represents an event stream to which events can be committed.
///
/// This is the write side of an event stream. See [`store::Read`] for the
/// read side.
pub trait Commit: Send {
    type Event: Event;

    /// Commits an event to the stream.
    ///
    /// Returns the commit number of the event.
    fn commit(
        &mut self,
        request: impl Request<Self::Event>,
    ) -> impl Future<Output = store::Result<Number>> + Send;
}
