use std::collections::HashSet;
use std::intrinsics::type_name;

use crate::{revision, Revision};

pub trait Event {
    type Revision: Revision = revision::TypeAndNumber;

    fn supported_revisions() -> HashSet<Self::Revision>;

    fn revision(&self) -> Self::Revision;
}

pub trait ConvertFromOldRevision: Event {
    type OldEvent: Event<Revision = Self::Revision>;
    type RevisionConverter: revision::Converter<
        OldEvent = Self::OldEvent,
        NewEvent = Self,
    >;

    #[must_use]
    fn supported_revisions() -> HashSet<Self::Revision> {
        let mut new_revisions = <Self as Event>::supported_revisions();
        let old_revisions = Self::OldEvent::supported_revisions();

        let mut intersection = new_revisions.intersection(&old_revisions);
        if let Some(conflicting_revision) = intersection.next() {
            let panic_msg = indoc::formatdoc!(
                r#"
                Conflicting revision in definition of {self_type_name}.
                
                The same revision appears in both old and new event types.
                
                    Revision:        {conflicting_revision:?}
                    Old event type:  {old_type_name}
                    New event type:  {self_type_name}

                Ensure you've set the revision of each event appropriately.
                "#,
                conflicting_revision = conflicting_revision,
                old_type_name = type_name::<Self::OldEvent>(),
                self_type_name = type_name::<Self>(),
            );
            panic!("{}", panic_msg);
        }

        new_revisions.extend(old_revisions);
        new_revisions
    }
}
