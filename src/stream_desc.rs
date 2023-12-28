use std::hash::Hash;

use crate::event::Empty;
use crate::{revision, Event};

pub trait StreamDesc {
    const NAME: &'static str;
    type Id: Clone + Eq + Hash;
    type Event: Event;

    type OldEvent: Event<Revision = <Self::Event as Event>::Revision> =
        Empty<<Self::Event as Event>::Revision>;

    type RevisionConverter: revision::Converter<Self::OldEvent, Self::Event> =
        revision::EmptyConverter;
}
