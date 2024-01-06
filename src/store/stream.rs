use std::future::Future;

use crate::store::{commit, read, Result};
use crate::Event;

pub trait Stream<T: Event>: Send {
    fn commit(
        &mut self,
        request: impl commit::Request<T>,
    ) -> impl Future<Output = Result<commit::Number>> + Send;

    fn read<R>(
        &self,
        start_from: commit::Number,
        converter: impl read::Converter<T, Result = R> + Send + 'static,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = R>>> + Send;

    fn subscribe<R>(
        &self,
        start_from: commit::Number,
        converter: impl read::Converter<T, Result = R> + Send + 'static,
    ) -> impl Future<Output = Result<impl Subscription<Item = R>>> + Send;
}

pub trait AsyncIterator: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
}

pub trait Subscription: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
    fn stop(self);
}
