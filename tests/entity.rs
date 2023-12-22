#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use occur::Entity;
use uuid::Uuid;

use crate::example::user;

mod example;

#[test]
fn new_entity_with_non_creation_event_returns_none() {
    let admin =
        user::Entity::new(user::Id(Uuid::now_v7()), user::Event::Renamed {
            new_name: "admin".to_owned(),
        });

    assert_matches!(admin, None);
}

#[test]
fn new_entity() {
    let admin_id = user::Id(Uuid::now_v7());
    let admin = user::Entity::new(admin_id, user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });

    assert_eq!(
        admin,
        Some(user::Entity {
            id: admin_id,
            name: "admin".to_owned(),
            is_admin: true,
            promoted_to_admin_by: None,
            friends: Vec::default(),
            is_deactivated: false,
            deactivation_reason: None,
        })
    );

    let admin = admin.unwrap();

    // a second creation event is ignored
    let next_admin = admin.clone().fold(user::Event::Created {
        name: "another_user".to_owned(),
        is_admin: false,
    });

    assert_eq!(next_admin, admin);
}
