use std::future::Future;

use futures::{Stream, StreamExt};

use crate::store::{commit, Result};
use crate::{revision, Event};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Options {
    pub start_from: commit::Number,
    pub limit: Option<usize>,
}

pub trait Read: Send {
    type Event: Event;

    fn read_unconverted(
        &self,
        options: Options,
    ) -> impl Future<
        Output = Result<impl Stream<Item = revision::OldOrNew<Self::Event>>>,
    > + Send;

    fn read(
        &self,
        options: Options,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        let future = self.read_unconverted(options);
        async { future.await.map(|it| it.map(revision::OldOrNew::to_new)) }
    }

    fn read_all(
        &self,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        self.read(Options { start_from: 0, limit: None })
    }

    fn read_from(
        &self,
        start_from: commit::Number,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        self.read(Options { start_from, limit: None })
    }

    fn read_with_limit(
        &self,
        limit: usize,
    ) -> impl Future<Output = Result<impl Stream<Item = Self::Event>>> + Send
    {
        self.read(Options { start_from: 0, limit: Some(limit) })
    }
}
