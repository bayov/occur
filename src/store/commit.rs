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
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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

impl Condition {
    pub const fn with_event<T: Event>(
        self,
        event: &T,
    ) -> RequestWithCondition<T> {
        RequestWithCondition {
            event: revision::OldOrNewRef::New(event),
            condition: self,
        }
    }

    pub const fn with_old_event<T: Event>(
        self,
        event: &T::OldRevision,
    ) -> RequestWithCondition<T> {
        RequestWithCondition {
            event: revision::OldOrNewRef::Old(event),
            condition: self,
        }
    }
}

/// A request to commit an event to a stream.
///
/// Implemented for [`Event`], allowing events to be committed with no
/// conditions.
///
/// Implemented for [`revision::OldOrNew`], allowing old event revisions to be
/// committed with no conditions.
///
/// To include a commit condition, use
///
/// Example:
/// ```
/// # use std::collections::HashSet;
/// # use occur::store::{Commit as _, Store as _};
/// use occur::revision;
///
/// #
/// # #[derive(Clone, Eq, PartialEq, Hash, Debug)]
/// # enum SomeEvent { Foo, Bar }
/// #
/// # impl occur::Event for SomeEvent {
/// #     type StreamId = i64;
/// #     type OldRevision = SomeEventOld;
/// # }
/// #
/// # impl occur::Revision for SomeEvent {
/// #     type Value = (&'static str, u8);
/// #     fn revision(&self) -> Self::Value {
/// #         match self {
/// #             Self::Foo => ("Foo", 0),
/// #             Self::Bar => ("Bar", 1),
/// #         }
/// #     }
/// #     fn revision_set() -> HashSet<Self::Value> {
/// #             HashSet::from([("Foo", 0), ("Bar", 1)])
/// #     }
/// # }
/// #
/// # #[allow(non_camel_case_types)]
/// # #[derive(Clone, Eq, PartialEq, Hash, Debug)]
/// # enum SomeEventOld { Bar_V0 }
/// #
/// # impl occur::Revision for SomeEventOld {
/// #     type Value = (&'static str, u8);
/// #     fn revision(&self) -> Self::Value {
/// #         match self {
/// #             Self::Bar_V0 => ("Bar", 0),
/// #         }
/// #     }
/// #     fn revision_set() -> HashSet<Self::Value> {
/// #             HashSet::from([("Bar", 0)])
/// #     }
/// # }
/// #
/// # impl revision::Convert for SomeEventOld {
/// #     type Event = SomeEvent;
/// #     fn convert(self) -> revision::OldOrNew<Self::Event> {
/// #        match self {
/// #            Self::Bar_V0 => SomeEvent::Bar.into(),
/// #        }
/// #    }
/// # }
/// #
/// # let mut store = occur::store::inmem::Store::<SomeEvent>::new();
/// # let _ =
/// store.stream(42).commit(&SomeEvent::Foo);
///
/// // If you need to commit an old event revision for some reason, you can
/// // wrap it in revision::OldOrNew first.
/// let old_event = SomeEventOld::Bar_V0;
/// # let _ =
/// store.stream(42).commit(&revision::OldOrNew::Old(old_event));
/// ```
///
/// and [`revision::OldOrNew`].
pub trait Request<T: Event> {
    /// The event to be committed.
    fn event(&self) -> revision::OldOrNewRef<'_, T>;

    /// The condition that must be satisfied to successfully commit.
    fn condition(&self) -> Condition;
}

impl<T: Event> Request<T> for T {
    fn event(&self) -> revision::OldOrNewRef<'_, T> {
        revision::OldOrNewRef::New(self)
    }
    fn condition(&self) -> Condition { Condition::None }
}

impl<T: Event> Request<T> for revision::OldOrNew<T> {
    fn event(&self) -> revision::OldOrNewRef<'_, T> { self.borrow() }
    fn condition(&self) -> Condition { Condition::None }
}

/// A request to commit an event to a stream with a condition.
///
/// Use [`Condition::with_event`] to create a commit request with a condition.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RequestWithCondition<'a, T: Event> {
    event: revision::OldOrNewRef<'a, T>,
    condition: Condition,
}

impl<T: Event> Request<T> for RequestWithCondition<'_, T> {
    fn event(&self) -> revision::OldOrNewRef<'_, T> { self.event.clone() }
    fn condition(&self) -> Condition { self.condition }
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
    ///
    /// Can be used by implementors of [`Commit`] to denote
    /// implementation-specific errors that do not match any other
    /// [`ErrorKind`].
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
        request: &dyn Request<Self::Event>,
    ) -> impl Future<Output = Result<Number, Self::Error>> + Send;
}
