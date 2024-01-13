use std::future::Future;

use crate::store::{commit, Result};
use crate::{revision, Event};

pub trait Request<T: Event>: Send + 'static {
    type Result;
    fn start_from(&self) -> commit::Number;
    fn limit(&self) -> Option<usize>;
    fn convert(event: revision::OldOrNew<T>) -> Self::Result;
}

pub struct All;

impl<T: Event> Request<T> for All {
    type Result = T;
    fn start_from(&self) -> commit::Number { 0 }
    fn limit(&self) -> Option<usize> { None }
    fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev.to_new() }
}

pub struct StartFrom(pub commit::Number);

impl<T: Event> Request<T> for StartFrom {
    type Result = T;
    fn start_from(&self) -> commit::Number { self.0 }
    fn limit(&self) -> Option<usize> { None }
    fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev.to_new() }
}

pub struct Options {
    pub start_from: commit::Number,
    pub limit: Option<usize>,
}

impl<T: Event> Request<T> for Options {
    type Result = T;
    fn start_from(&self) -> commit::Number { self.start_from }
    fn limit(&self) -> Option<usize> { self.limit }
    fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev.to_new() }
}

pub mod no_revision_convert {
    use super::Request;
    use crate::store::commit;
    use crate::{revision, Event};

    pub struct All;

    impl<T: Event> Request<T> for All {
        type Result = revision::OldOrNew<T>;
        fn start_from(&self) -> commit::Number { 0 }
        fn limit(&self) -> Option<usize> { None }
        fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev }
    }

    pub struct StartFrom(pub commit::Number);

    impl<T: Event> Request<T> for StartFrom {
        type Result = revision::OldOrNew<T>;
        fn start_from(&self) -> commit::Number { self.0 }
        fn limit(&self) -> Option<usize> { None }
        fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev }
    }

    pub struct Options {
        pub start_from: commit::Number,
        pub limit: Option<usize>,
    }

    impl<T: Event> Request<T> for Options {
        type Result = revision::OldOrNew<T>;
        fn start_from(&self) -> commit::Number { self.start_from }
        fn limit(&self) -> Option<usize> { self.limit }
        fn convert(rev: revision::OldOrNew<T>) -> Self::Result { rev }
    }
}

pub trait AsyncIterator: Send {
    type Item;
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
}

pub trait Read<T: Event>: Send {
    fn read<R>(
        &self,
        request: impl Request<T, Result = R>,
    ) -> impl Future<Output = Result<impl AsyncIterator<Item = R>>> + Send;
}
