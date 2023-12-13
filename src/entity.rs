use crate::{Event, Id};

pub trait Entity {
    type Id: Id;
    type Event: Event;
}
