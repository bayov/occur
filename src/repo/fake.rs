use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

use futures_locks::RwLock;

use crate::{repo, Event, SequenceNumber};

type Id = u32;

#[derive(Default)]
pub struct Repository<T: Event> {
    next_id: Cell<Id>,
    events_by_stream_id: HashMap<Id, Stream<T>>,
}

impl<T: Event> Repository<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: Cell::new(0), events_by_stream_id: HashMap::default() }
    }
}

impl<T: Event> repo::Repository<T> for Repository<T> {
    type Id = Id;
    type Stream = Stream<T>;

    fn new_id(&mut self) -> Id {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    fn stream(&mut self, id: Id) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Clone)]
pub struct Stream<T: Event> {
    events: SmartVec<Timed<T>>,
    sender: async_channel::Sender<SequenceNumber>,
    receiver: async_channel::Receiver<SequenceNumber>,
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

impl<T: Event> repo::Stream<T> for Stream<T> {
    type Id = Id;
    type EventIterator = EventIterator<T>;
    type EventSubscription = EventSubscription<T>;

    fn id(&self) -> &Self::Id { todo!() }

    async fn write(
        &mut self,
        sequence_number: SequenceNumber,
        event: &Timed<T>,
    ) -> repo::Result<()> {
        let want_sequence_number =
            SequenceNumber(self.events.read().await.len());
        assert_eq!(sequence_number, want_sequence_number);
        self.events.write().await.push(event.clone());
        self.sender.send(sequence_number).await.expect("Should not fail");
        Ok(())
    }

    async fn read(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventIterator> {
        Ok(EventIterator::new(Arc::clone(&self.events), start_sequence_number))
    }

    async fn subscribe(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventSubscription> {
        Ok(EventSubscription::new(
            Arc::clone(&self.events),
            start_sequence_number,
            self.receiver.clone(),
        ))
    }
}

pub struct EventIterator<T: Event> {
    events: SmartVec<Timed<T>>,
    sequence_number: SequenceNumber,
}

impl<T: Event> EventIterator<T> {
    #[must_use]
    const fn new(
        events: SmartVec<Timed<T>>,
        start_sequence_number: SequenceNumber,
    ) -> Self {
        Self { events, sequence_number: start_sequence_number }
    }
}

impl<T: Event> repo::EventIterator<T> for EventIterator<T> {
    async fn next(&mut self) -> Option<Timed<T>> {
        let events = self.events.read().await;
        let event = events.get(self.sequence_number.0);
        if event.is_some() {
            self.sequence_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<T: Event> {
    events: SmartVec<Timed<T>>,
    sequence_number: SequenceNumber,
    receiver: async_channel::Receiver<SequenceNumber>,
}

impl<T: Event> EventSubscription<T> {
    #[must_use]
    const fn new(
        events: SmartVec<Timed<T>>,
        start_sequence_number: SequenceNumber,
        receiver: async_channel::Receiver<SequenceNumber>,
    ) -> Self {
        Self { events, sequence_number: start_sequence_number, receiver }
    }
}

impl<T: Event> repo::EventSubscription<T> for EventSubscription<T> {
    async fn next(&mut self) -> Option<Timed<T>> {
        {
            let events = self.events.read().await;
            let event = events.get(self.sequence_number.0);
            if let Some(event) = event {
                self.sequence_number += 1;
                return Some(event.clone());
            }
        }
        while let Ok(sequence_number) = self.receiver.recv().await {
            if sequence_number >= self.sequence_number {
                let events = self.events.read().await;
                let event = events.get(self.sequence_number.0).unwrap();
                self.sequence_number += 1;
                return Some(event.clone());
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
