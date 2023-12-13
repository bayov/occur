use std::fmt::Debug;
use std::ops::{Index, Range};

use crate::types::Time;
use crate::SequenceNumber;

pub trait StreamDescriptor {
    type Id: Clone;
    type Event;
    fn name(&self) -> &str;
}

#[macro_export]
macro_rules! stream_descriptor {
    {
        name = $name:expr;
        type Id = $id:ty;
        type Event = $event:ty;
    } => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StreamDescriptor;

        impl $crate::StreamDescriptor for StreamDescriptor {
            type Id = $id;
            type Event = $event;
            fn name(&self) -> &str { $name }
        }

        pub type Ref = $crate::Ref<StreamDescriptor>;
        pub type Stream = $crate::Stream<StreamDescriptor>;
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ref<T: StreamDescriptor> {
    pub id: T::Id,
    pub sequence_number: SequenceNumber,
}

pub struct Recorded<T: StreamDescriptor> {
    pub id: T::Id,
    pub sequence_number: SequenceNumber,
    pub time: Time,
    pub event: T::Event,
}

impl<T: StreamDescriptor> Recorded<T> {
    #[must_use]
    pub fn refer(&self) -> Ref<T> {
        Ref { id: self.id.clone(), sequence_number: self.sequence_number }
    }
}

pub struct Stream<T: StreamDescriptor> {
    id: T::Id,
    events: Vec<Recorded<T>>,
}

impl<T: StreamDescriptor> Stream<T> {
    /// Creates an empty event stream.
    #[must_use]
    pub fn new(id: T::Id) -> Self { Self { id, events: Vec::default() } }

    /// Returns the stream's ID.
    pub const fn id(&self) -> &T::Id { &self.id }

    /// Creates an event stream from a list of previously recorded events.
    ///
    /// The stream doesn't have to hold all recorded events. It can hold an
    /// arbitrary slice of sequence numbers.
    ///
    /// The same stream `id` that was previously used to record the events
    /// should be provided, as well as the `start_sequence_number` of the first
    /// recorded event in the given `events` list.
    // pub const fn from_recorded_events(
    //     id: ID,
    //     start_sequence_number: SequenceNumber,
    //     events: Cow<'a, [Timed<T>]>,
    // ) -> Self {
    //     Self { id, start_sequence_number, events }
    // }

    /// Records the given event into the stream.
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn record(&mut self, event: T::Event) -> &Recorded<T> {
        self.events.push(Recorded {
            id: self.id.clone(),
            sequence_number: SequenceNumber(self.events.len()),
            time: Time::now(),
            event,
        });
        self.events.last().unwrap()
    }

    pub fn record_array<const N: usize>(&mut self, events: [T::Event; N]) {
        for event in events {
            self.record(event);
        }
    }

    /// Returns the recorded events within the stream.
    ///
    /// Note that this is not necessarily the full list of all recorded events.
    /// Depending on how this stream object was constructed, it might hold only
    /// a slice of events.
    pub fn events(&self) -> &[Recorded<T>] { &self.events }

    /// Same as [`Stream::events()`], but takes ownership of the events.
    pub fn take_events(self) -> Vec<Recorded<T>> { self.events }

    /// Returns the range of sequence numbers of events recorded within the
    /// stream.
    #[allow(clippy::range_plus_one)]
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn sequence_numbers_range(&self) -> Range<SequenceNumber> {
        if self.events.is_empty() {
            return Range::default();
        }
        let first = self.events.first().unwrap().sequence_number;
        let last = self.events.last().unwrap().sequence_number;
        first..(last + 1)
    }
}

impl<T: StreamDescriptor> Index<SequenceNumber> for Stream<T> {
    type Output = Recorded<T>;

    /// Returns the recorded event with sequence number `index`.
    ///
    /// # Panics
    /// If the recorded event with the given sequence number doesn't exist in
    /// the stream.
    fn index(&self, index: SequenceNumber) -> &Recorded<T> {
        &self.events[index.0]
    }
}

// impl<'a, ID: Id, T: Event> Index<Range<SequenceNumber>> for Stream<'a, ID, T>
// where
//     [Recorded<T>]: ToOwned<Owned = Vec<Recorded<T>>>,
// {
//     type Output = Self;
//
//     fn index(&self, index: Range<SequenceNumber>) -> &Self {
//         let s = self.start_sequence_number;
//         Self {} & self.events[(index.start - s)..(index.end - s)]
//     }
// }
