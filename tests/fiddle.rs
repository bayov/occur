#![allow(dead_code)]

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use futures::join;
use futures::task::SpawnExt;

use event_sourcing::repo::{
    EventIterator,
    EventSubscription,
    Repository,
    Stream as _,
};
use event_sourcing::{repo, Event, Id, SequenceNumber, Stream};

use crate::TvShowTrackEvents::{Created, WatchedEpisode};

#[derive(Debug, Clone)]
enum TvShowTrackEvents {
    Created { tv_show_name: String },
    WatchedEpisode { season: u64, episode: u64 },
}

impl Event for TvShowTrackEvents {}

#[test]
fn fiddle() {
    println!("\n----------------------- [ Basic test ]");

    let mut stream = Stream::new(42);
    stream.record(Created { tv_show_name: "Elementary".into() });
    stream.record(WatchedEpisode { season: 1, episode: 1 });
    stream.record(WatchedEpisode { season: 1, episode: 2 });
    stream.record(WatchedEpisode { season: 1, episode: 3 });

    print_stream(&stream);

    println!("\n----------------------- [ Indexing test ]");

    println!("stream[0] = {:?}", stream[SequenceNumber(0)]);
    println!("stream[2] = {:?}", stream[SequenceNumber(2)]);

    println!("\n----------------------- [ ThreadPool read test ]");

    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    let mut repo = repo::fake::Repository::new();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        let mut stream = repo.new_stream();
        stream.write(SequenceNumber(0), &e0).await.expect("wtf?");
        stream.write(SequenceNumber(1), &e1).await.expect("wtf?");
        stream.write(SequenceNumber(2), &e2).await.expect("wtf?");
        stream.write(SequenceNumber(3), &e3).await.expect("wtf?");

        let mut it = stream.read(SequenceNumber(1)).await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ ThreadPool subscribe test ]");

    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    let repo = Mutex::new(repo::fake::Repository::new());
    let id = repo.lock().unwrap().new_id();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        {
            let mut stream = repo.lock().unwrap().stream(id);
            stream.write(SequenceNumber(0), &e0).await.expect("wtf?");
            stream.write(SequenceNumber(1), &e1).await.expect("wtf?");
        }

        let f1 = async {
            let stream = repo.lock().unwrap().stream(id);
            let mut it =
                stream.subscribe(SequenceNumber(1)).await.expect("wtf?");
            while let Some(event) = it.next().await {
                println!("subscriber read {:?}", event);
                if let WatchedEpisode { episode, season: _ } = event.event {
                    if episode == 3 {
                        break;
                    }
                }
            }
        };

        let f2 = async {
            let mut stream = repo.lock().unwrap().stream(id);
            stream.write(SequenceNumber(2), &e2).await.expect("wtf?");
            stream.write(SequenceNumber(3), &e3).await.expect("wtf?");
        };

        join!(f1, f2);
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ LocalPool subscribe test ]");

    let e0 = stream[SequenceNumber(0)].clone();
    let e1 = stream[SequenceNumber(1)].clone();
    let e2 = stream[SequenceNumber(2)].clone();
    let e3 = stream[SequenceNumber(3)].clone();

    let repo = Arc::new(Mutex::new(repo::fake::Repository::new()));
    let id = repo.lock().unwrap().new_id();

    // remove "thread-pool" feature from futures if not using thread-pool
    let mut pool = futures::executor::LocalPool::new();
    let spawner = pool.spawner();

    let repo2 = Arc::clone(&repo);

    spawner
        .spawn(async move {
            let mut stream = repo2.lock().unwrap().stream(id);
            stream.write(SequenceNumber(0), &e0).await.expect("wtf?");
            stream.write(SequenceNumber(1), &e1).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    let repo2 = Arc::clone(&repo);

    spawner
        .spawn(async move {
            let stream = repo2.lock().unwrap().stream(id);
            let mut it =
                stream.subscribe(SequenceNumber(1)).await.expect("wtf?");
            while let Some(event) = it.next().await {
                println!("subscriber read {:?}", event);
                if let WatchedEpisode { episode, season: _ } = event.event {
                    if episode == 3 {
                        break;
                    }
                }
            }
        })
        .expect("wtf?");

    let repo2 = Arc::clone(&repo);

    spawner
        .spawn(async move {
            let mut stream = repo2.lock().unwrap().stream(id);
            stream.write(SequenceNumber(2), &e2).await.expect("wtf?");
            stream.write(SequenceNumber(3), &e3).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    println!("\n----------------------- [ Stream rebuild test ]");

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
