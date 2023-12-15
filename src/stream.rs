use std::ops::{Index, Range};

use crate::{RecordedEvent, Time, Version};

#[allow(clippy::module_name_repetitions)] // exported from crate root
pub trait StreamDescription {
    const NAME: &'static str;
    type Id: Clone + Eq;
    type Time: Time;
    type Event;
}

pub struct Stream<T: StreamDescription> {
    id: T::Id,
    events: Vec<RecordedEvent<T>>,
}

impl<T: StreamDescription> Stream<T> {
    /// Creates an empty event stream.
    #[must_use]
    pub fn new(id: T::Id) -> Self { Self { id, events: Vec::default() } }

    /// Returns the stream's ID.
    pub const fn id(&self) -> &T::Id { &self.id }

    /// Creates an event stream from a list of previously recorded events.
    ///
    /// The stream doesn't have to hold all recorded events. It can hold an
    /// arbitrary slice of versions.
    ///
    /// The same stream `id` that was previously used to record the events
    /// should be provided, as well as the `start_version` of the first
    /// recorded event in the given `events` list.
    // pub const fn from_recorded_events(
    //     id: ID,
    //     start_version: Version,
    //     events: Cow<'a, [Timed<T>]>,
    // ) -> Self {
    //     Self { id, start_version, events }
    // }

    /// Records the given event into the stream.
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn record(&mut self, event: T::Event) -> &RecordedEvent<T> {
        self.events.push(RecordedEvent {
            id: self.id.clone(),
            version: Version(u32::try_from(self.events.len()).unwrap()),
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
    pub fn events(&self) -> &[RecordedEvent<T>] { &self.events }

    /// Same as [`Stream::events()`], but takes ownership of the events.
    pub fn take_events(self) -> Vec<RecordedEvent<T>> { self.events }

    /// Returns the range of versions of events recorded within the
    /// stream.
    #[allow(clippy::range_plus_one)]
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn versions_range(&self) -> Range<Version> {
        if self.events.is_empty() {
            return Range::default();
        }
        let first = self.events.first().unwrap().version;
        let last = self.events.last().unwrap().version;
        first..(last + 1)
    }
}

impl<T: StreamDescription> Index<Version> for Stream<T> {
    type Output = RecordedEvent<T>;

    /// Returns the recorded event with version `index`.
    ///
    /// # Panics
    /// If the recorded event with the given version is not found in the stream.
    fn index(&self, index: Version) -> &RecordedEvent<T> {
        &self.events[usize::try_from(index.0).unwrap()]
    }
}

// impl<'a, ID: Id, T: Event> Index<Range<Version>> for Stream<'a, ID, T>
// where
//     [RecordedEvent<T>]: ToOwned<Owned = Vec<RecordedEvent<T>>>,
// {
//     type Output = Self;
//
//     fn index(&self, index: Range<Version>) -> &Self {
//         let s = self.start_version;
//         Self {} & self.events[(index.start - s)..(index.end - s)]
//     }
// }
