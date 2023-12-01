#![feature(
    coroutines,
    coroutine_trait,
    step_trait,
    type_alias_impl_trait,
    return_position_impl_trait_in_trait,
    error_generic_member_access
)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(async_fn_in_trait)]
// RustRover IDE doesn't realize that the return_position_impl_trait_in_trait
// feature is stable, and marks usages as Errors:
// https://youtrack.jetbrains.com/issue/RUST-10216
//
// When RustRover is patched, remove this line and remove
// feature(return_position_impl_trait_in_trait).
#![allow(stable_features)]

pub use event::{Event, Recorded, Stream};
pub use types::{Id, SequenceNumber, Time, TimeZone};

mod event;
pub mod repo;
mod types;
