pub use commit::Commit;
pub use read::Read;

use crate::Event;

pub mod commit;
pub mod inmem;
pub mod read;

pub trait Store {
    type Event: Event;
    type WriteStream: Commit<Event = Self::Event>;
    type ReadStream: Read<Event = Self::Event>;

    fn write_stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::WriteStream;

    fn read_stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::ReadStream;
}
