#![feature(
    coroutines,
    coroutine_trait,
    step_trait,
    type_alias_impl_trait,
    return_position_impl_trait_in_trait,
    error_generic_member_access,
    marker_trait_attr,
    associated_type_defaults,
    debug_closure_helpers,
    core_intrinsics,
    trait_alias
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

pub use committed_event::{CommitNumber, CommittedEvent};
pub use entity::Entity;
pub use event::Event;
pub use ref_::Ref;
pub use revision::Revision;
pub use stream::{Stream, StreamDescription};
pub use time::Time;

mod committed_event;
mod entity;
mod event;
mod ref_;
pub mod revision;
pub mod store;
mod stream;
mod time;
