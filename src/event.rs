use std::collections::HashSet;
use std::hash::Hash;

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

pub trait Streamable: Event {
    const STREAM_NAME: &'static str;
    type Id: Clone + Eq + Hash;

    type OldEvent: revision::Convert<NewEvent = Self> = revision::Never<Self>;

    /// TODO doc
    #[must_use]
    fn convertible_revisions() -> HashSet<Self::Revision>
    where
        Self::OldEvent: Event<Revision = Self::Revision>,
    {
        let mut new_revisions = Self::supported_revisions();
        let old_revisions = Self::OldEvent::supported_revisions();

        let mut intersection = new_revisions.intersection(&old_revisions);
        if let Some(conflicting_revision) = intersection.next() {
            let panic_msg = indoc::formatdoc!(
                r#"
                Conflicting revision in definition of {self_type_name}.
    
                The same revision appears in both old and new event types.
    
                    Revision = {conflicting_revision:?}
                    OldEvent = {old_event_type_name}
                    NewEvent = {new_event_type_name}
    
                Ensure you've set the revision of each event appropriately.
                "#,
                self_type_name = std::any::type_name::<Self>(),
                conflicting_revision = conflicting_revision,
                old_event_type_name = std::any::type_name::<Self::OldEvent>(),
                new_event_type_name = std::any::type_name::<Self>(),
            );
            panic!("{}", panic_msg);
        }

        new_revisions.extend(old_revisions);
        new_revisions
    }
}
