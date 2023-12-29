#![allow(dead_code)]
#![feature(assert_matches)]

use std::collections::HashSet;

use occur::revision::{Convert, OldOrNew};
use occur::{Event, Revision};

use crate::example::user;

mod example;

#[test]
fn convert_old_event() {
    let old_event = user::old::Revision::Deactivated_V0;
    let new_event = old_event.convert_until_new();
    assert_eq!(new_event, user::Event::Deactivated { reason: "".to_owned() });
}

#[test]
fn available_and_supported_revisions() {
    assert_eq!(
        user::Event::revision_set(),
        HashSet::from([
            ("Created", 0),
            ("Renamed", 0),
            ("Befriended", 0),
            ("PromotedToAdmin", 0),
            ("Deactivated", 1),
        ])
    );

    assert_eq!(
        user::old::Revision::revision_set(),
        HashSet::from([("Deactivated", 0)])
    );

    assert_eq!(
        user::Event::supported_revisions(),
        HashSet::from([
            // new revisions
            ("Created", 0),
            ("Renamed", 0),
            ("Befriended", 0),
            ("PromotedToAdmin", 0),
            ("Deactivated", 1),
            // old revisions
            ("Deactivated", 0),
        ])
    );
}

#[test]
#[should_panic]
fn panics_when_revisions_intersect() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SomeEvent {
        Foo,
        // has the same revision as SomeOldEvent::Foo_V1
        Bar,
    }

    impl Event for SomeEvent {
        type StreamId = u32;
        type OldRevision = SomeOldEvent;
    }

    impl Revision for SomeEvent {
        fn revision(&self) -> Self::Value { unreachable!() }

        fn revision_set() -> HashSet<Self::Value> {
            HashSet::from([("Foo", 1), ("Bar", 0)])
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SomeOldEvent {
        Foo_V0,
        Foo_V1,
    }

    impl Revision for SomeOldEvent {
        fn revision(&self) -> Self::Value { unreachable!() }

        fn revision_set() -> HashSet<Self::Value> {
            HashSet::from([("Foo", 0), ("Foo", 1)])
        }
    }

    impl Convert for SomeOldEvent {
        type New = SomeEvent;
        fn convert(self) -> OldOrNew<Self, Self::New> {
            match self {
                Self::Foo_V0 => OldOrNew::Old(Self::Foo_V1),
                Self::Foo_V1 => OldOrNew::New(Self::New::Foo),
            }
        }
    }

    let _ = SomeEvent::supported_revisions();
}
