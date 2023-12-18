use std::collections::HashSet;

use derive_more::Display;

use crate::example;
use crate::example::old_revision;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub example::Id);

pub struct StreamDescription;

impl event_sourcing::StreamDescription for StreamDescription {
    const NAME: &'static str = "user";
    type Id = Id;
    type Event = Event;
}

pub type Stream = event_sourcing::Stream<StreamDescription>;
pub type Ref = event_sourcing::Ref<StreamDescription>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Created { name: String, is_admin: bool },
    Renamed { new_name: String },
    Befriended { user: Id },
    PromotedToAdmin { by: Ref },
    Deactivated { reason: String },
}

impl event_sourcing::Event for Event {
    fn supported_revisions() -> HashSet<Self::Revision> {
        HashSet::from([
            Self::Revision::new("Created", 0),
            Self::Revision::new("Renamed", 0),
            Self::Revision::new("Befriended", 0),
            Self::Revision::new("PromotedToAdmin", 0),
            Self::Revision::new("Deactivated", 1),
        ])
    }

    fn revision(&self) -> Self::Revision {
        match &self {
            Event::Created { .. } => Self::Revision::new("Created", 0),
            Event::Renamed { .. } => Self::Revision::new("Renamed", 0),
            Event::Befriended { .. } => Self::Revision::new("Befriended", 0),
            Event::PromotedToAdmin { .. } => {
                Self::Revision::new("PromotedToAdmin", 0)
            }
            Event::Deactivated { .. } => Self::Revision::new("Deactivated", 1),
        }
    }
}

impl event_sourcing::ConvertFromOldRevision for Event {
    type OldEvent = old_revision::user::OldEvent;
    type RevisionConverter = old_revision::user::RevisionConverter;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: Id,
    pub name: String,
    pub is_admin: bool,
    pub promoted_to_admin_by: Option<Id>,
    pub friends: Vec<Id>,
    pub is_deactivated: bool,
    pub deactivation_reason: Option<String>,
}

impl event_sourcing::Entity<StreamDescription> for Entity {
    fn new(id: Id, event: Event) -> Option<Self> {
        match event {
            Event::Created { name, is_admin } => Some(Entity {
                id,
                name,
                is_admin,
                promoted_to_admin_by: None,
                friends: Vec::default(),
                is_deactivated: false,
                deactivation_reason: None,
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

            Event::Deactivated { reason } => {
                if !self.is_deactivated {
                    self.is_deactivated = true;
                    self.deactivation_reason = Some(reason);
                }
                self
            }
        }
    }
}
