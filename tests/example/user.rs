use std::time::SystemTime;

use derive_more::Display;

use event_sourcing::stream_descriptor;

use crate::example;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub example::Id);

stream_descriptor! {
    const NAME = "user";
    type Id = Id;
    type Time = SystemTime;
    type Event = Event;
    type Error = Error;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Created { name: String, is_admin: bool },
    Renamed { new_name: String },
    Befriended { user: Id },
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

pub enum Error {
    NonCreationEvent,
    AlreadyCreated,
    AlreadyFriends,
    AlreadyAdmin,
    AlreadyDeactivated,
}

impl event_sourcing::Entity<StreamDescriptor> for Entity {
    fn new(id: Id, event: Event) -> Result<Self, Error> {
        match event {
            Event::Created { name, is_admin } => Ok(Entity {
                id,
                name,
                is_admin,
                promoted_to_admin_by: None,
                friends: Vec::default(),
                is_deactivated: false,
            }),
            _ => Err(Error::NonCreationEvent),
        }
    }

    fn apply(mut self, event: Event) -> Result<Self, Error> {
        match event {
            Event::Created { .. } => Err(Error::AlreadyCreated),

            Event::Renamed { new_name } => {
                Ok(Entity { name: new_name, ..self })
            }

            Event::Befriended { user } => {
                if self.friends.contains(&user) {
                    Err(Error::AlreadyFriends)
                } else {
                    self.friends.push(user);
                    Ok(self)
                }
            }

            Event::PromotedToAdmin { by: admin } => {
                if self.is_admin {
                    Err(Error::AlreadyAdmin)
                } else {
                    Ok(Entity {
                        is_admin: true,
                        promoted_to_admin_by: Some(admin.id),
                        ..self
                    })
                }
            }

            Event::Deactivated => {
                if self.is_deactivated {
                    Err(Error::AlreadyDeactivated)
                } else {
                    Ok(Entity { is_deactivated: true, ..self })
                }
            }
        }
    }
}
