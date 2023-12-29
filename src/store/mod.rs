use std::error::Error;

pub use committed_event::{CommitNumber, CommittedEvent};

use crate::Streamable;

pub mod committed_event;
pub mod error;
pub mod inmem;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store<T: Streamable> {
    type Stream: Stream<T>;

    fn stream(&mut self, id: T::Id) -> Self::Stream;
}

pub trait Stream<T: Streamable> {
    type CommittedEvent: CommittedEvent<Event = T>;
    type EventIterator: EventIterator<Self::CommittedEvent>;
    type EventSubscription: EventSubscription<Self::CommittedEvent>;

    fn id(&self) -> &T::Id;

    async fn commit(&mut self, event: &T) -> Result<impl CommittedEvent>;

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
