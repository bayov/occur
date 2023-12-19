use std::error::Error;

use crate::{CommitNumber, CommittedEvent, StreamDescription};

pub mod error;
pub mod inmem;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Store<T: StreamDescription> {
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

    async fn write(&mut self, event: &CommittedEvent<T>) -> Result<()>;

    async fn read(
        &self,
        start_commit_number: CommitNumber,
    ) -> Result<Self::EventIterator>;

    async fn subscribe(
        &self,
        start_commit_number: CommitNumber,
    ) -> Result<Self::EventSubscription>;
}

pub trait EventIterator<T: StreamDescription> {
    async fn next(&mut self) -> Option<CommittedEvent<T>>;
}

pub trait EventSubscription<T: StreamDescription> {
    async fn next(&mut self) -> Option<CommittedEvent<T>>;
    fn stop(self);
}
