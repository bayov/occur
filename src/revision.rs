use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::intrinsics::type_name;
use std::marker::PhantomData;

use crate::{event, Event};

pub trait Revision: Debug + Eq + Hash {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAndNumber<
    T: Debug + Eq + Hash = &'static str,
    R: Debug + Eq + Hash = u8,
> {
    pub event_type: T,
    pub revision_number: R,
}

impl<T: Debug + Eq + Hash, R: Debug + Eq + Hash> TypeAndNumber<T, R> {
    #[must_use]
    pub const fn new(event_type: T, revision_number: R) -> Self {
        Self { event_type, revision_number }
    }
}

impl<T: Debug + Eq + Hash, R: Debug + Eq + Hash> Revision
    for TypeAndNumber<T, R>
{
}

pub enum OldOrNew<
    OldEvent: Event<Revision = NewEvent::Revision>,
    NewEvent: Event,
> {
    Old(OldEvent),
    New(NewEvent),
}

pub trait Converter {
    type OldEvent: Event<Revision = <Self::NewEvent as Event>::Revision>;
    type NewEvent: Event;

    fn convert(
        old_event: Self::OldEvent,
    ) -> OldOrNew<Self::OldEvent, Self::NewEvent>;

    fn convert_to_new(old_event: Self::OldEvent) -> Self::NewEvent {
        match Self::convert(old_event) {
            OldOrNew::Old(old_event) => Self::convert_to_new(old_event),
            OldOrNew::New(new_event) => new_event,
        }
    }

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

pub struct EmptyConverter<T: Event>(PhantomData<T>);

impl<T: Event> Converter for EmptyConverter<T> {
    type OldEvent = event::Empty<T::Revision>;
    type NewEvent = T;

    fn convert(_: Self::OldEvent) -> OldOrNew<Self::OldEvent, Self::NewEvent> {
        panic!("revision::EmptyConverter::convert() should not be called")
    }
}
