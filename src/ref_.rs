use std::fmt::{Debug, Formatter};

use crate::{RecordedEvent, StreamDescription, Version};

pub struct Ref<T: StreamDescription> {
    pub id: T::Id,
    pub version: Version,
}

impl<T: StreamDescription> Ref<T> {
    #[must_use]
    pub const fn new(id: T::Id, version: Version) -> Self {
        Self { id, version }
    }
}

impl<T: StreamDescription> From<&RecordedEvent<T>> for Ref<T> {
    fn from(r: &RecordedEvent<T>) -> Self {
        Self { id: r.id.clone(), version: r.version }
    }
}

impl<T: StreamDescription> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self { id: self.id.clone(), version: self.version }
    }
}

impl<T: StreamDescription> Debug for Ref<T>
where
    T::Id: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = &self.id;
        let version = &self.version;
        write!(f, "Ref {{ id = {id:?}, version = {version:?} }}")
    }
}

impl<T: StreamDescription> PartialEq for Ref<T>
where
    T::Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.version == other.version
    }
}

impl<T: StreamDescription> Eq for Ref<T> where T::Id: Eq {}
