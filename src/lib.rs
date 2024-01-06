#![feature(
    coroutines,
    coroutine_trait,
    step_trait,
    type_alias_impl_trait,
    error_generic_member_access,
    marker_trait_attr,
    associated_type_defaults,
    debug_closure_helpers,
    trait_alias,
    never_type
)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
// #![warn(missing_docs)] -- TODO: uncomment when ready

pub use entity::Entity;
pub use event::Event;
pub use revision::Revision;

mod entity;
mod event;
pub mod revision;
pub mod store;
