use crate::{revision, Event};

/// The sequence number of a committed event.
///
/// Whenever an event is committed to an event stream it is assigned a
/// sequentially increasing commit number, starting from 0.
pub type Number = u32;

/// Used to specify the conditions for a commit of an event to succeed.
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

pub trait Request<T: Event> {
    fn event(&self) -> revision::OldOrNewRef<'_, T>;
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

// pub trait CommittedEvent {
//     type Event: Event;
//
//     fn event(&self) -> &Self::Event;
//     fn take_event(self) -> Self::Event;
//     fn commit_number(&self) -> Number;
// }
