#![feature(coroutines, coroutine_trait, step_trait, type_alias_impl_trait)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(async_fn_in_trait)]

mod types;
pub use types::{Id, SequenceNumber, Time, TimeZone};

mod event;
pub use event::{Event, Recorded, Stream};

pub mod repo;
