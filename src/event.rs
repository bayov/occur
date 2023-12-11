use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::{Index, Range};

use crate::types::Time;
use crate::{Id, Ref, Referable, SequenceNumber};

/// An event that can be recorded as a fact in an event [`Stream`].
pub trait Event: Sized + Clone + Debug + Send + Sync {}

impl<T: Event> Referable for T {}

#[derive(Debug, Clone)]
pub struct Timed<T: Event> {
    /// The time at which the event was recorded.
    pub time: Time,
    /// The recorded event.
    pub event: T,
}

pub struct Recorded<'a, ID: Id, T: Event> {
    pub stream: &'a Stream<'a, ID, T>,
    pub sequence_number: SequenceNumber,
}

impl<'a, ID: Id, T: Event> Recorded<'a, ID, T> {
    #[must_use]
    pub const fn id(&self) -> &ID { &self.stream.id }

    #[must_use]
    pub fn time(&self) -> Time { self.timed_event().time }

    #[must_use]
    pub fn event(&self) -> &T { &self.timed_event().event }

    #[must_use]
    pub fn refer(&self) -> Ref<T, ID> {
        Ref::new(self.stream.id.clone(), self.sequence_number)
    }

    #[must_use]
    pub fn timed_event(&self) -> &Timed<T> {
        &self.stream[self.sequence_number]
    }
}

pub struct Stream<'a, ID: Id, T: Event> {
    id: ID,
    start_sequence_number: SequenceNumber,
    events: Cow<'a, [Timed<T>]>,
}

impl<'a, ID: Id, T: Event> Stream<'a, ID, T> {
    /// Creates an empty event stream.
    #[must_use]
    pub const fn new(id: ID) -> Self {
        Self {
            id,
            start_sequence_number: SequenceNumber(0),
            events: Cow::Borrowed(&[]),
        }
    }

    /// Creates an event stream from a list of previously recorded events.
    ///
    /// The stream doesn't have to hold all recorded events. It can hold an
    /// arbitrary slice of sequence numbers.
    ///
    /// The same stream `id` that was previously used to record the events
    /// should be provided, as well as the `start_sequence_number` of the first
    /// recorded event in the given `events` list.
    pub const fn from_recorded_events(
        id: ID,
        start_sequence_number: SequenceNumber,
        events: Cow<'a, [Timed<T>]>,
    ) -> Self {
        Self { id, start_sequence_number, events }
    }

    /// Returns the stream's ID.
    pub const fn id(&self) -> &ID { &self.id }

    /// Records the given event into the stream.
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn record(&mut self, event: T) -> Recorded<'_, ID, T> {
        self.events.to_mut().push(Timed { time: Time::now(), event });
        Recorded {
            stream: self,
            sequence_number: SequenceNumber(self.events.len() - 1),
        }
    }

    pub fn record_array<const N: usize>(&mut self, events: [T; N]) {
        for event in events {
            self.record(event);
        }
    }

    /// Returns the recorded events within the stream.
    ///
    /// Note that this is not necessarily the full list of all recorded events.
    /// Depending on how this stream object was constructed, it might hold only
    /// a slice of events.
    pub fn events(&self) -> &[Timed<T>] { &self.events }

    /// Same as [`Stream::events()`], but takes ownership of the events.
    pub fn take_events(self) -> Vec<Timed<T>> { self.events.to_vec() }

    /// Returns the range of sequence numbers of events recorded within the
    /// stream.
    pub fn sequence_numbers_range(&self) -> Range<SequenceNumber> {
        let s = self.start_sequence_number;
        s..(s + self.events.len())
    }
}

impl<'a, ID: Id, T: Event> Index<SequenceNumber> for Stream<'a, ID, T>
where
    [Timed<T>]: ToOwned<Owned = Vec<Timed<T>>>,
{
    type Output = Timed<T>;

    /// Returns the recorded event with sequence number `index`.
    ///
    /// # Panics
    /// If the recorded event with the given sequence number doesn't exist in
    /// the stream.
    fn index(&self, index: SequenceNumber) -> &Timed<T> {
        &self.events[index - self.start_sequence_number]
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
