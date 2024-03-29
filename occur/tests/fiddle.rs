#![allow(dead_code)]

use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use futures::task::SpawnExt as _;
use futures::{join, StreamExt};
use occur::store::inmem::InmemStore;
use occur::store::{
    inmem,
    read,
    write,
    ReadStream as _,
    Store as _,
    WriteStream as _,
};
use occur::{revision, Revision};
use uuid::Uuid;

use crate::TvShowTrackEvent::{Created, WatchedEpisode};

#[derive(Clone, PartialEq, Eq, Hash)]
struct TvShowTrackId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TvShowTrackEvent {
    Created { tv_show_name: String },
    WatchedEpisode { season: u64, episode: u64 },
}

impl occur::Event for TvShowTrackEvent {
    type StreamId = TvShowTrackId;
    type OldRevision = revision::Empty<Self>;
}

impl Revision for TvShowTrackEvent {
    type Value = (&'static str, u8);

    fn revision(&self) -> Self::Value {
        match &self {
            Created { .. } => ("Created", 0),
            WatchedEpisode { .. } => ("WatchedEpisode", 0),
        }
    }

    fn revision_set() -> HashSet<Self::Value> {
        HashSet::from([("Created", 0), ("WatchedEpisode", 0)])
    }
}

#[test]
fn fiddle() {
    println!("\n----------------------- [ ThreadPool read test ]");

    let id = TvShowTrackId(Uuid::now_v7());
    let e0 = Created { tv_show_name: "Elementary".to_owned() };
    let e1 = WatchedEpisode { season: 1, episode: 1 };
    let e2 = WatchedEpisode { season: 1, episode: 2 };
    let e3 = WatchedEpisode { season: 1, episode: 3 };

    let create_store = || InmemStore::new(inmem::no_serialization());

    let mut store = create_store();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        let mut ws = store.write_stream(id.clone());
        ws.commit_as_number(&e0, 0).await.expect("wtf?");
        ws.commit_unconditionally(&e1).await.unwrap();
        ws.commit_as_number(&e2, 2).await.expect("wtf?");
        ws.commit_unconditionally(&e3).await.expect("wtf?");

        let mut rs = store.read_stream(id);
        let mut it = rs.read_all().await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ ThreadPool read test ]");

    let id = TvShowTrackId(Uuid::now_v7());
    let e0 = Created { tv_show_name: "Elementary".to_owned() };
    let e1 = WatchedEpisode { season: 1, episode: 1 };
    let e2 = WatchedEpisode { season: 1, episode: 2 };
    let e3 = WatchedEpisode { season: 1, episode: 3 };

    let store = Mutex::new(create_store());

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        {
            let mut stream = store.lock().unwrap().write_stream(id.clone());
            stream
                .commit_many([&e0, &e1], write::Condition::None)
                .await
                .unwrap();
        }

        let f1 = async {
            let mut stream = store.lock().unwrap().read_stream(id.clone());
            let mut it = stream
                .read_unconverted(read::Options {
                    position: read::Position::CommitNumber(1),
                    direction: read::Direction::Forward,
                    limit: None,
                })
                .await
                .expect("wtf?");
            while let Some(event) = it.next().await {
                println!("subscriber read {:?}", event);
            }
        };

        let f2 = async {
            let mut stream = store.lock().unwrap().write_stream(id.clone());
            stream.commit_unconditionally(&e2).await.expect("wtf?");
            stream.commit_unconditionally(&e3).await.expect("wtf?");
        };

        join!(f1, f2);
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ LocalPool read test ]");

    let id = TvShowTrackId(Uuid::now_v7());
    let e0 = Created { tv_show_name: "Elementary".to_owned() };
    let e1 = WatchedEpisode { season: 1, episode: 1 };
    let e2 = WatchedEpisode { season: 1, episode: 2 };
    let e3 = WatchedEpisode { season: 1, episode: 3 };

    let store = Arc::new(Mutex::new(create_store()));

    // remove "thread-pool" feature from futures if not using thread-pool
    let mut pool = futures::executor::LocalPool::new();
    let spawner = pool.spawner();

    let id2 = id.clone();
    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().write_stream(id2);
            stream.commit_unconditionally(&e0).await.expect("wtf?");
            stream.commit_unconditionally(&e1).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    let id2 = id.clone();
    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().read_stream(id2);
            let mut it = stream
                .read_unconverted(read::Options {
                    position: read::Position::Last,
                    direction: read::Direction::Backward,
                    limit: None,
                })
                .await
                .expect("wtf?");
            while let Some(revision::OldOrNew::New(event)) = it.next().await {
                println!("subscriber read {:?}", event);
                if let WatchedEpisode { episode, season: _ } = event {
                    if episode == 3 {
                        break;
                    }
                }
            }
        })
        .expect("wtf?");

    let id2 = id.clone();
    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().write_stream(id2);
            stream.commit_unconditionally(&e2).await.expect("wtf?");
            stream.commit_unconditionally(&e3).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    println!();
}
