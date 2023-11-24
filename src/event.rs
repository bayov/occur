use crate::types::{Time, TimeZone};
use crate::{Id, SequenceNumber};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::{Index, Range};

pub trait Event: Sized + Clone + Debug + Send + Sync {}

#[derive(Debug, Clone)]
pub struct Recorded<T: Event> {
    /// The time at which the event was recorded.
    pub time: Time,
    /// The recorded event.
    pub event: T,
}

pub struct Stream<'e, ID: Id, T: Event> {
    id: ID,
    start_sequence_number: SequenceNumber,
    events: Cow<'e, [Recorded<T>]>,
}

impl<'e, ID: Id, T: Event> Stream<'e, ID, T> {
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
        events: Cow<'e, [Recorded<T>]>,
    ) -> Self {
        Self { id, start_sequence_number, events }
    }

    /// Returns the stream's ID.
    pub const fn id(&self) -> &ID { &self.id }

    /// Records the given event into the stream.
    pub fn record(&mut self, event: T) {
        self.events.to_mut().push(Recorded { time: TimeZone::now(), event });
    }

    /// Returns the recorded events within the stream.
    ///
    /// Note that this is not necessarily the full list of all recorded events.
    /// Depending on how this stream object was constructed, it might hold only
    /// a slice of events.
    pub fn events(&self) -> &[Recorded<T>] { &self.events }

    /// Same as [`Stream::events()`], but takes ownership of the events.
    pub fn take_events(self) -> Vec<Recorded<T>> { self.events.to_vec() }

    /// Returns the range of sequence numbers of events recorded within the
    /// stream.
    pub fn sequence_numbers_range(&self) -> Range<SequenceNumber> {
        let s = self.start_sequence_number;
        s..(s + self.events.len())
    }
}

impl<'e, ID: Id, T: Event> Index<SequenceNumber> for Stream<'e, ID, T>
where
    [Recorded<T>]: ToOwned<Owned = Vec<Recorded<T>>>,
{
    type Output = Recorded<T>;

    /// Returns the recorded event with sequence number `index`.
    ///
    /// # Panics
    /// If the recorded event with the given sequence number doesn't exist in
    /// the stream.
    fn index(&self, index: SequenceNumber) -> &Recorded<T> {
        &self.events[index - self.start_sequence_number]
    }
}

// impl<'e, ID: Id, T: Event> Index<Range<SequenceNumber>> for Stream<'e, ID, T>
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
