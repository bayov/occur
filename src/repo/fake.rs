use crate::{repo, Event, Recorded, SequenceNumber};
use std::collections::HashMap;
use std::error::Error;

type Id = u32;

#[derive(Default)]
pub struct Repository<T: Event> {
    next_id: Id,
    events_by_stream_id: HashMap<Id, Stream<T>>,
}

impl<T: Event> Repository<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: 0, events_by_stream_id: HashMap::default() }
    }

    fn stream(&self, id: Id) -> Option<&Stream<T>> {
        self.events_by_stream_id.get(&id)
    }

    fn mut_stream(&mut self, id: Id) -> &mut Stream<T> {
        self.events_by_stream_id.entry(id).or_default()
    }
}

impl<T: Event> repo::Repository<T> for Repository<T> {
    type StreamId = Id;
    type EventIterator<'s> = EventIterator<'s, T> where Self: 's;
    type EventSubscription<'s> = EventSubscription<'s, T> where Self: 's;

    fn new_id(&mut self) -> Self::StreamId {
        self.next_id += 1;
        self.next_id
    }

    async fn write_event(
        &mut self,
        stream_id: Self::StreamId,
        sequence_number: SequenceNumber,
        event: &Recorded<T>,
    ) -> Result<(), Box<dyn Error>> {
        let stream = self.mut_stream(stream_id);
        let events = &mut stream.events;
        let want_sequence_number = SequenceNumber(events.len());
        assert_eq!(sequence_number, want_sequence_number);
        events.push(event.clone());
        stream.sender.send(sequence_number).await.expect("Should not fail");
        Ok(())
    }

    async fn read_stream(
        &self,
        stream_id: &Self::StreamId,
        start_sequence_number: SequenceNumber,
    ) -> Result<Self::EventIterator<'_>, Box<dyn Error>> {
        let stream = &self.stream(*stream_id);
        stream.map_or_else(
            || {
                panic!("stream doesn't exist");
            },
            |stream| {
                Ok(EventIterator::new(&stream.events, start_sequence_number))
            },
        )
    }

    async fn subscribe_to_stream(
        &self,
        stream_id: &Self::StreamId,
        start_sequence_number: SequenceNumber,
    ) -> Result<Self::EventSubscription<'_>, Box<dyn Error>> {
        let stream = &self.stream(*stream_id);
        stream.map_or_else(
            || {
                panic!("stream doesn't exist");
            },
            |stream| {
                Ok(EventSubscription::new(
                    &stream.events,
                    start_sequence_number,
                    stream.receiver.clone(),
                ))
            },
        )
    }
}

struct Stream<T: Event> {
    events: Vec<Recorded<T>>,
    sender: async_channel::Sender<SequenceNumber>,
    receiver: async_channel::Receiver<SequenceNumber>,
}

impl<T: Event> Stream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Vec::default(), sender, receiver }
    }
}

impl<T: Event> Default for Stream<T> {
    fn default() -> Self { Self::new() }
}

pub struct EventIterator<'s, T: Event> {
    events: &'s Vec<Recorded<T>>,
    sequence_number: SequenceNumber,
}

impl<'s, T: Event> EventIterator<'s, T> {
    #[must_use]
    const fn new(
        events: &'s Vec<Recorded<T>>,
        start_sequence_number: SequenceNumber,
    ) -> Self {
        Self { events, sequence_number: start_sequence_number }
    }
}

impl<'s, T: Event> repo::EventIterator<'s, T> for EventIterator<'s, T> {
    async fn next(&mut self) -> Option<Recorded<T>> {
        let event = self.events.get(self.sequence_number.0);
        if event.is_some() {
            self.sequence_number += 1;
        }
        event.cloned()
    }
}

pub struct EventSubscription<'s, T: Event> {
    events: &'s Vec<Recorded<T>>,
    sequence_number: SequenceNumber,
    receiver: async_channel::Receiver<SequenceNumber>,
}

impl<'s, T: Event> EventSubscription<'s, T> {
    #[must_use]
    const fn new(
        events: &'s Vec<Recorded<T>>,
        start_sequence_number: SequenceNumber,
        receiver: async_channel::Receiver<SequenceNumber>,
    ) -> Self {
        Self { events, sequence_number: start_sequence_number, receiver }
    }
}

impl<'s, T: Event> repo::EventSubscription<'s, T> for EventSubscription<'s, T> {
    async fn next(&mut self) -> Option<Recorded<T>> {
        let event = self.events.get(self.sequence_number.0);
        if let Some(event) = event {
            self.sequence_number += 1;
            Some(event.clone())
        } else {
            while let Ok(sequence_number) = self.receiver.recv().await {
                if sequence_number >= self.sequence_number {
                    let event =
                        self.events.get(self.sequence_number.0).unwrap();
                    self.sequence_number += 1;
                    return Some(event.clone());
                }
            }
            None
        }
    }

    fn stop(self) { self.receiver.close(); }
}

// pub struct EventSubscription<'s, T: Event> {}
//
// impl<'s, T: Event> repo::EventSubscription<'s, T>
//     for EventSubscription<'s, T>
// {
// }
