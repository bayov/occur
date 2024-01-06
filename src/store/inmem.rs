use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use futures_locks::RwLock;

use crate::store::read::AsyncIterator;
use crate::store::stream::Subscription;
use crate::store::{commit, read, Result};
use crate::{revision, store, Event};

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
        start_from: commit::Number,
        converter: impl read::Converter<T, Result = R> + Send,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = R>>> + Send {
        async move {
            Ok(EventIterator::new(
                Arc::clone(&self.events),
                start_from,
                converter,
            ))
        }
    }
}

#[allow(clippy::manual_async_fn)]
impl<T: Event> store::Stream<T> for Stream<T> {
    fn subscribe<R>(
        &self,
        start_from: commit::Number,
        converter: impl read::Converter<T, Result = R> + Send,
    ) -> impl Future<Output = Result<impl Subscription<Item = R>>> + Send {
        async move {
            Ok(EventSubscription::new(
                Arc::clone(&self.events),
                start_from,
                self.receiver.clone(),
                converter,
            ))
        }
    }
}

pub struct EventIterator<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    events: SmartVec<revision::OldOrNew<T>>,
    commit_number: commit::Number,
    converter: C,
}

impl<T, C> EventIterator<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    #[must_use]
    const fn new(
        events: SmartVec<revision::OldOrNew<T>>,
        start_commit_number: commit::Number,
        converter: C,
    ) -> Self {
        Self { events, commit_number: start_commit_number, converter }
    }
}

#[allow(clippy::future_not_send)]
impl<T, C> AsyncIterator for EventIterator<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    type Item = C::Result;

    async fn next(&mut self) -> Option<C::Result> {
        let events = self.events.read().await;
        let index = usize::try_from(self.commit_number).unwrap();
        let old_or_new = events.get(index);
        old_or_new.map(|old_or_new| {
            self.commit_number += 1;
            self.converter.convert(old_or_new.clone())
        })
    }
}

pub struct EventSubscription<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    events: SmartVec<revision::OldOrNew<T>>,
    commit_number: commit::Number,
    receiver: async_channel::Receiver<commit::Number>,
    converter: C,
}

impl<T, C> EventSubscription<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    #[must_use]
    const fn new(
        events: SmartVec<revision::OldOrNew<T>>,
        start_commit_number: commit::Number,
        receiver: async_channel::Receiver<commit::Number>,
        converter: C,
    ) -> Self {
        Self { events, commit_number: start_commit_number, receiver, converter }
    }
}

#[allow(clippy::future_not_send)]
impl<T, C> Subscription for EventSubscription<T, C>
where
    T: Event,
    C: read::Converter<T>,
{
    type Item = C::Result;

    async fn next(&mut self) -> Option<C::Result> {
        {
            let events = self.events.read().await;
            let index = usize::try_from(self.commit_number).unwrap();
            let old_or_new = events.get(index);
            if let Some(old_or_new) = old_or_new {
                self.commit_number += 1;
                return Some(self.converter.convert(old_or_new.clone()));
            }
        }
        while let Ok(commit_number) = self.receiver.recv().await {
            if commit_number >= self.commit_number {
                let events = self.events.read().await;
                let index = usize::try_from(self.commit_number).unwrap();
                let old_or_new = events.get(index).unwrap();
                self.commit_number += 1;
                return Some(self.converter.convert(old_or_new.clone()));
            }
        }
        None
    }

    fn stop(self) { self.receiver.close(); }
}
