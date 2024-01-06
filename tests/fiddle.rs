#![allow(dead_code)]

use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use futures::join;
use futures::task::SpawnExt;
use occur::store::stream::{AsyncIterator as _, Subscription as _};
use occur::store::{read, Store as _, Stream as _};
use occur::{revision, store, Revision};
use uuid::Uuid;

use crate::TvShowTrackEvent::{Created, WatchedEpisode};

#[derive(Clone, PartialEq, Eq, Hash)]
struct TvShowTrackId(Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
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

    let mut store = store::inmem::Store::<TvShowTrackEvent>::new();

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        let mut stream = store.stream(id);
        stream.commit(&e0).await.expect("wtf?");
        stream.commit(&e1).await.expect("wtf?");
        stream.commit(&e2).await.expect("wtf?");
        stream.commit(&e3).await.expect("wtf?");

        let mut it = stream.read(1, read::NewRevision).await.expect("wtf?");
        while let Some(event) = it.next().await {
            println!("read {:?}", event);
        }
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ ThreadPool subscribe test ]");

    let id = TvShowTrackId(Uuid::now_v7());
    let e0 = Created { tv_show_name: "Elementary".to_owned() };
    let e1 = WatchedEpisode { season: 1, episode: 1 };
    let e2 = WatchedEpisode { season: 1, episode: 2 };
    let e3 = WatchedEpisode { season: 1, episode: 3 };

    let store = Mutex::new(store::inmem::Store::<TvShowTrackEvent>::new());

    // remove "thread-pool" feature from futures if not using thread-pool
    let pool = futures::executor::ThreadPool::new().unwrap();

    let t = pool.spawn_with_handle(async move {
        {
            let mut stream = store.lock().unwrap().stream(id.clone());
            stream.commit(&e0).await.expect("wtf?");
            stream.commit(&e1).await.expect("wtf?");
        }

        let f1 = async {
            let stream = store.lock().unwrap().stream(id.clone());
            let mut it =
                stream.subscribe(1, read::NewRevision).await.expect("wtf?");
            while let Some(event) = it.next().await {
                println!("subscriber read {:?}", event);
                if let WatchedEpisode { episode, season: _ } = event {
                    if episode == 3 {
                        break;
                    }
                }
            }
        };

        let f2 = async {
            let mut stream = store.lock().unwrap().stream(id.clone());
            stream.commit(&e2).await.expect("wtf?");
            stream.commit(&e3).await.expect("wtf?");
        };

        join!(f1, f2);
    });

    futures::executor::block_on(t.unwrap());

    println!("\n----------------------- [ LocalPool subscribe test ]");

    let id = TvShowTrackId(Uuid::now_v7());
    let e0 = Created { tv_show_name: "Elementary".to_owned() };
    let e1 = WatchedEpisode { season: 1, episode: 1 };
    let e2 = WatchedEpisode { season: 1, episode: 2 };
    let e3 = WatchedEpisode { season: 1, episode: 3 };

    let store =
        Arc::new(Mutex::new(store::inmem::Store::<TvShowTrackEvent>::new()));

    // remove "thread-pool" feature from futures if not using thread-pool
    let mut pool = futures::executor::LocalPool::new();
    let spawner = pool.spawner();

    let id2 = id.clone();
    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let mut stream = store2.lock().unwrap().stream(id2);
            stream.commit(&e0).await.expect("wtf?");
            stream.commit(&e1).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    let id2 = id.clone();
    let store2 = Arc::clone(&store);

    spawner
        .spawn(async move {
            let stream = store2.lock().unwrap().stream(id2);
            let mut it =
                stream.subscribe(1, read::NewRevision).await.expect("wtf?");
            while let Some(event) = it.next().await {
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
            let mut stream = store2.lock().unwrap().stream(id2);
            stream.commit(&e2).await.expect("wtf?");
            stream.commit(&e3).await.expect("wtf?");
        })
        .expect("wtf?");

    pool.run();

    println!();
}
