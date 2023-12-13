use derive_more::Display;

use event_sourcing::stream_descriptor;

use crate::example;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub example::Id);

stream_descriptor! {
    const NAME = "user";
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

pub struct Entity {
    pub id: Id,
    pub name: String,
    pub is_admin: bool,
    pub promoted_to_admin_by: Option<Id>,
    pub friends: Vec<Id>,
    pub is_deactivated: bool,
}

impl event_sourcing::Entity for Entity {
    type Id = Id;
    type Event = Event;
}
