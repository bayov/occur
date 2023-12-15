#![allow(clippy::derive_partial_eq_without_eq)] // false positive

use std::fmt::{Debug, Formatter};

use impl_tools::autoimpl;

use crate::{RecordedEvent, StreamDescription, Version};

#[autoimpl(Clone where T::Event: Clone)]
#[autoimpl(PartialEq where T::Event: PartialEq)]
#[autoimpl(Eq where T::Event: Eq)]
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

#[allow(clippy::missing_fields_in_debug)] // false positive
impl<T: StreamDescription> Debug for Ref<T>
where
    T::Id: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!(r#"Ref<"{}">"#, T::NAME))
            .field_with("id", |f| write!(f, "{:?}", self.id))
            .field("version", &self.version.0)
            .finish()
    }
}
