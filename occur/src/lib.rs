#![feature(
    coroutines,
    coroutine_trait,
    step_trait,
    type_alias_impl_trait,
    error_generic_member_access,
    marker_trait_attr,
    debug_closure_helpers,
    trait_alias,
    never_type
)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
// #![warn(missing_docs)] -- TODO: uncomment when ready

pub use entity::Entity;
pub use event::Event;
pub use revision::Revision;

mod entity;
mod error;
mod event;
pub mod revision;
pub mod store;
