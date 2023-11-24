#![feature(coroutines, coroutine_trait, step_trait, impl_trait_in_assoc_type)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod types;
pub use types::{Id, SequenceNumber, Time, TimeZone};

mod event;
pub use event::{Event, Recorded, Stream};

pub mod repo;
