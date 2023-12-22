//! Event revisioning.
//!
//! TODO: Doc

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::intrinsics::type_name;
use std::marker::PhantomData;

use crate::{event, Event};

/// The traits that every revision type must implement.
pub trait Traits = Debug + Clone + Eq + Hash;

/// The revision of an [`Event`] variant, uniquely identifying its form.
pub trait Revision: Traits {}

/// A pair of event `name` (string) and revision `number` (integer) that
/// together uniquely identify an [`Event`] variant form.
///
/// This type is used as the default for [`Event::Revision`].
///
/// By default, `name` is `&'static str` and `number` is `u8`. Generic
/// parameters `V` and `N` can be provided to change these types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair<V: Traits = &'static str, N: Traits = u8> {
    /// The name of the event variant.
    pub name: V,

    /// The revision number of the event variant.
    ///
    /// This field should be incremented by 1 every time a new revision is
    /// created for an event variant.
    pub number: N,
}

impl<V: Traits, N: Traits> Pair<V, N> {
    /// Constructs a new event revision from a pair of `name` and `number`.
    #[must_use]
    pub const fn new(name: V, number: N) -> Self { Self { name, number } }
}

impl<V: Traits, N: Traits> Revision for Pair<V, N> {}

/// Holds either a new event variant or an old revision of one.
pub enum OldOrNew<
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
> {
    Old(OldEvent),
    New(NewEvent),
}

/// Converts old event revisions to newer ones.
pub trait Converter {
    /// Old event variants that are to be converted.
    type OldEvent: Event<Revision = <Self::NewEvent as Event>::Revision>;

    /// New event variants to converted to.
    type NewEvent: Event;

    /// Converts an old event variant to a newer one.
    ///
    /// This function will be called as many times as needed on instances of
    /// [`Self::OldEvent`] until it returns an instance of [`Self::NewEvent`].
    ///
    /// Take care to ensure that each invocation returns a newer event revision,
    /// to avoid an infinite conversion loop.
    ///
    /// Typically, when using the default [`Pair`] type, an event with revision
    /// number `n` should be converted to the event with revision number
    /// `n + 1`, until we reach the latest form of the event.
    fn convert(
        old_event: Self::OldEvent,
    ) -> OldOrNew<Self::OldEvent, Self::NewEvent>;

    /// Converts an old event variant as many times as needed until it becomes a
    /// new event variant.
    fn convert_until_new(old_event: Self::OldEvent) -> Self::NewEvent {
        match Self::convert(old_event) {
            OldOrNew::Old(old_event) => Self::convert_until_new(old_event),
            OldOrNew::New(new_event) => new_event,
        }
    }

    /// TODO doc
    #[must_use]
    fn supported_revisions() -> HashSet<<Self::NewEvent as Event>::Revision> {
        let mut new_revisions = Self::NewEvent::supported_revisions();
        let old_revisions = Self::OldEvent::supported_revisions();

        let mut intersection = new_revisions.intersection(&old_revisions);
        if let Some(conflicting_revision) = intersection.next() {
            let panic_msg = indoc::formatdoc!(
                r#"
                Conflicting revision in definition of {self_type_name}.
    
                The same revision appears in both OldEvent and NewEvent types.
    
                    Revision:        {conflicting_revision:?}
                    Old event type:  {old_event_type_name}
                    New event type:  {new_event_type_name}
    
                Ensure you've set the revision of each event appropriately.
                "#,
                self_type_name = type_name::<Self>(),
                conflicting_revision = conflicting_revision,
                old_event_type_name = type_name::<Self::OldEvent>(),
                new_event_type_name = type_name::<Self::NewEvent>(),
            );
            panic!("{}", panic_msg);
        }

        new_revisions.extend(old_revisions);
        new_revisions
    }
}

/// A no-op revision converter from an empty set of old event variants to
/// the provided event type `T`.
///
/// This type is used as the default for
/// [`crate::stream_desc::StreamDesc::RevisionConverter`], and represents the
/// fact that the described event stream has no old event revisions yet.
pub struct EmptyConverter<T: Event>(PhantomData<T>);

impl<T: Event> Converter for EmptyConverter<T> {
    type OldEvent = event::Empty<T::Revision>;
    type NewEvent = T;

    fn convert(_: Self::OldEvent) -> OldOrNew<Self::OldEvent, Self::NewEvent> {
        panic!("revision::EmptyConverter::convert() should not be called")
    }
}
