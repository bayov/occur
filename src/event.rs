use std::fmt::Debug;
use std::ops::{Index, Range};

use crate::types::Time;
use crate::Version;

pub trait StreamDescriptor {
    const NAME: &'static str;
    type Id: Clone;
    type Event;
}

#[macro_export]
macro_rules! stream_descriptor {
    {
        const NAME = $name:expr;
        type Id = $id:ty;
        type Event = $event:ty;
    } => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StreamDescriptor;

        impl $crate::StreamDescriptor for StreamDescriptor {
            const NAME: &'static str = $name;
            type Id = $id;
            type Event = $event;
        }

        pub type Ref = $crate::Ref<StreamDescriptor>;
        pub type Stream = $crate::Stream<StreamDescriptor>;
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ref<T: StreamDescriptor> {
    pub id: T::Id,
    pub version: Version,
}

pub struct Recorded<T: StreamDescriptor> {
    pub id: T::Id,
    pub version: Version,
    pub time: Time,
    pub event: T::Event,
}

impl<T: StreamDescriptor> Recorded<T> {
    #[must_use]
    pub fn refer(&self) -> Ref<T> {
        Ref { id: self.id.clone(), version: self.version }
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
    pub fn record(&mut self, event: T::Event) -> &Recorded<T> {
        self.events.push(Recorded {
            id: self.id.clone(),
            version: Version(self.events.len()),
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

impl<T: StreamDescriptor> Index<Version> for Stream<T> {
    type Output = Recorded<T>;

    /// Returns the recorded event with version `index`.
    ///
    /// # Panics
    /// If the recorded event with the given version is not found in the stream.
    fn index(&self, index: Version) -> &Recorded<T> { &self.events[index.0] }
}

// impl<'a, ID: Id, T: Event> Index<Range<Version>> for Stream<'a, ID, T>
// where
//     [Recorded<T>]: ToOwned<Owned = Vec<Recorded<T>>>,
// {
//     type Output = Self;
//
//     fn index(&self, index: Range<Version>) -> &Self {
//         let s = self.start_version;
//         Self {} & self.events[(index.start - s)..(index.end - s)]
//     }
// }
