use std::future::Future;

use crate::store::{commit, Result};
use crate::{revision, Event};

pub trait Request<T: Event> {
    type Result;
    fn start_from(&self) -> commit::Number;
    fn limit(&self) -> usize;
}

pub struct Foo {
    pub start_from: commit::Number,
}

impl<T: Event> Request<T> for Foo {
    type Result = T;

    fn start_from(&self) -> commit::Number { self.start_from }

    fn limit(&self) -> usize { todo!() }
}

pub trait Converter<T: Event>: Send {
    type Result;
    fn convert(&self, event: revision::OldOrNew<T>) -> Self::Result;
}

pub struct NewRevision;

impl<T: Event> Converter<T> for NewRevision {
    type Result = T;
    fn convert(&self, event: revision::OldOrNew<T>) -> Self::Result {
        event.to_new()
    }
}

pub trait AsyncIterator: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
}

pub trait Read<T: Event>: Send {
    fn read<R>(
        &self,
        start_from: commit::Number,
        converter: impl Converter<T, Result = R> + Send + 'static,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = R>>> + Send;
}
