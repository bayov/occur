// pub mod error;

mod repository;
pub use repository::{EventIterator, EventSubscription, Repository, Stream};

pub mod fake;
