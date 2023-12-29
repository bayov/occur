use crate::store::{CommitNumber, CommittedEvent};
use crate::Event;

pub trait Stream<T: Event> {
    type CommittedEvent: CommittedEvent<Event = T>;
    type EventIterator: Iterator<Self::CommittedEvent>;
    type EventSubscription: Subscription<Self::CommittedEvent>;

    fn id(&self) -> &T::StreamId;

    async fn commit(
        &mut self,
        event: &T,
    ) -> crate::store::Result<impl CommittedEvent>;

    async fn read(
        &self,
        start_from: CommitNumber,
    ) -> crate::store::Result<Self::EventIterator>;

    async fn subscribe(
        &self,
        start_from: CommitNumber,
    ) -> crate::store::Result<Self::EventSubscription>;
}

pub trait Iterator<T: CommittedEvent> {
    async fn next(&mut self) -> Option<T>;
}

pub trait Subscription<T: CommittedEvent> {
    async fn next(&mut self) -> Option<T>;
    fn stop(self);
}
