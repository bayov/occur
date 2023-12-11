pub mod event;

pub type Id = u32;
pub type Ref<T> = event_sourcing::Ref<T, Id>;
