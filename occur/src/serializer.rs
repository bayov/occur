use std::marker::PhantomData;

use crate::{revision, Event};

pub trait Serializer: Clone + Send + Sync {
    type Event: Event;
    type SerializedEvent;

    fn serialize(
        &self,
        event: revision::OldOrNewRef<Self::Event>,
    ) -> Self::SerializedEvent;
}

pub trait Deserializer: Clone + Send + Sync {
    type Event: Event;
    type SerializedEvent: Send + Sync;

    fn deserialize(
        &self,
        event: Self::SerializedEvent,
    ) -> revision::OldOrNew<Self::Event>;
}

#[derive(Clone)]
pub struct Noop<T: Event>(PhantomData<T>);

impl<T: Event> Noop<T> {
    #[must_use]
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<T: Event> Serializer for Noop<T> {
    type Event = T;
    type SerializedEvent = revision::OldOrNew<T>;

    fn serialize(
        &self,
        event: revision::OldOrNewRef<Self::Event>,
    ) -> Self::SerializedEvent {
        event.to_owned()
    }
}

impl<T: Event> Deserializer for Noop<T> {
    type Event = T;
    type SerializedEvent = revision::OldOrNew<T>;

    fn deserialize(
        &self,
        event: Self::SerializedEvent,
    ) -> revision::OldOrNew<Self::Event> {
        event
    }
}
