use crate::Id;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct EventAlreadyExists<ID: Id> {
    pub entity_kind: &'static str,
    pub id: ID,
}

impl<ID: Id> EntityAlreadyExists<ID> {
    pub fn new<T: Entity>(id: ID) -> Self {
        Self { entity_kind: T::kind(), id }
    }
}

impl<ID: Id> Error for EntityAlreadyExists<ID> {}

impl<ID: Id> fmt::Display for EntityAlreadyExists<ID> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{} already exists", self.entity_kind, self.id)
    }
}

#[derive(Debug)]
pub struct EntityDoesntExist<ID: Id> {
    pub entity_kind: &'static str,
    pub id: ID,
}

impl<ID: Id> EntityDoesntExist<ID> {
    pub fn new<T: Entity>(id: ID) -> Self {
        Self { entity_kind: T::kind(), id }
    }
}

impl<ID: Id> Error for EntityDoesntExist<ID> {}

impl<ID: Id> fmt::Display for EntityDoesntExist<ID> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{} doesn't exist", self.entity_kind, self.id)
    }
}

#[derive(Debug)]
pub struct EventDoesntExist<ID: Id> {
    pub entity_kind: &'static str,
    pub entity_id: ID,
    pub version: Version,
}

impl<ID: Id> EventDoesntExist<ID> {
    pub fn new<T: Entity>(entity_id: ID, version: Version) -> Self {
        Self { entity_kind: T::kind(), entity_id, version }
    }
}

impl<ID: Id> Error for EventDoesntExist<ID> {}

impl<ID: Id> fmt::Display for EventDoesntExist<ID> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (kind, id, v) = (self.entity_kind, &self.entity_id, self.version);
        write!(f, "{kind}/{id}/event/{v} doesn't exist")
    }
}

#[derive(Debug)]
pub struct InconsistentSave<ID: Id, T: Entity> {
    pub entity_id: ID,
    pub current_version: Version,
    pub discarded_event: RecordedModificationEvent<ID, T>,
}

impl<ID: Id, T: Entity> Error for InconsistentSave<ID, T> {}

impl<ID: Id, T: Entity> fmt::Display for InconsistentSave<ID, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "attempted to save an event that does not immediately succeed \
             the current version {} of {}/{}; \
             discarded event: {}",
            self.current_version,
            T::kind(),
            self.entity_id,
            self.discarded_event,
        ))
    }
}
