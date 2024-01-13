use std::future::Future;

use crate::store::{read, Commit, Read, Result};
use crate::Event;

pub trait Stream<T: Event>: Commit<T> + Read<T> {
    // TODO: subscriptions are probably not needed by default.
    fn subscribe<R>(
        &self,
        request: impl read::Request<T, Result = R>,
    ) -> impl Future<Output = Result<impl Subscription<Item = R>>> + Send;
}

pub trait Subscription: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
    fn stop(self);
}
