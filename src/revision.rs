use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::intrinsics::type_name;
use std::marker::PhantomData;

use indoc::formatdoc;

use crate::{event, Event, StreamDescription};

pub trait Revision: Debug + Eq + Hash {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeAndNumber<
    T: Debug + Eq + Hash = &'static str,
    R: Debug + Eq + Hash = u16,
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
    type NewEvent: Event;
    type OldEvent: Event<Revision = <Self::NewEvent as Event>::Revision>;

    fn convert(
        old_event: Self::OldEvent,
    ) -> OldOrNew<Self::OldEvent, Self::NewEvent>;

    fn convert_to_new(old_event: Self::OldEvent) -> Self::NewEvent {
        match Self::convert(old_event) {
            OldOrNew::Old(old_event) => Self::convert_to_new(old_event),
            OldOrNew::New(new_event) => new_event,
        }
    }
}

pub struct PanicConverter<T: Event>(PhantomData<T>);

impl<T: Event> Converter for PanicConverter<T> {
    type NewEvent = T;
    type OldEvent = event::Unit<T::Revision>;

    fn convert(_: Self::OldEvent) -> OldOrNew<Self::OldEvent, T> {
        panic!("PanicConverter::convert must never be called")
    }
}

#[must_use]
pub fn supported_by_stream<T: StreamDescription>() -> HashSet<T::Revision> {
    let mut new_revisions = T::Event::supported_revisions();
    let old_revisions =
        <T::RevisionConverter as Converter>::OldEvent::supported_revisions();

    let mut intersection = new_revisions.intersection(&old_revisions);
    if let Some(conflicting_revision) = intersection.next() {
        let msg = formatdoc!(
            r#"
                Conflicting revision for stream "{name}" ({desc}): {revision:?}
                
                    The revision above appears twice:
                        In the old event:      {old}
                        And in the new event:  {new}
                        
                Ensure you've set each event variant revision appropriately.
            "#,
            name = T::NAME,
            desc = type_name::<T>(),
            revision = conflicting_revision,
            old = type_name::<<T::RevisionConverter as Converter>::OldEvent>(),
            new = type_name::<T::Event>(),
        );
        panic!("{}", msg);
    }

    new_revisions.extend(old_revisions);
    new_revisions
}
