use std::marker::PhantomData;

use crate::store::serialization::Serialization;
use crate::store::{Deserializer, Serializer};
use crate::{revision, Event};

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub const fn no_serialization<T: Event>(
) -> Serialization<NoSerializer<T>, NoSerializer<T>> {
    Serialization {
        serializer: NoSerializer::new(),
        deserializer: NoSerializer::new(),
    }
}

#[derive(Clone)]
pub struct NoSerializer<T: Event>(PhantomData<T>);

impl<T: Event> NoSerializer<T> {
    #[must_use]
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<T: Event> Serializer for NoSerializer<T> {
    type Event = T;
    type SerializedEvent = revision::OldOrNew<T>;

    fn serialize(
        &self,
        event: revision::OldOrNewRef<Self::Event>,
    ) -> Self::SerializedEvent {
        event.to_owned()
    }
}

impl<T: Event> Deserializer for NoSerializer<T> {
    type Event = T;
    type SerializedEvent = revision::OldOrNew<T>;

    fn deserialize(
        &self,
        event: Self::SerializedEvent,
    ) -> revision::OldOrNew<Self::Event> {
        event
    }
}
