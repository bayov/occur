pub use error::Result;
pub use repository::{EventIterator, EventSubscription, Repository, Stream};

pub mod error;

mod repository;

pub mod fake;
