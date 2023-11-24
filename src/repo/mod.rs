// pub mod error;

mod repository;
pub use repository::{EventIterator, EventSubscription, Repository};

pub mod fake;
