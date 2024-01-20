use std::error::Error;

pub use commit::Commit;
pub use read::Read;

use crate::Event;

pub mod commit;
pub mod error;
pub mod inmem;
pub mod read;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store {
    type Event: Event;
    type EventStream: Commit<Event = Self::Event> + Read<Event = Self::Event>;

    fn stream(
        &mut self,
        id: <Self::Event as Event>::StreamId,
    ) -> Self::EventStream;
}
