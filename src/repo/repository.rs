use crate::{repo, Event, Id, SequenceNumber};

pub trait Repository<T: Event> {
    type Id: Id;
    type Stream: Stream<T>;

    fn new_id(&mut self) -> Self::Id;

    fn stream(&mut self, id: Self::Id) -> Self::Stream;

    fn new_stream(&mut self) -> Self::Stream {
        let id = self.new_id();
        self.stream(id)
    }
}

pub trait Stream<T: Event> {
    type Id: Id;
    type EventIterator: EventIterator<T>;
    type EventSubscription: EventSubscription<T>;

    fn id(&self) -> &Self::Id;

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

pub trait EventIterator<T: Event> {
    async fn next(&mut self) -> Option<Timed<T>>;
}

pub trait EventSubscription<T: Event> {
    async fn next(&mut self) -> Option<Timed<T>>;
    fn stop(self);
}
