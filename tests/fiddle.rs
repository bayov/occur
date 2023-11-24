#![allow(dead_code)]

use crate::TvShowTrackEvents::{Created, WatchedEpisode};
use event_sourcing::repo::{EventIterator, Repository, Stream as RepoStream};
use event_sourcing::{repo, Event, Id, SequenceNumber, Stream};
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
    let mut repo_stream = repo.new_stream();
    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    futures::executor::block_on(async {
        repo_stream.write(SequenceNumber(0), &e0).await.expect("wtf?");
        repo_stream.write(SequenceNumber(1), &e1).await.expect("wtf?");
        repo_stream.write(SequenceNumber(2), &e2).await.expect("wtf?");
        repo_stream.write(SequenceNumber(3), &e3).await.expect("wtf?");

        let mut it = repo_stream.read(SequenceNumber(1)).await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

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
