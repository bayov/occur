use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use futures_locks::RwLock;

use crate::{Event, revision, store};
use crate::store::{commit, read, Result};
use crate::store::read::AsyncIterator;
use crate::store::stream::Subscription;

#[derive(Default)]
pub struct Store<T: Event> {
    events_by_stream_id: HashMap<T::StreamId, Stream<T>>,
}

impl<T: Event> Store<T> {
    #[must_use]
    pub fn new() -> Self { Self { events_by_stream_id: HashMap::default() } }
}

impl<T: Event> store::Store<T> for Store<T> {
    type Stream = Stream<T>;

    fn stream(&mut self, id: T::StreamId) -> Self::Stream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Clone)]
pub struct Stream<T: Event> {
    events: SmartVec<revision::OldOrNew<T>>,
    sender: async_channel::Sender<commit::Number>,
    receiver: async_channel::Receiver<commit::Number>,
}

impl<T: Event> Stream<T> {
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { events: Arc::default(), sender, receiver }
    }
}

impl<T: Event> Default for Stream<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Event> store::Commit<T> for Stream<T> {
    fn commit(
        &mut self,
        request: impl commit::Request<T>,
    ) -> impl Future<Output = Result<commit::Number>> + Send {
        let event = request.event().to_owned();
        let condition = request.condition();
        async move {
            let commit_number;
            {
                let mut guarded_events = self.events.write().await;
                commit_number = guarded_events.len().try_into().unwrap();
                match condition {
                    commit::Condition::None => {}
                    commit::Condition::Number(want_commit_number) => {
                        assert_eq!(commit_number, want_commit_number);
                    }
                }
                guarded_events.push(event);
            }

            self.sender.send(commit_number).await.unwrap();
            Ok(commit_number)
        }
    }
}

#[allow(clippy::manual_async_fn)]
impl<T: Event> store::Read<T> for Stream<T> {
    fn read<R>(
        &self,
        request: impl read::Request<T, Result=R>,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = R>>> + Send {
        async move { Ok(EventIterator::new(Arc::clone(&self.events), &request)) }
    }
}

#[allow(clippy::manual_async_fn)]
impl<T: Event> store::Stream<T> for Stream<T> {
    fn subscribe<R>(
        &self,
        request: impl read::Request<T, Result=R>,
    ) -> impl Future<Output = Result<impl Subscription<Item = R>>> + Send {
        async move {
            Ok(EventSubscription::new(
                Arc::clone(&self.events),
                &request,
                self.receiver.clone(),
            ))
        }
    }
}

pub struct EventIterator<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    events: SmartVec<revision::OldOrNew<T>>,
    commit_number: commit::Number,
    limit: usize,
    phantom: std::marker::PhantomData<R>,
}

impl<T, R> EventIterator<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    #[must_use]
    fn new(events: SmartVec<revision::OldOrNew<T>>, request: &R) -> Self {
        Self {
            events,
            commit_number: request.start_from(),
            limit: request.limit().unwrap_or(usize::MAX),
            phantom: std::marker::PhantomData,
        }
    }
}

#[allow(clippy::future_not_send)]
impl<T, R> AsyncIterator for EventIterator<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    type Item = R::Result;

    async fn next(&mut self) -> Option<R::Result> {
        if self.limit == 0 {
            return None;
        }
        let events = self.events.read().await;
        let index = usize::try_from(self.commit_number).unwrap();
        let old_or_new = events.get(index);
        old_or_new.map(|old_or_new| {
            self.commit_number += 1;
            self.limit -= 1;
            R::convert(old_or_new.clone())
        })
    }
}

pub struct EventSubscription<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    events: SmartVec<revision::OldOrNew<T>>,
    commit_number: commit::Number,
    limit: usize,
    receiver: async_channel::Receiver<commit::Number>,
    phantom: std::marker::PhantomData<R>,
}

impl<T, R> EventSubscription<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    #[must_use]
    fn new(
        events: SmartVec<revision::OldOrNew<T>>,
        request: &R,
        receiver: async_channel::Receiver<commit::Number>,
    ) -> Self {
        Self {
            events,
            commit_number: request.start_from(),
            limit: request.limit().unwrap_or(usize::MAX),
            receiver,
            phantom: std::marker::PhantomData,
        }
    }
}

#[allow(clippy::future_not_send)]
impl<T, R> Subscription for EventSubscription<T, R>
where
    T: Event,
    R: read::Request<T>,
{
    type Item = R::Result;

    async fn next(&mut self) -> Option<R::Result> {
        if self.limit == 0 {
            return None;
        }
        {
            let events = self.events.read().await;
            let index = usize::try_from(self.commit_number).unwrap();
            let old_or_new = events.get(index);
            if let Some(old_or_new) = old_or_new {
                self.commit_number += 1;
                self.limit -= 1;
                return Some(R::convert(old_or_new.clone()));
            }
        }
        while let Ok(commit_number) = self.receiver.recv().await {
            if commit_number >= self.commit_number {
                let events = self.events.read().await;
                let index = usize::try_from(self.commit_number).unwrap();
                let old_or_new = events.get(index).unwrap();
                self.commit_number += 1;
                self.limit -= 1;
                return Some(R::convert(old_or_new.clone()));
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
