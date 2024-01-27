pub use commit::Commit;
pub use read::Read;

use crate::Event;

pub mod commit;
pub mod inmem;
pub mod read;

pub trait Store {
    type Event: Event;
    type EventStream: Commit<Event = Self::Event> + Read<Event = Self::Event>;

    fn stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::EventStream;
}
