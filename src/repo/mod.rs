pub use error::Result;
pub use repository::{EventIterator, EventSubscription, Repository, Stream};

pub mod error;
pub mod fake;
mod repository;
