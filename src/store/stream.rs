use std::future::Future;

use crate::store::{read, Commit, Read, Result};
use crate::{revision, Event};

pub trait Stream<T: Event>: Commit<T> + Read<T> {
    // TODO: subscriptions are probably not needed by default.
    #[rustfmt::skip]
    fn subscribe(&self, options: read::Options) -> impl Future<Output=
    Result<impl Subscription<Item=revision::OldOrNew<T>>>
    > + Send;
}

pub trait Subscription: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
    fn stop(self);
}
