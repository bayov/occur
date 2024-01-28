use std::future::Future;

use derive_more::Display;

use crate::error::ErrorWithKind;
use crate::{revision, Event};

/// Sequence number assigned to a committed event.
///
/// Whenever an event is committed to an event stream it is assigned a
/// sequentially increasing commit number, starting from 0.
pub type CommitNumber = u32;

/// Specifies the condition for a commit to succeed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Condition {
    /// No conditions; a commit is expected to always succeed.
    None,

    /// For the commit to succeed, the event to be committed must be assigned
    /// the provided commit number.
    ///
    /// Used for optimistic concurrency ([OCC]).
    ///
    /// When committing many events ([`WriteStream::commit_many`]), this
    /// condition ensures the first event is assigned the provided commit
    /// number.
    ///
    /// [OCC]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control
    AssignCommitNumber(CommitNumber),
}

/// Errors that might occur when committing an event to a stream.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
pub enum ErrorKind {
    /// The stream is full, and cannot accept anymore events.
    #[display("stream full")]
    StreamFull,

    /// The specified commit condition was not met.
    ///
    /// TODO:
    ///     Maybe it's better not to consider this an error, as creating an
    ///     error whenever an optimistic write fails is heavy (due to having to
    ///     potentially create a backtrace, etc).
    #[display("condition not met")]
    ConditionNotMet,

    /// An unexpected error occurred.
    ///
    /// Can be used by implementors of [`WriteStream`] to denote
    /// implementation-specific errors that do not match any other
    /// [`ErrorKind`].
    #[display("unexpected error")]
    Other,
}

/// An event stream to which events can be committed.
///
/// This is the write side of an event stream. See [`crate::store::ReadStream`]
/// for the read side.
pub trait WriteStream: Send {
    /// The type of events held within the stream.
    type Event: Event;

    /// The type of error that might occur when trying to commit an event.
    type Error: ErrorWithKind<Kind = ErrorKind>;

    /// Commits either an old or a new event revision to the stream, given the
    /// provided condition holds.
    ///
    /// On successful commit, returns the assigned commit number.
    ///
    /// Prefer using [`WriteStream::commit`] instead when committing new events.
    fn commit_old_or_new(
        &mut self,
        event: revision::OldOrNewRef<'_, Self::Event>,
        condition: Condition,
    ) -> impl Future<Output = Result<CommitNumber, Self::Error>> + Send;

    /// Commits many events to the stream, given the provided condition holds
    ///
    /// Either all events will successfully commit, or none will and an error
    /// will be returned.
    ///
    /// On successful commit, returns the commit number assigned to the first
    /// committed event. If the given `events` iterator is empty, returns
    /// `None`.
    ///
    /// For convenience, [`WriteStream::commit_many_unconditionally`] can be
    /// used in place of [`Condition::None`], and
    /// [`WriteStream::commit_many_with_number`] can be used in place of
    /// [`Condition::AssignCommitNumber`].
    fn commit_many<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a Self::Event>,
        condition: Condition,
    ) -> impl Future<Output = Result<Option<CommitNumber>, Self::Error>> + Send;

    /// Commits an event to the stream, given the provided condition holds.
    ///
    /// On successful commit, returns the assigned commit number.
    ///
    /// For convenience, [`WriteStream::commit_unconditionally`] can be used in
    /// place of [`Condition::None`], and
    /// [`WriteStream::commit_as_number`] can be used in place of
    /// [`Condition::AssignCommitNumber`].
    fn commit(
        &mut self,
        event: &Self::Event,
        condition: Condition,
    ) -> impl Future<Output = Result<CommitNumber, Self::Error>> + Send {
        self.commit_old_or_new(revision::OldOrNewRef::New(event), condition)
    }

    /// Commits an event to the stream unconditionally.
    ///
    /// On successful commit, returns the assigned commit number.
    fn commit_unconditionally(
        &mut self,
        event: &Self::Event,
    ) -> impl Future<Output = Result<CommitNumber, Self::Error>> + Send {
        self.commit(event, Condition::None)
    }

    /// Commits an event to the stream with the condition that it will be
    /// assigned the provided commit number.
    ///
    /// On successful commit, returns the assigned commit number.
    fn commit_as_number(
        &mut self,
        event: &Self::Event,
        commit_number: CommitNumber,
    ) -> impl Future<Output = Result<CommitNumber, Self::Error>> + Send {
        self.commit(event, Condition::AssignCommitNumber(commit_number))
    }

    /// Commits many events to the stream unconditionally.
    ///
    /// Convenience function for [`WriteStream::commit_many`].
    fn commit_many_unconditionally<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a Self::Event>,
    ) -> impl Future<Output = Result<Option<CommitNumber>, Self::Error>> + Send
    {
        self.commit_many(events, Condition::None)
    }

    /// Commits many events to the stream with the condition that the first
    /// committed event will be assigned the provided commit number.
    ///
    /// Convenience function for [`WriteStream::commit_many`].
    fn commit_many_with_number<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a Self::Event>,
        commit_number: CommitNumber,
    ) -> impl Future<Output = Result<Option<CommitNumber>, Self::Error>> + Send
    {
        self.commit_many(events, Condition::AssignCommitNumber(commit_number))
    }
}
