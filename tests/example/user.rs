use derive_more::Display;

use event_sourcing::stream_descriptor;

use crate::example;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub example::Id);

stream_descriptor! {
    name = "user";
    type Id = Id;
    type Event = Event;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Created { name: String, admin: bool },
    Renamed { new_name: String },
    Befriended { user: Ref },
    PromotedToAdmin { by: Ref },
    Deactivated,
}

impl event_sourcing::Event for Event {}
