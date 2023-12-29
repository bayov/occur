use std::collections::HashSet;

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
