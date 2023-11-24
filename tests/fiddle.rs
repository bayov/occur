#![allow(dead_code)]

use crate::TvShowTrackEvents::{Created, WatchedEpisode};
use event_sourcing::repo::{EventIterator, EventSubscription, Repository};
use event_sourcing::{repo, Event, Id, SequenceNumber, Stream};
use futures::task::SpawnExt;
use std::borrow::Cow;

#[derive(Debug, Clone)]
enum TvShowTrackEvents<'s> {
    Created { tv_show_name: Cow<'s, str> },
    WatchedEpisode { season: u64, episode: u64 },
}

impl<'s> Event for TvShowTrackEvents<'s> {}

#[test]
fn fiddle() {
    println!();

    let mut stream = Stream::new(42);
    stream.record(Created { tv_show_name: "Elementary".into() });
    stream.record(WatchedEpisode { season: 1, episode: 1 });
    stream.record(WatchedEpisode { season: 1, episode: 2 });
    stream.record(WatchedEpisode { season: 1, episode: 3 });

    print_stream(&stream);

    println!("-----------------------");

    println!("stream[0] = {:?}", stream[SequenceNumber(0)]);
    println!("stream[2] = {:?}", stream[SequenceNumber(2)]);

    println!("-----------------------");

    let mut repo = repo::fake::Repository::new();
    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        let id = repo.new_id();
        repo.write_event(id, SequenceNumber(0), &e0).await.expect("wtf?");
        repo.write_event(id, SequenceNumber(1), &e1).await.expect("wtf?");
        repo.write_event(id, SequenceNumber(2), &e2).await.expect("wtf?");
        repo.write_event(id, SequenceNumber(3), &e3).await.expect("wtf?");

        let mut it =
            repo.read_stream(&id, SequenceNumber(1)).await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

    futures::executor::block_on(t.unwrap());

    println!("-----------------------");

    let mut repo = repo::fake::Repository::new();
    let id = repo.new_id();
    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    futures::executor::block_on(async {
        repo.write_event(id, SequenceNumber(0), &e0).await.expect("wtf?");
        repo.write_event(id, SequenceNumber(1), &e1).await.expect("wtf?");
    });

    let f1 = async {
        let mut it = repo
            .subscribe_to_stream(&id, SequenceNumber(1))
            .await
            .expect("wtf?");
        while let Some(event) = it.next().await {
            println!("subscriber read {:?}", event);
        }
    };

    let f2 = async {
        repo.write_event(id, SequenceNumber(2), &e2).await.expect("wtf?");
        repo.write_event(id, SequenceNumber(3), &e3).await.expect("wtf?");
    };

    // futures::executor::block_on();

    println!("-----------------------");

    let mut events = stream.take_events();
    events.remove(0);
    let mut stream =
        Stream::from_recorded_events(42, SequenceNumber(1), Cow::Owned(events));
    stream.record(WatchedEpisode { season: 1, episode: 4 });

    print_stream(&stream);

    println!();
}

fn print_stream<ID: Id, T: Event>(stream: &Stream<ID, T>) {
    println!(
        "Stream {{ id: {:?}, range: {}..{} }}",
        stream.id(),
        stream.sequence_numbers_range().start,
        stream.sequence_numbers_range().end,
    );
    for event in stream.events() {
        println!("  {event:?}");
    }
}
