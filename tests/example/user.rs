use std::collections::HashSet;

use derive_more::Display;
use uuid::Uuid;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub Uuid);

pub struct Desc;

impl occur::StreamDesc for Desc {
    const NAME: &'static str = "user";
    type Id = Id;
    type Event = Event;
    type OldEvent = old::Event;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Created { name: String, is_admin: bool },
    Renamed { new_name: String },
    Befriended { user: Id },
    PromotedToAdmin { by: Id },
    Deactivated { reason: String },
}

impl occur::Event for Event {
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

impl occur::Entity<Desc> for Entity {
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

    fn fold(mut self, event: Event) -> Self {
        match event {
            Event::Created { .. } => self,

            Event::Renamed { new_name } => Entity { name: new_name, ..self },

            Event::Befriended { user } => {
                if !self.friends.contains(&user) {
                    self.friends.push(user);
                }
                self
            }

            Event::PromotedToAdmin { by: admin_id } => {
                if !self.is_admin {
                    self.is_admin = true;
                    self.promoted_to_admin_by = Some(admin_id);
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

pub mod old {
    use std::collections::HashSet;

    use occur::revision;
    use occur::revision::OldOrNew;

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Event {
        Deactivated_V0,
    }

    impl occur::Event for Event {
        fn supported_revisions() -> HashSet<Self::Revision> {
            HashSet::from([Self::Revision::new("Deactivated", 0)])
        }

        fn revision(&self) -> Self::Revision {
            match &self {
                Event::Deactivated_V0 => Self::Revision::new("Deactivated", 0),
            }
        }
    }

    impl revision::Convert for Event {
        type NewEvent = super::Event;

        fn convert(self) -> OldOrNew<Self, Self::NewEvent> {
            match self {
                Self::Deactivated_V0 => {
                    OldOrNew::New(Self::NewEvent::Deactivated {
                        reason: "".to_owned(),
                    })
                }
            }
        }
    }
}
