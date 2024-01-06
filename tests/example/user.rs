use std::collections::HashSet;

use derive_more::Display;
use uuid::Uuid;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Display)]
pub struct Id(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Created { name: String, is_admin: bool },
    Renamed { new_name: String },
    Befriended { user: Id },
    PromotedToAdmin { by: Id },
    Deactivated { reason: String },
}

impl occur::Event for Event {
    type StreamId = Id;
    type OldRevision = old::Revision;
}

impl occur::Revision for Event {
    type Value = (&'static str, u8);

    fn revision(&self) -> Self::Value {
        match &self {
            Event::Created { .. } => ("Created", 0),
            Event::Renamed { .. } => ("Renamed", 0),
            Event::Befriended { .. } => ("Befriended", 0),
            Event::PromotedToAdmin { .. } => ("PromotedToAdmin", 0),
            Event::Deactivated { .. } => ("Deactivated", 1),
        }
    }

    fn revision_set() -> HashSet<Self::Value> {
        HashSet::from([
            ("Created", 0),
            ("Renamed", 0),
            ("Befriended", 0),
            ("PromotedToAdmin", 0),
            ("Deactivated", 1),
        ])
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

impl occur::Entity<Event> for Entity {
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

    use crate::example::user::Event;

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Revision {
        Deactivated_V0,
    }

    impl occur::Revision for Revision {
        type Value = (&'static str, u8);

        fn revision(&self) -> Self::Value {
            match &self {
                Revision::Deactivated_V0 => ("Deactivated", 0),
            }
        }

        fn revision_set() -> HashSet<Self::Value> {
            HashSet::from([("Deactivated", 0)])
        }
    }

    impl revision::Convert for Revision {
        type Event = Event;

        fn convert(self) -> OldOrNew<Self::Event> {
            match self {
                Self::Deactivated_V0 => {
                    Event::Deactivated { reason: "".to_owned() }.into()
                }
            }
        }
    }
}
