use std::marker::PhantomData;

use crate::{Id, SequenceNumber};

#[marker]
pub trait Referable {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ref<T: Referable, ID: Id> {
    pub id: ID,
    pub sequence_number: SequenceNumber,
    phantom: PhantomData<T>,
}

impl<T: Referable, ID: Id> Ref<T, ID> {
    #[must_use]
    pub const fn new(id: ID, sequence_number: SequenceNumber) -> Self {
        Self { id, sequence_number, phantom: PhantomData }
    }
}
