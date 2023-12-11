use crate::{Event, Id, Referable};

pub trait Entity {
    type Id: Id;
    type Event: Event;
}

impl<T: Entity> Referable for T {}
