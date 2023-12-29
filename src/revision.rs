//! Event revisioning.
//!
//! TODO: Doc

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::Event;

/// The revision of an [`Event`] variant, uniquely identifying its form.
pub trait Revision = Debug + Clone + Eq + Hash;

/// A pair of event `name` (string) and revision `number` (integer) that
/// together uniquely identify an [`Event`] variant form.
///
/// This type is used as the default for [`Event::Revision`].
///
/// By default, `name` is `&'static str` and `number` is `u8`. Generic
/// parameters `V` and `N` can be provided to change these types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair<V = &'static str, N = u8> {
    /// The name of the event variant.
    pub name: V,

    /// The revision number of the event variant.
    ///
    /// This field should be incremented by 1 every time a new revision is
    /// created for an event variant.
    pub number: N,
}

impl<V, N> Pair<V, N> {
    /// Constructs a new event revision from a pair of `name` and `number`.
    #[must_use]
    pub const fn new(name: V, number: N) -> Self { Self { name, number } }
}

/// Holds either a new event variant or an old revision of one.
pub enum OldOrNew<OldEvent, NewEvent>
where
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
{
    Old(OldEvent),
    New(NewEvent),
}

/// An event that can be converted to a newer event type.
pub trait Convert: Event {
    /// The newer event to which this event can be converted to.
    type NewEvent: Event<Revision = Self::Revision>;

    /// Converts this event variant to a newer one.
    ///
    /// Use [`Self::convert_until_new`] to convert an old event as many times
    /// as needed to acquire an instance of [`Self::NewEvent`].
    ///
    /// Implementation guideline:
    /// -------------------------
    /// Ensure that each invocation of `convert` returns a newer event revision,
    /// to avoid an infinite conversion loop.
    fn convert(self) -> OldOrNew<Self, Self::NewEvent>;

    /// Converts this event variant as many times as needed until it becomes a
    /// new event variant.
    fn convert_until_new(self) -> Self::NewEvent {
        match Self::convert(self) {
            OldOrNew::Old(old_event) => old_event.convert_until_new(),
            OldOrNew::New(new_event) => new_event,
        }
    }
}

/// Represents an event with no variants.
///
/// Used as the default [`StreamDesc::OldEvent`] type, indicating there are no
/// existing old event variants for the described stream.
pub struct Never<T: Event>(!, PhantomData<T>);

impl<T: Event> Clone for Never<T> {
    fn clone(&self) -> Self { unreachable!() }
}

impl<T: Event> Event for Never<T> {
    type Revision = T::Revision;
    fn supported_revisions() -> HashSet<Self::Revision> { HashSet::default() }
    fn revision(&self) -> Self::Revision { unreachable!() }
}

impl<T: Event> Convert for Never<T> {
    type NewEvent = T;
    fn convert(self) -> OldOrNew<Self, Self::NewEvent> { unreachable!() }
}
