use std::error::Error;

pub use committed_event::{CommitNumber, CommittedEvent};
pub use stream::Stream;

use crate::Event;

mod committed_event;
pub mod error;
pub mod inmem;
pub mod stream;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store<T: Event> {
    type Stream: Stream<T>;

    fn stream(&mut self, id: T::StreamId) -> Self::Stream;
}
