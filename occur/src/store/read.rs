use std::future::Future;

use derive_more::Display;
use futures::{Stream, StreamExt};

use crate::error::ErrorWithKind;
use crate::store::CommitNumber;
use crate::{revision, Event};

/// Position of an event within an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Position {
    /// Represents the position of the first event.
    First,
    /// Represents the position of the last event.
    Last,
    /// Represents the position of the event at the given commit number.
    CommitNumber(CommitNumber),
}

/// The order in which events should be read from an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    /// Events should be read from oldest to newest.
    Forward,
    /// Events should be read from newest to oldest.
    Backward,
}

/// Options for reading events from an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Options {
    /// The position of the first event to read.
    pub position: Position,

    /// The direction in which events should be read.
    ///
    /// When direction is [`Direction::Forward`], events will be read until the
    /// last event in the stream is reached, from old to new:
    /// ```text
    /// [position..=last_committed_event]
    /// ```
    ///
    /// When direction is [`Direction::Forward`], events will be read until the
    /// first event in the stream is reached, from new to old:
    /// ```text
    /// [position..=first_committed_event]
    /// ```
    pub direction: Direction,

    /// The maximum number of events to read. If `None`, events will be read
    /// until the first or last event is reached, depending on the read
    /// `direction`.
    pub limit: Option<usize>,
}

/// Errors that might occur when reading events from a stream.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
pub enum ErrorKind {
    /// The specified [`Position::CommitNumber`] number was not found within
    /// the stream.
    #[display("commit not found")]
    CommitNotFound,

    /// An unexpected error occurred.
    ///
    /// Can be used by implementors of [`ReadStream`] to denote
    /// implementation-specific errors that do not match any other
    /// [`ErrorKind`].
    #[display("unexpected error")]
    Other,
}

/// An event stream from which events can be read.
///
/// This is the read side of an event stream. See [`crate::store::WriteStream`]
/// for the write side.
pub trait ReadStream: Send {
    /// The type of events held within the stream.
    type Event: Event;

    /// The type of error that might occur when trying to commit an event.
    type Error: ErrorWithKind<Kind = ErrorKind>;

    /// Read events from the stream without converting them to their newest
    /// revision.
    ///
    /// Use [`Self::read`] to automatically convert the read events (using
    /// [`revision::OldOrNew::to_new`]).
    fn read_unconverted(
        &mut self,
        options: Options,
    ) -> impl Future<
        Output = Result<
            impl Stream<Item = revision::OldOrNew<Self::Event>>,
            Self::Error,
        >,
    > + Send;

    #[rustfmt::skip]
    /// Read events from the stream based on the provided options.
    fn read(
        &mut self,
        options: Options,
    ) -> impl Future<
        Output=Result<
            impl Stream<Item=Self::Event>,
            Self::Error,
        >
    > + Send {
        let future = self.read_unconverted(options);
        async { future.await.map(|it| it.map(revision::OldOrNew::to_new)) }
    }

    #[rustfmt::skip]
    /// Read all events from the stream, from first to last.
    fn read_all(
        &mut self,
    ) -> impl Future<
        Output=Result<
            impl Stream<Item=Self::Event>,
            Self::Error,
        >
    > + Send {
        self.read(Options {
            position: Position::First,
            direction: Direction::Forward,
            limit: None,
        })
    }
}
