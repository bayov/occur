use crate::{Event, Id, Recorded, SequenceNumber};
use std::error::Error;

pub trait Repository<T: Event> {
    type StreamId: Id;
    type EventIterator<'s>: EventIterator<'s, T>
    where
        Self: 's;
    type EventSubscription<'s>: EventSubscription<'s, T>
    where
        Self: 's;

    fn new_id(&mut self) -> Self::StreamId;

    async fn write_event(
        &mut self,
        stream_id: Self::StreamId,
        sequence_number: SequenceNumber,
        event: &Recorded<T>,
    ) -> Result<(), Box<dyn Error>>;

    async fn read_stream(
        &self,
        stream_id: &Self::StreamId,
        start_sequence_number: SequenceNumber,
    ) -> Result<Self::EventIterator<'_>, Box<dyn Error>>;

    async fn subscribe_to_stream(
        &self,
        stream_id: &Self::StreamId,
        start_sequence_number: SequenceNumber,
    ) -> Result<Self::EventSubscription<'_>, Box<dyn Error>>;
}

pub trait EventIterator<'s, T: Event> {
    async fn next(&mut self) -> Option<Recorded<T>>;
}

pub trait EventSubscription<'s, T: Event> {
    async fn next(&mut self) -> Option<Recorded<T>>;
    fn stop(self);
}
