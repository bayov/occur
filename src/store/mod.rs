pub use error::Result;

use crate::{CommitNumber, CommittedEvent, StreamDescription};

pub mod error;
pub mod inmem;

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

    async fn write(
        &mut self,
        event: &CommittedEvent<T>,
    ) -> crate::store::Result<()>;

    async fn read(
        &self,
        start_commit_number: CommitNumber,
    ) -> crate::store::Result<Self::EventIterator>;

    async fn subscribe(
        &self,
        start_commit_number: CommitNumber,
    ) -> crate::store::Result<Self::EventSubscription>;
}

pub trait EventIterator<T: StreamDescription> {
    async fn next(&mut self) -> Option<CommittedEvent<T>>;
}

pub trait EventSubscription<T: StreamDescription> {
    async fn next(&mut self) -> Option<CommittedEvent<T>>;
    fn stop(self);
}
