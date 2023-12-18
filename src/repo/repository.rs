use crate::{repo, Event, StreamDescription};

pub trait Repository<T: StreamDescription> {
    type Stream: Stream<T>;

    fn new_id(&mut self) -> T::Id;

    fn stream(&mut self, id: T::Id) -> Self::Stream;

    fn new_stream(&mut self) -> Self::Stream {
        let id = self.new_id();
        self.stream(id)
    }
}

pub trait Stream<T: StreamDescription> {
    type EventIterator: EventIterator<T>;
    type EventSubscription: EventSubscription<T>;

    fn id(&self) -> &T::Id;

    async fn write(
        &mut self,
        sequence_number: SequenceNumber,
        event: &Timed<T>,
    ) -> repo::Result<()>;

    async fn read(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventIterator>;

    async fn subscribe(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventSubscription>;
}

pub trait EventIterator<T: StreamDescription> {
    async fn next(&mut self) -> Option<Timed<T>>;
}

pub trait EventSubscription<T: StreamDescription> {
    async fn next(&mut self) -> Option<Timed<T>>;
    fn stop(self);
}
