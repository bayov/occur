use std::collections::HashSet;

use event_sourcing::revision::OldOrNew;

use crate::example::user::Event as NewEvent;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OldEvent {
    Deactivated_V0,
}

impl event_sourcing::Event for OldEvent {
    fn supported_revisions() -> HashSet<Self::Revision> {
        HashSet::from([Self::Revision::new("Deactivated", 0)])
    }

    fn revision(&self) -> Self::Revision {
        match &self {
            OldEvent::Deactivated_V0 => Self::Revision::new("Deactivated", 0),
        }
    }
}

pub struct RevisionConverter;

impl event_sourcing::revision::Converter for RevisionConverter {
    type OldEvent = OldEvent;
    type NewEvent = NewEvent;

    fn convert(old_event: OldEvent) -> OldOrNew<OldEvent, NewEvent> {
        match old_event {
            OldEvent::Deactivated_V0 => {
                OldOrNew::New(NewEvent::Deactivated { reason: "".to_owned() })
            }
        }
    }
}
