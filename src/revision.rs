//! Event revisioning.
//!
//! TODO: Doc

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

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
pub enum OldOrNew<
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
> {
    Old(OldEvent),
    New(NewEvent),
}

/// Converts old event revisions to newer ones.
pub trait Converter<OldEvent, NewEvent>
where
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
{
    /// Converts an old event variant to a newer one.
    ///
    /// Use [`Self::convert_until_new`] to convert an old event as many times
    /// as needed to acquire an instance of `NewEvent`.
    ///
    /// Implementation guideline:
    /// -------------------------
    /// Ensure that each invocation of `convert` returns a newer event revision,
    /// to avoid an infinite conversion loop.
    ///
    /// When using the default revision type, [`Pair`], this function should
    /// return an event which has a higher revision number.
    fn convert(old_event: OldEvent) -> OldOrNew<OldEvent, NewEvent>;

    /// Converts an old event variant as many times as needed until it becomes a
    /// new event variant.
    fn convert_until_new(old_event: OldEvent) -> NewEvent {
        match Self::convert(old_event) {
            OldOrNew::Old(old_event) => Self::convert_until_new(old_event),
            OldOrNew::New(new_event) => new_event,
        }
    }

    /// TODO doc
    #[must_use]
    fn supported_revisions() -> HashSet<NewEvent::Revision> {
        let mut new_revisions = NewEvent::supported_revisions();
        let old_revisions = OldEvent::supported_revisions();

        let mut intersection = new_revisions.intersection(&old_revisions);
        if let Some(conflicting_revision) = intersection.next() {
            let panic_msg = indoc::formatdoc!(
                r#"
                Conflicting revision in definition of {self_type_name}.
    
                The same revision appears in both OldEvent and NewEvent types.
    
                    Revision = {conflicting_revision:?}
                    OldEvent = {old_event_type_name}
                    NewEvent = {new_event_type_name}
    
                Ensure you've set the revision of each event appropriately.
                "#,
                self_type_name = std::any::type_name::<Self>(),
                conflicting_revision = conflicting_revision,
                old_event_type_name = std::any::type_name::<OldEvent>(),
                new_event_type_name = std::any::type_name::<NewEvent>(),
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
pub struct EmptyConverter;

impl<OldEvent, NewEvent> Converter<OldEvent, NewEvent> for EmptyConverter
where
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
{
    fn convert(_: OldEvent) -> OldOrNew<OldEvent, NewEvent> {
        panic!("revision::EmptyConverter::convert() should not be called")
    }
}
