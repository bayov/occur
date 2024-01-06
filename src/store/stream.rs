use std::future::Future;

use crate::store::{commit, read, Commit, Read, Result};
use crate::Event;

pub trait Stream<T: Event>: Commit<T> + Read<T> {
    fn subscribe<R>(
        &self,
        start_from: commit::Number,
        converter: impl read::Converter<T, Result = R> + Send + 'static,
    ) -> impl Future<Output = Result<impl Subscription<Item = R>>> + Send;
}

pub trait Subscription: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
    fn stop(self);
}
