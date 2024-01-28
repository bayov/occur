pub use read::ReadStream;
pub use serialization::{Deserializer, Serializer};
pub use write::{CommitNumber, WriteStream};

use crate::Event;

pub mod inmem;
pub mod read;
pub mod serialization;
pub mod write;

/// An event store for events of a specific types.
///
/// An event store is the persistence layer of event streams. Each event stream
/// within the store is identifiable by a unique value of stream ID
/// ([`Event::StreamId`]).
///
/// Note that a stream doesn't have to exist before writing or reading from it.
/// A non-existent stream is equivalent to an empty stream.
pub trait Store {
    /// The type of events held within the store.
    type Event: Event;

    /// The type that is used as the write side of an event stream.
    type WriteStream: WriteStream<Event = Self::Event>;

    /// The type that is used as the read side of an event stream.
    type ReadStream: ReadStream<Event = Self::Event>;

    /// Returns a write stream for the given stream ID.
    fn write_stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::WriteStream;

    /// Returns a read stream for the given stream ID.
    fn read_stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::ReadStream;
}
