use std::hash::Hash;

use crate::{revision, Event};

pub trait StreamDesc {
    const NAME: &'static str;
    type Id: Clone + Eq + Hash;
    type Event: Event;

    type RevisionConverter: revision::Converter<NewEvent = Self::Event> =
        revision::EmptyConverter<Self::Event>;
}
