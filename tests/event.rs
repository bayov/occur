#![feature(assert_matches)]

use event_sourcing::{SequenceNumber, Stream, Time};

use crate::example::{event, Id};

mod example;

#[test]
fn record_event_in_stream() {
    let admin_id = 42 as Id;
    let mut admin_stream = Stream::new(admin_id);

    let before = Time::now();

    let admin_created = admin_stream
        .record(event::User::Created { name: "admin".to_owned(), admin: true });

    let after = Time::now();

    assert_eq!(admin_created.id(), &admin_id);
    assert_eq!(admin_created.sequence_number, SequenceNumber(0));
    assert!(admin_created.time() >= before);
    assert!(admin_created.time() <= after);
    assert_eq!(admin_created.event(), &event::User::Created {
        name: "admin".to_owned(),
        admin: true,
    });
}

#[test]
fn record_many_events_in_stream() {
    let admin_id = 42 as Id;
    let user_id = 43 as Id;
    let mut admin_stream = Stream::new(admin_id);
    let mut user_stream = Stream::new(user_id);

    let admin_created = admin_stream
        .record(event::User::Created { name: "admin".to_owned(), admin: true });

    let before = Time::now();

    user_stream.record_array([
        event::User::Created { name: "aki".to_owned(), admin: false },
        event::User::Renamed { new_name: "bayov".to_owned() },
        event::User::Befriended { user: admin_created.refer() },
        event::User::PromotedToAdmin { by: admin_created.refer() },
        event::User::Deactivated,
    ]);

    let after = Time::now();

    assert_eq!(
        user_stream.sequence_numbers_range(),
        SequenceNumber(0)..SequenceNumber(5)
    );

    for event in user_stream.events() {
        assert!(event.time >= before);
        assert!(event.time <= after);
    }
}
