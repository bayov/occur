use std::fmt::Debug;
use std::hash::Hash;

use crate::Event;

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
}
