use std::collections::HashSet;
use std::marker::PhantomData;

use crate::{revision, Revision};

pub trait Event: Clone {
    type Revision: Revision = revision::TypeAndNumber;

    fn supported_revisions() -> HashSet<Self::Revision>;

    fn revision(&self) -> Self::Revision;
}

#[derive(Clone)]
pub struct Empty<R: Revision>(PhantomData<R>);

impl<R: Revision> Event for Empty<R> {
    type Revision = R;

    fn supported_revisions() -> HashSet<Self::Revision> { HashSet::default() }

    fn revision(&self) -> Self::Revision {
        panic!("event::Empty::revision() should not be called")
    }
}
