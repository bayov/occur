use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

use futures_locks::RwLock;
use impl_tools::autoimpl;

use crate::{repo, CommitNumber, CommittedEvent, StreamDescription};

pub type Id = u32;

#[autoimpl(Default)]
pub struct Repository<T: StreamDescription<Id = Id>> {
    next_id: Cell<Id>,
    events_by_stream_id: HashMap<Id, Stream<T>>,
}

impl<T: StreamDescription<Id = Id>> Repository<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: Cell::new(0), events_by_stream_id: HashMap::default() }
    }
}

impl<T: StreamDescription<Id = Id>> repo::Repository<T> for Repository<T> {
    type Stream = Stream<T>;

    fn new_id(&mut self) -> Id {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    fn stream(&mut self, id: T::Id) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[autoimpl(Clone)]
pub struct Stream<T: StreamDescription> {
    events: SmartVec<CommittedEvent<T>>,
    sender: async_channel::Sender<CommitNumber>,
    receiver: async_channel::Receiver<CommitNumber>,
}

impl<T: StreamDescription> Stream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Arc::default(), sender, receiver }
    }
}

impl<T: StreamDescription> Default for Stream<T> {
    fn default() -> Self { Self::new() }
}

impl<T: StreamDescription> repo::Stream<T> for Stream<T> {
    type EventIterator = EventIterator<T>;
    type EventSubscription = EventSubscription<T>;

    fn id(&self) -> &T::Id { todo!() }

    async fn write(&mut self, event: &CommittedEvent<T>) -> repo::Result<()> {
        let want_commit_number = self.events.read().await.len().into();
        assert_eq!(event.commit_number, want_commit_number);
        self.events.write().await.push((*event).clone());
        self.sender.send(event.commit_number).await.expect("Should not fail");
        Ok(())
    }

    async fn read(
        &self,
        start_commit_number: CommitNumber,
    ) -> repo::Result<Self::EventIterator> {
        Ok(EventIterator::new(Arc::clone(&self.events), start_commit_number))
    }

    async fn subscribe(
        &self,
        start_commit_number: CommitNumber,
    ) -> repo::Result<Self::EventSubscription> {
        Ok(EventSubscription::new(
            Arc::clone(&self.events),
            start_commit_number,
            self.receiver.clone(),
        ))
    }
}

pub struct EventIterator<T: StreamDescription> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: CommitNumber,
}

impl<T: StreamDescription> EventIterator<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: CommitNumber,
    ) -> Self {
        Self { events, commit_number: start_commit_number }
    }
}

impl<T: StreamDescription> repo::EventIterator<T> for EventIterator<T> {
    async fn next(&mut self) -> Option<CommittedEvent<T>> {
        let events = self.events.read().await;
        let event = events.get(usize::from(self.commit_number));
        if event.is_some() {
            self.commit_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<T: StreamDescription> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: CommitNumber,
    receiver: async_channel::Receiver<CommitNumber>,
}

impl<T: StreamDescription> EventSubscription<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: CommitNumber,
        receiver: async_channel::Receiver<CommitNumber>,
    ) -> Self {
        Self { events, commit_number: start_commit_number, receiver }
    }
}

impl<T: StreamDescription> repo::EventSubscription<T> for EventSubscription<T> {
    async fn next(&mut self) -> Option<CommittedEvent<T>> {
        {
            let events = self.events.read().await;
            let event = events.get(usize::from(self.commit_number));
            if let Some(event) = event {
                self.commit_number += 1;
                return Some((*event).clone());
            }
        }
        while let Ok(commit_number) = self.receiver.recv().await {
            if commit_number >= self.commit_number {
                let events = self.events.read().await;
                let event =
                    events.get(usize::from(self.commit_number)).unwrap();
                self.commit_number += 1;
                return Some((*event).clone());
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
