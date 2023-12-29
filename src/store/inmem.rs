use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use futures_locks::RwLock;

use crate::store::CommitNumber;
use crate::{store, Event, Streamable};

pub type Time = SystemTime;

#[derive(Clone, Debug)]
pub struct CommittedEvent<T: Event> {
    pub commit_number: CommitNumber,
    pub time: Time,
    pub event: T,
}

impl<T: Event> store::CommittedEvent for CommittedEvent<T> {
    type Event = T;
    type Time = Time;

    fn event(&self) -> &Self::Event { &self.event }
    fn commit_number(&self) -> CommitNumber { self.commit_number }
    fn time(&self) -> &Self::Time { &self.time }
}

#[derive(Default)]
pub struct Store<T: Streamable> {
    events_by_stream_id: HashMap<T::Id, Stream<T>>,
}

impl<T: Streamable> Store<T> {
    #[must_use]
    pub fn new() -> Self { Self { events_by_stream_id: HashMap::default() } }
}

impl<T: Streamable> store::Store<T> for Store<T> {
    type Stream = Stream<T>;

    fn stream(&mut self, id: T::Id) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Clone)]
pub struct Stream<T: Streamable> {
    events: SmartVec<CommittedEvent<T>>,
    sender: async_channel::Sender<CommitNumber>,
    receiver: async_channel::Receiver<CommitNumber>,
}

impl<T: Streamable> Stream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Arc::default(), sender, receiver }
    }
}

impl<T: Streamable> Default for Stream<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Streamable> store::Stream<T> for Stream<T> {
    type CommittedEvent = CommittedEvent<T>;
    type EventIterator = EventIterator<T>;
    type EventSubscription = EventSubscription<T>;

    fn id(&self) -> &T::Id { todo!() }

    async fn commit(
        &mut self,
        event: &T,
    ) -> store::Result<impl store::CommittedEvent> {
        let commit_number = self.events.read().await.len().into();
        let committed_event = CommittedEvent {
            event: event.clone(),
            commit_number,
            time: Time::now(),
        };
        self.events.write().await.push(committed_event.clone());
        self.sender.send(commit_number).await.expect("Should not fail");
        Ok(committed_event)
    }

    async fn read(
        &self,
        start_commit_number: CommitNumber,
    ) -> store::Result<Self::EventIterator> {
        Ok(EventIterator::new(Arc::clone(&self.events), start_commit_number))
    }

    async fn subscribe(
        &self,
        start_commit_number: CommitNumber,
    ) -> store::Result<Self::EventSubscription> {
        Ok(EventSubscription::new(
            Arc::clone(&self.events),
            start_commit_number,
            self.receiver.clone(),
        ))
    }
}

pub struct EventIterator<T: Event> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: CommitNumber,
}

impl<T: Event> EventIterator<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: CommitNumber,
    ) -> Self {
        Self { events, commit_number: start_commit_number }
    }
}

impl<T: Event> store::EventIterator<CommittedEvent<T>> for EventIterator<T> {
    async fn next(&mut self) -> Option<CommittedEvent<T>> {
        let events = self.events.read().await;
        let event = events.get(usize::from(self.commit_number));
        if event.is_some() {
            self.commit_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<T: Event> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: CommitNumber,
    receiver: async_channel::Receiver<CommitNumber>,
}

impl<T: Event> EventSubscription<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: CommitNumber,
        receiver: async_channel::Receiver<CommitNumber>,
    ) -> Self {
        Self { events, commit_number: start_commit_number, receiver }
    }
}

impl<T: Event> store::EventSubscription<CommittedEvent<T>>
    for EventSubscription<T>
{
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
