#![allow(dead_code)]
#![feature(assert_matches)]

use std::collections::HashSet;

use occur::revision::{Convert, OldOrNew};
use occur::{revision, Event, Revision};

use crate::example::user;

mod example;

#[test]
fn convert_old_event() {
    let old_event = user::old::Event::Deactivated_V0;
    let new_event = old_event.convert_until_new();
    assert_eq!(new_event, user::Event::Deactivated { reason: "".to_owned() });
}

#[test]
fn supported_and_convertible_revisions() {
    assert_eq!(
        user::Event::supported_revisions(),
        HashSet::from([
            revision::Pair::new("Created", 0),
            revision::Pair::new("Renamed", 0),
            revision::Pair::new("Befriended", 0),
            revision::Pair::new("PromotedToAdmin", 0),
            revision::Pair::new("Deactivated", 1),
        ])
    );

    assert_eq!(
        user::old::Event::supported_revisions(),
        HashSet::from([revision::Pair::new("Deactivated", 0)])
    );

    assert_eq!(
        user::Event::convertible_revisions(),
        HashSet::from([
            // new revisions
            revision::Pair::new("Created", 0),
            revision::Pair::new("Renamed", 0),
            revision::Pair::new("Befriended", 0),
            revision::Pair::new("PromotedToAdmin", 0),
            revision::Pair::new("Deactivated", 1),
            // old revisions
            revision::Pair::new("Deactivated", 0),
        ])
    );
}

#[test]
#[should_panic]
fn panics_when_old_and_new_event_revisions_intersect() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SomeEvent {
        Foo,
        // has the same revision as SomeOldEvent::Foo_V1
        Bar,
    }

    impl Event for SomeEvent {
        const STREAM_NAME: &'static str = "some_stream";
        type Id = u32;
        type OldEvent = SomeOldEvent;
    }

    impl Revision for SomeEvent {
        fn revision(&self) -> Self::Revision { unreachable!() }

        fn supported_revisions() -> HashSet<Self::Revision> {
            HashSet::from([
                Self::Revision::new("Foo", 1),
                Self::Revision::new("Bar", 0),
            ])
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum SomeOldEvent {
        Foo_V0,
        Foo_V1,
    }

    impl Revision for SomeOldEvent {
        fn revision(&self) -> Self::Revision { unreachable!() }

        fn supported_revisions() -> HashSet<Self::Revision> {
            HashSet::from([
                Self::Revision::new("Foo", 0),
                Self::Revision::new("Foo", 1),
            ])
        }
    }

    impl Convert for SomeOldEvent {
        type NewEvent = SomeEvent;
        fn convert(self) -> OldOrNew<Self, Self::NewEvent> {
            match self {
                Self::Foo_V0 => OldOrNew::Old(Self::Foo_V1),
                Self::Foo_V1 => OldOrNew::New(Self::NewEvent::Foo),
            }
        }
    }

    let _ = SomeEvent::convertible_revisions();
}
