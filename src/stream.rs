use std::ops::{Index, Range};
use std::time::SystemTime;

use crate::{revision, CommitNumber, CommittedEvent, Event, Time};

#[allow(clippy::module_name_repetitions)] // exported from crate root
pub trait StreamDescription {
    const NAME: &'static str;
    type Id: Clone + Eq;
    type Event: Event;

    type Time: Time = SystemTime;

    type RevisionConverter: revision::Converter<NewEvent = Self::Event> =
        revision::EmptyConverter<Self::Event>;
}

pub struct Stream<T: StreamDescription> {
    id: T::Id,
    events: Vec<CommittedEvent<T>>,
}

impl<T: StreamDescription> Stream<T> {
    /// Creates an empty event stream.
    #[must_use]
    pub fn new(id: T::Id) -> Self { Self { id, events: Vec::default() } }

    /// Returns the stream's ID.
    pub const fn id(&self) -> &T::Id { &self.id }

    /// Creates an event stream from a list of previously committed events.
    ///
    /// The stream doesn't have to hold all committed events. It can hold an
    /// arbitrary slice of commit numbers.
    ///
    /// The same stream `id` that was previously used to commit the events
    /// should be provided, as well as the `start_commit_number` of the first
    /// committed event in the given `events` list.
    // pub const fn from_committed_events(
    //     id: ID,
    //     start_commit_number: CommitNumber,
    //     events: Cow<'a, [Timed<T>]>,
    // ) -> Self {
    //     Self { id, start_commit_number, events }
    // }

    /// Commits the given event into the stream.
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn commit(&mut self, event: T::Event) -> &CommittedEvent<T> {
        self.events.push(CommittedEvent {
            id: self.id.clone(),
            commit_number: CommitNumber(
                u32::try_from(self.events.len()).unwrap(),
            ),
            time: Time::now(),
            event,
        });
        self.events.last().unwrap()
    }

    pub fn commit_array<const N: usize>(&mut self, events: [T::Event; N]) {
        for event in events {
            self.commit(event);
        }
    }

    /// Returns the committed events within the stream.
    ///
    /// Note that this is not necessarily the full list of all committed events.
    /// Depending on how this stream object was constructed, it might hold only
    /// a slice of events.
    pub fn events(&self) -> &[CommittedEvent<T>] { &self.events }

    /// Same as [`Stream::events()`], but takes ownership of the events.
    pub fn take_events(self) -> Vec<CommittedEvent<T>> { self.events }

    /// Returns the commit numbers range committed within the stream.
    #[allow(clippy::range_plus_one)]
    #[allow(clippy::missing_panics_doc)] // doesn't panic
    pub fn commit_numbers_range(&self) -> Range<CommitNumber> {
        if self.events.is_empty() {
            return Range::default();
        }
        let first = self.events.first().unwrap().commit_number;
        let last = self.events.last().unwrap().commit_number;
        first..(last + 1)
    }
}

impl<T: StreamDescription> Index<CommitNumber> for Stream<T> {
    type Output = CommittedEvent<T>;

    /// Returns the committed event with commit number `index`.
    ///
    /// # Panics
    /// If the committed event with the given commit number is not found in the
    /// stream.
    fn index(&self, index: CommitNumber) -> &CommittedEvent<T> {
        &self.events[usize::try_from(index.0).unwrap()]
    }
}

// impl<'a, ID: Id, T: Event> Index<Range<CommitNumber>> for Stream<'a, ID, T>
// where
//     [CommittedEvent<T>]: ToOwned<Owned = Vec<CommittedEvent<T>>>,
// {
//     type Output = Self;
//
//     fn index(&self, index: Range<CommitNumber>) -> &Self {
//         let s = self.start_commit_number;
//         Self {} & self.events[(index.start - s)..(index.end - s)]
//     }
// }
