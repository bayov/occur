use std::error::Error;

pub use committed_event::{CommitNumber, CommittedEvent};

use crate::stream_desc;

pub mod committed_event;
pub mod error;
pub mod inmem;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store<D: stream_desc::StreamDesc> {
    type Stream: Stream<D>;

    fn stream(&mut self, id: D::Id) -> Self::Stream;
}

pub trait Stream<D: stream_desc::StreamDesc> {
    type CommittedEvent: CommittedEvent<Event = D::Event>;
    type EventIterator: EventIterator<Self::CommittedEvent>;
    type EventSubscription: EventSubscription<Self::CommittedEvent>;

    fn id(&self) -> &D::Id;

    async fn commit(&mut self, event: &D::Event)
        -> Result<impl CommittedEvent>;

    async fn read(
        &self,
        start_from: CommitNumber,
    ) -> Result<Self::EventIterator>;

    async fn subscribe(
        &self,
        start_from: CommitNumber,
    ) -> Result<Self::EventSubscription>;
}

pub trait EventIterator<T: CommittedEvent> {
    async fn next(&mut self) -> Option<T>;
}

pub trait EventSubscription<T: CommittedEvent> {
    async fn next(&mut self) -> Option<T>;
    fn stop(self);
}
