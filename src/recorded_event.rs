use std::fmt::{Debug, Formatter};

use crate::{StreamDescriptor, Version};

pub struct RecordedEvent<T: StreamDescriptor> {
    pub id: T::Id,
    pub version: Version,
    pub time: T::Time,
    pub event: T::Event,
}

impl<T: StreamDescriptor> Clone for RecordedEvent<T>
where
    T::Time: Clone,
    T::Event: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            version: self.version,
            time: self.time.clone(),
            event: self.event.clone(),
        }
    }
}

impl<T: StreamDescriptor> PartialEq for RecordedEvent<T>
where
    T::Id: PartialEq,
    T::Time: PartialEq,
    T::Event: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.version == other.version
            && self.time == other.time
            && self.event == other.event
    }
}

impl<T: StreamDescriptor> Eq for RecordedEvent<T>
where
    T::Id: PartialEq,
    T::Time: PartialEq,
    T::Event: PartialEq,
{
}

#[allow(clippy::missing_fields_in_debug)] // false positive
impl<T: StreamDescriptor> Debug for RecordedEvent<T>
where
    T::Id: Debug,
    T::Time: Debug,
    T::Event: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct(&format!(r#"RecordedEvent<"{}">"#, T::NAME))
            .field_with("id", |f| write!(f, "{:?}", self.id))
            .field("version", &self.version.0)
            .field_with("time", |f| write!(f, "{:?}", self.time))
            .field("event", &self.event)
            .finish()
    }
}
