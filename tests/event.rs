#![feature(assert_matches)]

use event_sourcing::{Time, Version};

use crate::example::user;

mod example;

#[test]
fn record_event_in_stream() {
    let admin_id = user::Id(42);
    let mut admin_stream = user::Stream::new(admin_id);

    let before = Time::now();

    let admin_created = admin_stream.record(user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });

    let after = Time::now();

    assert_eq!(admin_created.id, admin_id);
    assert_eq!(admin_created.version, Version(0));
    assert!(admin_created.time >= before);
    assert!(admin_created.time <= after);
    assert_eq!(admin_created.event, user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });
}

#[test]
fn record_many_events_in_stream() {
    let admin_id = user::Id(42);
    let mut admin_stream = user::Stream::new(admin_id);
    let admin_created = admin_stream.record(user::Event::Created {
        name: "admin".to_owned(),
        is_admin: true,
    });

    let user_id = user::Id(43);
    let mut user_stream = user::Stream::new(user_id);

    let before = Time::now();

    user_stream.record_array([
        user::Event::Created { name: "aki".to_owned(), is_admin: false },
        user::Event::Renamed { new_name: "bayov".to_owned() },
        user::Event::Befriended { user: admin_id },
        user::Event::PromotedToAdmin { by: admin_created.into() },
        user::Event::Deactivated,
    ]);

    let after = Time::now();

    assert_eq!(user_stream.versions_range(), Version(0)..Version(5));

    for event in user_stream.events() {
        assert!(event.time >= before);
        assert!(event.time <= after);
    }
}
