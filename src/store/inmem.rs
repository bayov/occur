use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use futures_locks::RwLock;
use impl_tools::autoimpl;

use crate::store::CommitNumber;
use crate::{store, stream_desc, Event};

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

#[autoimpl(Default)]
pub struct Store<D: stream_desc::StreamDesc> {
    events_by_stream_id: HashMap<D::Id, Stream<D>>,
}

impl<D: stream_desc::StreamDesc> Store<D> {
    #[must_use]
    pub fn new() -> Self { Self { events_by_stream_id: HashMap::default() } }
}

impl<D: stream_desc::StreamDesc> store::Store<D> for Store<D> {
    type Stream = Stream<D>;

    fn stream(&mut self, id: D::Id) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[autoimpl(Clone)]
pub struct Stream<D: stream_desc::StreamDesc> {
    events: SmartVec<CommittedEvent<D::Event>>,
    sender: async_channel::Sender<CommitNumber>,
    receiver: async_channel::Receiver<CommitNumber>,
}

impl<D: stream_desc::StreamDesc> Stream<D> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Arc::default(), sender, receiver }
    }
}

impl<D: stream_desc::StreamDesc> Default for Stream<D> {
    fn default() -> Self { Self::new() }
}

impl<D: stream_desc::StreamDesc> store::Stream<D> for Stream<D> {
    type CommittedEvent = CommittedEvent<D::Event>;
    type EventIterator = EventIterator<D::Event>;
    type EventSubscription = EventSubscription<D::Event>;

    fn id(&self) -> &D::Id { todo!() }

    async fn commit(
        &mut self,
        event: &D::Event,
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
