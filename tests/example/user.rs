use std::time::SystemTime;

use derive_more::Display;

use crate::example;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub example::Id);

pub struct StreamDescriptor;

impl event_sourcing::StreamDescriptor for StreamDescriptor {
    const NAME: &'static str = "user";
    type Id = Id;
    type Time = SystemTime;
    type Event = Event;
}

pub type Stream = event_sourcing::Stream<StreamDescriptor>;
pub type Ref = event_sourcing::Ref<StreamDescriptor>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Created { name: String, is_admin: bool },
    Renamed { new_name: String },
    Befriended { user: Id },
    PromotedToAdmin { by: Ref },
    Deactivated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: Id,
    pub name: String,
    pub is_admin: bool,
    pub promoted_to_admin_by: Option<Id>,
    pub friends: Vec<Id>,
    pub is_deactivated: bool,
}

impl event_sourcing::Entity<StreamDescriptor> for Entity {
    fn new(id: Id, event: Event) -> Option<Self> {
        match event {
            Event::Created { name, is_admin } => Some(Entity {
                id,
                name,
                is_admin,
                promoted_to_admin_by: None,
                friends: Vec::default(),
                is_deactivated: false,
            }),
            _ => None,
        }
    }

    fn apply(mut self, event: Event) -> Self {
        match event {
            Event::Created { .. } => self,

            Event::Renamed { new_name } => Entity { name: new_name, ..self },

            Event::Befriended { user } => {
                if !self.friends.contains(&user) {
                    self.friends.push(user);
                }
                self
            }

            Event::PromotedToAdmin { by: admin } => {
                if !self.is_admin {
                    self.is_admin = true;
                    self.promoted_to_admin_by = Some(admin.id);
                }
                self
            }

            Event::Deactivated => {
                if !self.is_deactivated {
                    self.is_deactivated = true;
                }
                self
            }
        }
    }
}
