use std::error::Error;

pub use commit::Commit;
pub use read::Read;
pub use stream::Stream;

use crate::Event;

pub mod commit;
pub mod error;
pub mod inmem;
pub mod read;
pub mod stream;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store<T: Event> {
    type Stream: Stream<T>;

    fn stream(&mut self, id: T::StreamId) -> Self::Stream;
}
