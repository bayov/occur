use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

use futures_locks::RwLock;

use crate::{repo, Event, Recorded, SequenceNumber};

type Id = u32;

#[derive(Default)]
pub struct Repository<T: Event> {
    next_id: Cell<Id>,
    events_by_stream_id: HashMap<Id, SmartInnerStream<T>>,
}

impl<T: Event> Repository<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: Cell::new(0), events_by_stream_id: HashMap::default() }
    }

    // fn stream(&self, id: Id) -> Option<&Stream<T>> {
    //     self.events_by_stream_id.get(&id)
    // }
    //
    // fn mut_stream(&mut self, id: Id) -> &mut Stream<T> {
    //     self.events_by_stream_id.entry(id).or_default()
    // }
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
        Stream::new(Arc::clone(self.events_by_stream_id.entry(id).or_default()))
    }
}

struct InnerStream<T: Event> {
    events: Vec<Recorded<T>>,
    sender: async_channel::Sender<SequenceNumber>,
    receiver: async_channel::Receiver<SequenceNumber>,
}

impl<T: Event> InnerStream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Vec::default(), sender, receiver }
    }
}

impl<T: Event> Default for InnerStream<T> {
    fn default() -> Self { Self::new() }
}

type SmartInnerStream<T> = Arc<RwLock<InnerStream<T>>>;

pub struct Stream<T: Event>(SmartInnerStream<T>);

impl<T: Event> Stream<T> {
    #[must_use]
    fn new(inner_stream: SmartInnerStream<T>) -> Self { Self(inner_stream) }
}

impl<T: Event> repo::Stream<T> for Stream<T> {
    type Id = Id;
    type EventIterator = EventIterator<T>;
    type EventSubscription = EventSubscription<T>;

    fn id(&self) -> &Self::Id { todo!() }

    async fn write(
        &mut self,
        sequence_number: SequenceNumber,
        event: &Recorded<T>,
    ) -> repo::Result<()> {
        let want_sequence_number =
            SequenceNumber(self.0.read().await.events.len());
        assert_eq!(sequence_number, want_sequence_number);
        self.0.write().await.events.push(event.clone());
        self.0
            .read()
            .await
            .sender
            .send(sequence_number)
            .await
            .expect("Should not fail");
        Ok(())
    }

    async fn read(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventIterator> {
        Ok(EventIterator::new(Arc::clone(&self.0), start_sequence_number))
    }

    async fn subscribe(
        &self,
        start_sequence_number: SequenceNumber,
    ) -> repo::Result<Self::EventSubscription> {
        Ok(EventSubscription::new(
            Arc::clone(&self.0),
            start_sequence_number,
            self.0.read().await.receiver.clone(),
        ))
    }
}

pub struct EventIterator<T: Event> {
    stream: SmartInnerStream<T>,
    sequence_number: SequenceNumber,
}

impl<T: Event> EventIterator<T> {
    #[must_use]
    const fn new(
        stream: SmartInnerStream<T>,
        start_sequence_number: SequenceNumber,
    ) -> Self {
        Self { stream, sequence_number: start_sequence_number }
    }
}

impl<T: Event> repo::EventIterator<T> for EventIterator<T> {
    async fn next(&mut self) -> Option<Recorded<T>> {
        let stream = self.stream.read().await;
        let event = stream.events.get(self.sequence_number.0);
        if event.is_some() {
            self.sequence_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<T: Event> {
    stream: SmartInnerStream<T>,
    sequence_number: SequenceNumber,
    receiver: async_channel::Receiver<SequenceNumber>,
}

impl<T: Event> EventSubscription<T> {
    #[must_use]
    const fn new(
        stream: SmartInnerStream<T>,
        start_sequence_number: SequenceNumber,
        receiver: async_channel::Receiver<SequenceNumber>,
    ) -> Self {
        Self { stream, sequence_number: start_sequence_number, receiver }
    }
}

impl<T: Event> repo::EventSubscription<T> for EventSubscription<T> {
    async fn next(&mut self) -> Option<Recorded<T>> {
        {
            let stream = self.stream.read().await;
            let event = stream.events.get(self.sequence_number.0);
            if let Some(event) = event {
                self.sequence_number += 1;
                return Some(event.clone());
            }
        }
        while let Ok(sequence_number) = self.receiver.recv().await {
            if sequence_number >= self.sequence_number {
                let stream = self.stream.read().await;
                let event = stream.events.get(self.sequence_number.0).unwrap();
                self.sequence_number += 1;
                return Some(event.clone());
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
