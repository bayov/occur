#![allow(clippy::derive_partial_eq_without_eq)] // false positive

use std::fmt::{Debug, Formatter};

use impl_tools::autoimpl;

use crate::{CommitNumber, CommittedEvent, StreamDescription};

#[autoimpl(Clone where T::Event: Clone)]
#[autoimpl(PartialEq where T::Event: PartialEq)]
#[autoimpl(Eq where T::Event: Eq)]
pub struct Ref<T: StreamDescription> {
    pub id: T::Id,
    pub commit_number: CommitNumber,
}

impl<T: StreamDescription> Ref<T> {
    #[must_use]
    pub const fn new(id: T::Id, commit_number: CommitNumber) -> Self {
        Self { id, commit_number }
    }
}

impl<T: StreamDescription> From<&CommittedEvent<T>> for Ref<T> {
    fn from(r: &CommittedEvent<T>) -> Self {
        Self { id: r.id.clone(), commit_number: r.commit_number }
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
            .field("commit_number", &self.commit_number.0)
            .finish()
    }
}
