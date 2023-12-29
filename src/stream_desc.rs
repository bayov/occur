use std::collections::HashSet;
use std::hash::Hash;

use crate::{revision, Event};

pub trait StreamDesc {
    const NAME: &'static str;
    type Id: Clone + Eq + Hash;
    type Event: Event;

    type OldEvent: revision::Convert<
        Revision = <Self::Event as Event>::Revision,
        NewEvent = Self::Event,
    > = revision::Never<Self::Event>;

    /// TODO doc
    #[must_use]
    fn supported_revisions() -> HashSet<<Self::Event as Event>::Revision> {
        let mut new_revisions = Self::Event::supported_revisions();
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
                new_event_type_name = std::any::type_name::<Self::Event>(),
            );
            panic!("{}", panic_msg);
        }

        new_revisions.extend(old_revisions);
        new_revisions
    }
}
