use std::collections::HashSet;
use std::hash::Hash;

use crate::{revision, Revision};

/// An event that can be committed to an event stream.
pub trait Event: Revision {
    /// A name, which together with an [`Self::Id`] value identifies the stream
    /// to which an event belongs to.
    const STREAM_NAME: &'static str;

    /// An ID type that is used to uniquely identify the stream to which an
    /// event belongs to.
    type Id: Clone + Eq + Hash;

    /// A type that holds old revisions that can be converted to `Self`.
    ///
    /// By default, this is [`revision::Empty`], representing that there are no
    /// existing old revisions.
    ///
    /// See [`revision`] module documentation for details about event
    /// revisioning.
    type OldRevision: revision::Convert<New = Self> = revision::Empty<Self>;

    /// Returns the set of all supported revision values, which is the union of
    /// the revisions defined by `Self` and [`Self::OldRevision`].
    ///
    /// # Panics
    ///
    /// When the same revision is defined by both `Self` and
    /// [`Self::OldRevision`].
    #[must_use]
    fn supported_revisions() -> HashSet<Self::Value>
    where
        Self::OldRevision: Revision<Value = Self::Value>,
    {
        let mut new_revisions = Self::revision_set();
        let old_revisions = Self::OldRevision::revision_set();

        let mut intersection = new_revisions.intersection(&old_revisions);
        if let Some(conflicting_revision) = intersection.next() {
            let panic_msg = indoc::formatdoc!(
                r#"
                Conflicting revision value: {conflicting_revision:?}
                
                    Event       = {self_type_name}
                    OldRevision = {old_revision_type_name}
    
                Ensure you've set the revision of each variant appropriately.
                "#,
                self_type_name = std::any::type_name::<Self>(),
                conflicting_revision = conflicting_revision,
                old_revision_type_name =
                    std::any::type_name::<Self::OldRevision>(),
            );
            panic!("{}", panic_msg);
        }

        new_revisions.extend(old_revisions);
        new_revisions
    }
}
