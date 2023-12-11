use event_sourcing::Event;

use crate::example::Ref;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum User {
    Created { name: String, admin: bool },
    Renamed { new_name: String },
    Befriended { user: Ref<User> },
    PromotedToAdmin { by: Ref<User> },
    Deactivated,
}

impl Event for User {}
