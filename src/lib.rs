#![feature(
    coroutines,
    coroutine_trait,
    step_trait,
    type_alias_impl_trait,
    return_position_impl_trait_in_trait,
    error_generic_member_access,
    marker_trait_attr,
    associated_type_defaults
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

pub use entity::Entity;
pub use event::{Recorded, Ref, Stream, StreamDescriptor};
pub use time::Time;
pub use version::Version;

mod entity;
mod event;
// pub mod repo;
mod time;
mod version;
