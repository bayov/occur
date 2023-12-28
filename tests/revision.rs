#![feature(assert_matches)]

use std::collections::HashSet;

use occur::revision::Converter;
use occur::{revision, Event, StreamDesc};

use crate::example::user;

mod example;

#[test]
fn convert_old_event() {
    let old_event = user::old::Event::Deactivated_V0;
    let new_event = user::old::RevisionConverter::convert_until_new(old_event);
    assert_eq!(new_event, user::Event::Deactivated { reason: "".to_owned() });
}

#[test]
fn supported_revisions() {
    assert_eq!(
        <user::Event as Event>::supported_revisions(),
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

    type Converter = <user::Desc as StreamDesc>::RevisionConverter;

    assert_eq!(
        Converter::supported_revisions(),
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
