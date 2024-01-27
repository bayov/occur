#![feature(
    coroutines,
    coroutine_trait,
    iter_collect_into,
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
pub use error::ErrorWithKind;
pub use event::Event;
pub use revision::Revision;
pub use serializer::{Deserializer, Serializer};
pub use store::Store;

mod entity;
mod error;
mod event;
pub mod revision;
pub mod serializer;
pub mod store;
