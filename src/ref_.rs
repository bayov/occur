use crate::{Recorded, StreamDescriptor, Version};

pub struct Ref<T: StreamDescriptor> {
    pub id: T::Id,
    pub version: Version,
}

impl<T: StreamDescriptor> From<Recorded<T>> for Ref<T> {
    fn from(r: Recorded<T>) -> Self { Self { id: r.id, version: r.version } }
}
