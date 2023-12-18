use std::collections::HashSet;

use event_sourcing::revision::OldOrNew;

use crate::example::user::Event as NewEvent;

pub enum OldEvent {
    Deactivated,
}

impl event_sourcing::Event for OldEvent {
    fn supported_revisions() -> HashSet<Self::Revision> {
        HashSet::from([Self::Revision::new("Deactivated", 0)])
    }

    fn revision(&self) -> Self::Revision {
        match &self {
            OldEvent::Deactivated => Self::Revision::new("Deactivated", 0),
        }
    }
}

pub struct RevisionConverter;

impl event_sourcing::revision::Converter for RevisionConverter {
    type NewEvent = NewEvent;
    type OldEvent = OldEvent;

    fn convert(old_event: OldEvent) -> OldOrNew<OldEvent, NewEvent> {
        match old_event {
            OldEvent::Deactivated => {
                OldOrNew::New(NewEvent::Deactivated { reason: "".to_owned() })
            }
        }
    }
}
