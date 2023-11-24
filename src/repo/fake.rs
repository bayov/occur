use crate::{repo, Event, Recorded, SequenceNumber};
use std::collections::HashMap;
use std::error::Error;

type Id = u32;

#[derive(Default)]
pub struct Repository<T: Event> {
    next_id: Id,
    events_by_stream_id: HashMap<Id, Vec<Recorded<T>>>,
}

impl<T: Event> Repository<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: 0, events_by_stream_id: HashMap::default() }
    }
}

impl<T: Event> repo::Repository<T> for Repository<T> {
    type Id = Id;
    type Stream<'repo> = Stream<'repo, T> where Self: 'repo;

    fn new_id(&mut self) -> Self::Id {
        self.next_id += 1;
        self.next_id
    }

    fn stream(&mut self, id: Self::Id) -> Self::Stream<'_> {
        Stream::new(self, id)
    }
}

pub struct Stream<'repo, T: Event> {
    repo: &'repo mut Repository<T>,
    id: Id,
}

impl<'repo, T: Event> Stream<'repo, T> {
    #[must_use]
    pub fn new(repo: &'repo mut Repository<T>, id: Id) -> Self {
        Self { repo, id }
    }

    fn events(&'repo self) -> Option<&'repo [Recorded<T>]> {
        self.repo.events_by_stream_id.get(&self.id).map(|events| &events[..])
    }

    fn mut_events(&mut self) -> &mut Vec<Recorded<T>> {
        self.repo.events_by_stream_id.entry(self.id).or_default()
    }
}

impl<'repo, T: Event> repo::Stream<'repo, T> for Stream<'repo, T> {
    type Id = Id;
    type EventIterator = EventIterator<'repo, T>;
    // type EventIterator = impl futures::Stream<Item = Recorded<T>> + 'repo;
    // type EventSubscription = EventSubscription<'repo, T>;

    fn id(&self) -> &Self::Id { &self.id }

    async fn write(
        &mut self,
        sequence_number: SequenceNumber,
        event: &Recorded<T>,
    ) -> Result<(), Box<dyn Error>> {
        let events = self.mut_events();
        let want_sequence_number = SequenceNumber(events.len());
        assert_eq!(sequence_number, want_sequence_number);
        events.push(event.clone());
        Ok(())
    }

    async fn read(
        &'repo self,
        start_sequence_number: SequenceNumber,
    ) -> Result<Self::EventIterator, Box<dyn Error>> {
        self.events().map_or_else(
            || {
                panic!("stream doesn't exist");
            },
            |events| Ok(EventIterator::new(events, start_sequence_number)),
        )
    }

    // async fn read(
    //     &self,
    //     start_sequence_number: SequenceNumber,
    // ) -> Result<Self::EventIterator, Box<dyn Error>> {
    //     let events = self.events();
    //     if events.is_none() {
    //         panic!("stream doesn't exist");
    //     }
    //     let events = &events.unwrap()[start_sequence_number.0..];
    //     Ok(stream::unfold(SequenceNumber(0), |sequence_number| async move {
    //         if sequence_number.0 >= events.len() {
    //             None
    //         } else {
    //             Some((events[sequence_number.0].clone(), sequence_number +
    // 1))         }
    //     }))
    // }

    // async fn subscribe(
    //     &self,
    //     start_sequence_number: SequenceNumber,
    // ) -> Result<Self::EventSubscription, Box<dyn Error>> {
    //     todo!()
    // }
}

// struct EventIteratorState<'repo, T: Event> {
//     events: &'repo [Recorded<T>],
//     sequence_number: SequenceNumber,
// }
//
// async fn advance_event_iterator<'repo, T: Event>(
//     state: EventIteratorState<'repo, T>,
// ) -> Option<(Recorded<T>, EventIteratorState<'repo, T>)> {
//     if state.sequence_number.0 >= state.events.len() {
//         None
//     } else {
//         let next_state = EventIteratorState::<'repo, T> {
//             events: state.events,
//             sequence_number: state.sequence_number + 1,
//         };
//         Some((state.events[state.sequence_number], next_state))
//     }
// }

// type AdvanceEventIterator<'repo, T: Event> =
//     fn(EventIteratorState<'repo, T>) -> Fut<'repo, T>;

pub struct EventIterator<'repo, T: Event> {
    events: &'repo [Recorded<T>],
    sequence_number: SequenceNumber,
}

impl<'repo, T: Event> EventIterator<'repo, T> {
    #[must_use]
    const fn new(
        events: &'repo [Recorded<T>],
        start_sequence_number: SequenceNumber,
    ) -> Self {
        Self { events, sequence_number: start_sequence_number }
    }
}

impl<'repo, T: Event> repo::EventIterator<'repo, T>
    for EventIterator<'repo, T>
{
    async fn next(&mut self) -> Option<Recorded<T>> {
        let event = self.events.get(self.sequence_number.0);
        if event.is_some() {
            self.sequence_number = self.sequence_number.next();
        }
        event.cloned()
    }
}

// pub struct EventSubscription<'repo, T: Event> {}
//
// impl<'repo, T: Event> repo::EventSubscription<'repo, T>
//     for EventSubscription<'repo, T>
// {
// }
