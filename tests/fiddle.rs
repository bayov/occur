#![allow(dead_code)]

use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use event_sourcing::store::{
    EventIterator,
    EventSubscription,
    Store,
    Stream as _,
};
use event_sourcing::{store, CommitNumber, Event, Stream, StreamDescription};
use futures::join;
use futures::task::SpawnExt;

use crate::TvShowTrackEvent::{Created, WatchedEpisode};

struct TvShowTrackStreamDescription;

impl StreamDescription for TvShowTrackStreamDescription {
    const NAME: &'static str = "tv_show_track";
    type Id = store::inmem::Id;
    type Event = TvShowTrackEvent;
}

type TvShowTrackStream = Stream<TvShowTrackStreamDescription>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TvShowTrackEvent {
    Created { tv_show_name: String },
    WatchedEpisode { season: u64, episode: u64 },
}

impl Event for TvShowTrackEvent {
    fn supported_revisions() -> HashSet<Self::Revision> {
        HashSet::from([
            Self::Revision::new("Created", 0),
            Self::Revision::new("WatchedEpisode", 0),
        ])
    }

    fn revision(&self) -> Self::Revision {
        match &self {
            Created { .. } => Self::Revision::new("Created", 0),
            WatchedEpisode { .. } => Self::Revision::new("WatchedEpisode", 0),
        }
    }
}

#[test]
fn fiddle() {
    println!("\n----------------------- [ Basic test ]");

    let mut stream = TvShowTrackStream::new(42);
    stream.commit(Created { tv_show_name: "Elementary".into() });
    stream.commit(WatchedEpisode { season: 1, episode: 1 });
    stream.commit(WatchedEpisode { season: 1, episode: 2 });
    stream.commit(WatchedEpisode { season: 1, episode: 3 });

    print_stream(&stream);

    println!("\n----------------------- [ Indexing test ]");

    println!("stream[0] = {:?}", stream[CommitNumber(0)]);
    println!("stream[2] = {:?}", stream[CommitNumber(2)]);

    println!("\n----------------------- [ ThreadPool read test ]");

    let e0 = stream[CommitNumber(0)].clone();
    let e1 = stream[CommitNumber(1)].clone();
    let e2 = stream[CommitNumber(2)].clone();
    let e3 = stream[CommitNumber(3)].clone();

    let mut store = store::inmem::Store::new();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        let mut stream = store.new_stream();
        stream.write(&e0).await.expect("wtf?");
        stream.write(&e1).await.expect("wtf?");
        stream.write(&e2).await.expect("wtf?");
        stream.write(&e3).await.expect("wtf?");

        let mut it = stream.read(CommitNumber(1)).await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ ThreadPool subscribe test ]");

    let e0 = stream[CommitNumber(0)].clone();
    let e1 = stream[CommitNumber(1)].clone();
    let e2 = stream[CommitNumber(2)].clone();
    let e3 = stream[CommitNumber(3)].clone();

    let store = Mutex::new(store::inmem::Store::new());
    let id = store.lock().unwrap().new_id();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        {
            let mut stream = store.lock().unwrap().stream(id);
            stream.write(&e0).await.expect("wtf?");
            stream.write(&e1).await.expect("wtf?");
        }

        let f1 = async {
            let stream = store.lock().unwrap().stream(id);
            let mut it = stream.subscribe(CommitNumber(1)).await.expect("wtf?");
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
            let mut stream = store.lock().unwrap().stream(id);
            stream.write(&e2).await.expect("wtf?");
            stream.write(&e3).await.expect("wtf?");
        };

        join!(f1, f2);
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ LocalPool subscribe test ]");

    let e0 = stream[CommitNumber(0)].clone();
    let e1 = stream[CommitNumber(1)].clone();
    let e2 = stream[CommitNumber(2)].clone();
    let e3 = stream[CommitNumber(3)].clone();

    let store = Arc::new(Mutex::new(store::inmem::Store::new()));
    let id = store.lock().unwrap().new_id();

    // remove "thread-pool" feature from futures if not using thread-pool
    let mut pool = futures::executor::LocalPool::new();
    let spawner = pool.spawner();

    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().stream(id);
            stream.write(&e0).await.expect("wtf?");
            stream.write(&e1).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let stream = store2.lock().unwrap().stream(id);
            let mut it = stream.subscribe(CommitNumber(1)).await.expect("wtf?");
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

    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().stream(id);
            stream.write(&e2).await.expect("wtf?");
            stream.write(&e3).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    // println!("\n----------------------- [ Stream rebuild test ]");
    //
    // let mut events = stream.take_events();
    // events.remove(0);
    // let mut stream =
    //     Stream::from_recorded_events(42, CommitNumber(1),
    // Cow::Owned(events)); stream.commit(WatchedEpisode { season: 1,
    // episode: 4 });
    //
    // print_stream(&stream);

    println!();
}

fn print_stream<T: StreamDescription>(stream: &Stream<T>)
where
    T::Id: Debug,
    T::Time: Debug,
    T::Event: Debug,
{
    println!(
        "Stream {{ id: {:?}, range: {}..{} }}",
        stream.id(),
        stream.commit_numbers_range().start,
        stream.commit_numbers_range().end,
    );
    for event in stream.events() {
        println!("  {event:?}");
    }
}
