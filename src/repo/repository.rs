use crate::{Event, Id, Recorded, SequenceNumber};
use std::error::Error;
use std::future::Future;

pub trait Repository<T: Event> {
    type Id: Id;

    type Stream<'repo>: Stream<'repo, T>
    where
        Self: 'repo;

    fn new_id(&mut self) -> Self::Id;

    fn stream(&mut self, id: Self::Id) -> Self::Stream<'_>;

    fn new_stream(&mut self) -> Self::Stream<'_> {
        let id = self.new_id();
        self.stream(id)
    }
}

pub trait Stream<'repo, T: Event> {
    type Id: Id;
    type EventIterator: EventIterator<'repo, T>;
    // type EventIterator: futures::Stream<Item = Recorded<T>> + 'repo;
    // type EventSubscription: EventSubscription<'repo, T>;

    fn id(&self) -> &Self::Id;

    fn write(
        &mut self,
        sequence_number: SequenceNumber,
        event: &Recorded<T>,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send + Sync;

    fn read(
        &'repo self,
        start_sequence_number: SequenceNumber,
    ) -> impl Future<Output = Result<Self::EventIterator, Box<dyn Error>>>
           + Send
           + Sync;

    // async fn read(
    //     &self,
    //     start_sequence_number: SequenceNumber,
    // ) -> Result<Self::EventIterator, Box<dyn Error>>;

    // async fn subscribe(
    //     &self,
    //     start_sequence_number: SequenceNumber,
    // ) -> Result<Self::EventSubscription, Box<dyn Error>>;
}

pub trait EventIterator<'repo, T: Event> {
    fn next(
        &mut self,
    ) -> impl Future<Output = Option<Recorded<T>>> + Send + Sync;
}

pub trait EventSubscription<'repo, T: Event>:
    futures::Stream<Item = Recorded<T>>
{
    fn stop(self);
}
