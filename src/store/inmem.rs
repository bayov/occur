use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use futures_locks::RwLock;

use crate::store::{commit, stream};
use crate::{store, Event, Revision};

pub type Time = SystemTime;

#[derive(Clone, Debug)]
pub struct CommittedEvent<T: Revision> {
    pub commit_number: commit::Number,
    pub time: Time,
    pub event: T,
}

impl<T: Revision> store::CommittedEvent for CommittedEvent<T> {
    type Event = T;
    type Time = Time;

    fn event(&self) -> &Self::Event { &self.event }
    fn commit_number(&self) -> commit::Number { self.commit_number }
    fn time(&self) -> &Self::Time { &self.time }
}

#[derive(Default)]
pub struct Store<T: Event> {
    events_by_stream_id: HashMap<T::StreamId, Stream<T>>,
}

impl<T: Event> Store<T> {
    #[must_use]
    pub fn new() -> Self { Self { events_by_stream_id: HashMap::default() } }
}

impl<T: Event> store::Store<T> for Store<T> {
    type Stream = Stream<T>;

    fn stream(&mut self, id: T::StreamId) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Clone)]
pub struct Stream<T: Event> {
    events: SmartVec<CommittedEvent<T>>,
    sender: async_channel::Sender<commit::Number>,
    receiver: async_channel::Receiver<commit::Number>,
}

impl<T: Event> Stream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Arc::default(), sender, receiver }
    }
}

impl<T: Event> Default for Stream<T> {
    fn default() -> Self { Self::new() }
}

#[allow(clippy::future_not_send)]
impl<T: Event> store::Stream<T> for Stream<T> {
    type CommittedEvent = CommittedEvent<T>;
    type EventIterator = EventIterator<T>;
    type EventSubscription = EventSubscription<T>;

    fn id(&self) -> &T::StreamId { todo!() }

    async fn commit(
        &mut self,
        event: &T,
        condition: commit::Condition,
    ) -> store::Result<Self::CommittedEvent> {
        let mut c = CommittedEvent {
            // commit number is modified below before actually committing
            commit_number: 0,
            time: Time::now(),
            event: event.clone(),
        };

        {
            let mut guarded_events = self.events.write().await;
            c.commit_number = guarded_events.len().try_into().unwrap();
            match condition {
                commit::Condition::None => {}
                commit::Condition::Number(want_commit_number) => {
                    assert_eq!(c.commit_number, want_commit_number);
                }
            }
            guarded_events.push(c.clone());
        }

        self.sender.send(c.commit_number).await.unwrap();
        Ok(c)
    }

    async fn read(
        &self,
        start_commit_number: commit::Number,
    ) -> store::Result<Self::EventIterator> {
        Ok(EventIterator::new(Arc::clone(&self.events), start_commit_number))
    }

    async fn subscribe(
        &self,
        start_commit_number: commit::Number,
    ) -> store::Result<Self::EventSubscription> {
        Ok(EventSubscription::new(
            Arc::clone(&self.events),
            start_commit_number,
            self.receiver.clone(),
        ))
    }
}

pub struct EventIterator<T: Revision> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: commit::Number,
}

impl<T: Revision> EventIterator<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: commit::Number,
    ) -> Self {
        Self { events, commit_number: start_commit_number }
    }
}

#[allow(clippy::future_not_send)]
impl<T: Revision> stream::Iterator<CommittedEvent<T>> for EventIterator<T> {
    async fn next(&mut self) -> Option<CommittedEvent<T>> {
        let events = self.events.read().await;
        let index = usize::try_from(self.commit_number).unwrap();
        let event = events.get(index);
        if event.is_some() {
            self.commit_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<T: Revision> {
    events: SmartVec<CommittedEvent<T>>,
    commit_number: commit::Number,
    receiver: async_channel::Receiver<commit::Number>,
}

impl<T: Revision> EventSubscription<T> {
    #[must_use]
    const fn new(
        events: SmartVec<CommittedEvent<T>>,
        start_commit_number: commit::Number,
        receiver: async_channel::Receiver<commit::Number>,
    ) -> Self {
        Self { events, commit_number: start_commit_number, receiver }
    }
}

#[allow(clippy::future_not_send)]
impl<T: Revision> stream::Subscription<CommittedEvent<T>>
    for EventSubscription<T>
{
    async fn next(&mut self) -> Option<CommittedEvent<T>> {
        {
            let events = self.events.read().await;
            let index = usize::try_from(self.commit_number).unwrap();
            let event = events.get(index);
            if let Some(event) = event {
                self.commit_number += 1;
                return Some((*event).clone());
            }
        }
        while let Ok(commit_number) = self.receiver.recv().await {
            if commit_number >= self.commit_number {
                let events = self.events.read().await;
                let index = usize::try_from(self.commit_number).unwrap();
                let event = events.get(index).unwrap();
                self.commit_number += 1;
                return Some((*event).clone());
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
