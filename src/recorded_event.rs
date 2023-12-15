use std::fmt::{Debug, Formatter};

use impl_tools::autoimpl;

use crate::{StreamDescription, Version};

#[autoimpl(Clone where T::Event: Clone)]
#[autoimpl(PartialEq where T::Event: PartialEq)]
#[autoimpl(Eq where T::Event: Eq)]
pub struct RecordedEvent<T: StreamDescription> {
    pub id: T::Id,
    pub version: Version,
    pub time: T::Time,
    pub event: T::Event,
}

#[allow(clippy::missing_fields_in_debug)] // false positive
impl<T: StreamDescription> Debug for RecordedEvent<T>
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
