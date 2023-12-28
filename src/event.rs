use std::collections::HashSet;
use std::marker::PhantomData;

use crate::{revision, Revision};

/// An event that can be committed to a stream to represent an immutable fact.
pub trait Event: Clone {
    /// The type used to revision event variants.
    ///
    /// By default, this is [`revision::Pair`].
    ///
    /// For event revisioning documentation, see module [`revision`].
    type Revision: Revision = revision::Pair;

    fn supported_revisions() -> HashSet<Self::Revision>;

    fn revision(&self) -> Self::Revision;
}

/// Represents an event with no variants.
///
/// Used as the default [`StreamDesc::OldEvent`] type, indicating there are no
/// existing old event variants for the described stream.
pub struct Empty<R: Revision>(!, PhantomData<R>);

impl<R: Revision> Clone for Empty<R> {
    fn clone(&self) -> Self { unreachable!() }
}

impl<R: Revision> Event for Empty<R> {
    type Revision = R;
    fn supported_revisions() -> HashSet<Self::Revision> { HashSet::default() }
    fn revision(&self) -> Self::Revision { unreachable!() }
}
