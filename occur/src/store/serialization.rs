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

pub struct Serialization<S, D>
where
    S: Serializer,
    D: Deserializer<Event = S::Event, SerializedEvent = S::SerializedEvent>,
{
    pub serializer: S,
    pub deserializer: D,
}
