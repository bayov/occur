#![feature(assert_matches)]

use std::collections::HashSet;

use event_sourcing::revision::Converter;
use event_sourcing::{revision, CommitNumber, Event, Time};
use rstest::rstest;

use crate::example::{old_revision, user};
use crate::fixture::user::{admin_created, admin_id, admin_stream};

mod example;
mod fixture;

#[test]
fn create_stream() {
    let id = user::Id(12);
    let stream = user::Stream::new(id);

    assert_eq!(stream.id(), &id);
    assert_eq!(stream.commit_numbers_range(), CommitNumber(0)..CommitNumber(0));
    assert_eq!(stream.events(), vec![]);
}

#[rstest]
fn commit_event_in_stream(admin_id: user::Id, mut admin_stream: user::Stream) {
    let before = Time::now();

    let admin_created = admin_stream.commit(user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });

    let after = Time::now();

    assert_eq!(admin_created.id, admin_id);
    assert_eq!(admin_created.commit_number, CommitNumber(0));
    assert!(admin_created.time >= before);
    assert!(admin_created.time <= after);
    assert_eq!(admin_created.event, user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });
}

#[rstest]
fn commit_many_events_in_stream(admin_created: user::Event) {
    let admin_id = user::Id(42);
    let mut admin_stream = user::Stream::new(admin_id);
    let admin_created = admin_stream.commit(admin_created);

    let user_id = user::Id(43);
    let mut user_stream = user::Stream::new(user_id);

    let before = Time::now();

    user_stream.commit_array([
        user::Event::Created { name: "aki".to_owned(), is_admin: false },
        user::Event::Renamed { new_name: "bayov".to_owned() },
        user::Event::Befriended { user: admin_id },
        user::Event::PromotedToAdmin { by: admin_created.into() },
        user::Event::Deactivated { reason: "bot".to_owned() },
    ]);

    let after = Time::now();

    assert_eq!(
        user_stream.commit_numbers_range(),
        CommitNumber(0)..CommitNumber(5)
    );

    for event in user_stream.events() {
        assert!(event.time >= before);
        assert!(event.time <= after);
    }
}

#[test]
fn convert_old_event() {
    let old_event = old_revision::user::OldEvent::Deactivated_V0;
    let new_event =
        old_revision::user::RevisionConverter::convert_to_new(old_event);
    assert_eq!(new_event, user::Event::Deactivated { reason: "".to_owned() });
}

#[test]
fn supported_revisions() {
    assert_eq!(
        <user::Event as Event>::supported_revisions(),
        HashSet::from([
            revision::TypeAndNumber::new("Created", 0),
            revision::TypeAndNumber::new("Renamed", 0),
            revision::TypeAndNumber::new("Befriended", 0),
            revision::TypeAndNumber::new("PromotedToAdmin", 0),
            revision::TypeAndNumber::new("Deactivated", 1),
        ])
    );

    assert_eq!(
        old_revision::user::OldEvent::supported_revisions(),
        HashSet::from([revision::TypeAndNumber::new("Deactivated", 0)])
    );

    type Converter = <
    user::StreamDescription as event_sourcing::StreamDescription
    >::RevisionConverter;

    assert_eq!(
        <Converter as revision::Converter>::supported_revisions(),
        HashSet::from([
            // new revisions
            revision::TypeAndNumber::new("Created", 0),
            revision::TypeAndNumber::new("Renamed", 0),
            revision::TypeAndNumber::new("Befriended", 0),
            revision::TypeAndNumber::new("PromotedToAdmin", 0),
            revision::TypeAndNumber::new("Deactivated", 1),
            // old revisions
            revision::TypeAndNumber::new("Deactivated", 0),
        ])
    );
}
