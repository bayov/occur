use crate::Revision;

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

pub trait CommittedEvent {
    type Event: Revision;
    type Time;

    fn event(&self) -> &Self::Event;
    fn commit_number(&self) -> Number;
    fn time(&self) -> &Self::Time;
}
