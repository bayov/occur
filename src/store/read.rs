use std::future::Future;

use crate::store::{commit, Result};
use crate::{AsyncIterator, Event, OldOrNewEventIterator};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Options {
    pub start_from: commit::Number,
    pub limit: Option<usize>,
}

pub trait Read<T: Event>: Send {
    fn read_unconverted(
        &self,
        options: Options,
    ) -> impl Future<Output = Result<impl OldOrNewEventIterator<T>>> + Send;

    fn read(
        &self,
        options: Options,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = T>>> + Send {
        let future = self.read_unconverted(options);
        async { future.await.map(OldOrNewEventIterator::to_new) }
    }

    fn read_all(
        &self,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = T>>> + Send {
        self.read(Options { start_from: 0, limit: None })
    }

    fn read_from(
        &self,
        start_from: commit::Number,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = T>>> + Send {
        self.read(Options { start_from, limit: None })
    }

    fn read_with_limit(
        &self,
        limit: usize,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = T>>> + Send {
        self.read(Options { start_from: 0, limit: Some(limit) })
    }
}
