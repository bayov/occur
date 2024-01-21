use std::future::Future;

use futures::{Stream, StreamExt};

use crate::store::{commit, Result};
use crate::{revision, Event};

/// Represents a position within an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Position {
    /// Represents the position of the first event.
    Start,
    /// Represents the position of the last event.
    End,
    /// Represents the position at the given commit number.
    Commit(commit::Number),
}

/// The order in which events should be read from an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    /// Events should be read from first to last.
    Forward,
    /// Events should be read from last to first. This implies reverse order.
    Backward,
}

/// Options for reading events from an event stream.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Options {
    /// The position of the first event to read.
    pub position: Position,
    /// The order in which events should be read.
    pub direction: Direction,
    /// The maximum number of events to read. If None, all events should be
    /// read.
    pub limit: Option<usize>,
}

/// Represents an event stream from which events can be read.
///
/// This is the read side of an event stream. See [`crate::store::Commit`] for
/// the write side.
pub trait Read: Send {
    /// The type of events held within the stream.
    type Event: Event;

    /// Read events from the stream without converting them to their newest
    /// revision.
    ///
    /// Use [`Self::read`] to convert the read events (using
    /// [`revision::OldOrNew::to_new`]).
    fn read_unconverted(
        &self,
        options: Options,
    ) -> impl Future<
        Output = Result<impl Stream<Item = revision::OldOrNew<Self::Event>>>,
    > + Send;

    /// Read events from the stream based on the provided options.
    fn read(
        &self,
        options: Options,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        let future = self.read_unconverted(options);
        async { future.await.map(|it| it.map(revision::OldOrNew::to_new)) }
    }

    /// Read all events from the stream, from first to last.
    fn read_all(
        &self,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        self.read(Options {
            position: Position::Start,
            direction: Direction::Forward,
            limit: None,
        })
    }
}
