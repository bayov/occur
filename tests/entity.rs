#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use event_sourcing::Entity;

use crate::example::user;

mod example;

#[test]
fn new_entity_with_non_creation_event_returns_none() {
    let admin = user::Entity::new(user::Id(42), user::Event::Renamed {
        new_name: "admin".to_owned(),
    });

    assert_matches!(admin, None);
}

#[test]
fn new_entity() {
    let admin = user::Entity::new(user::Id(42), user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });

    assert_eq!(
        admin,
        Some(user::Entity {
            id: user::Id(42),
            name: "admin".to_owned(),
            is_admin: true,
            promoted_to_admin_by: None,
            friends: Vec::default(),
            is_deactivated: false,
        })
    );

    let admin = admin.unwrap();

    // a second creation event is ignored
    let next_admin = admin.clone().apply(user::Event::Created {
        name: "another_user".to_owned(),
        is_admin: false,
    });

    assert_eq!(next_admin, admin);
}
