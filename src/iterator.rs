use std::future::Future;

use crate::{revision, Event};

/// An asynchronous iterator, returning `Future` items.
///
/// Similar to the [`std::async_iter::AsyncIterator`] trait, but easier to
/// implement using `async` blocks (compared to implementing `poll_next`).
///
/// TODO:
///     Remove this trait and use the standard `AsyncIterator` trait when it
///     becomes easy to implement.
#[allow(clippy::module_name_repetitions)]
pub trait AsyncIterator: Send {
    type Item;

    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
}

#[allow(clippy::module_name_repetitions)]
pub trait OldOrNewEventIterator<T: Event>:
    AsyncIterator<Item = revision::OldOrNew<T>> + Send
{
    fn to_new(self) -> impl AsyncIterator<Item = T>
    where
        Self: Sized,
    {
        NewEventIterator { it: self, phantom: std::marker::PhantomData }
    }
}

impl<T, It> OldOrNewEventIterator<T> for It
where
    T: Event,
    It: AsyncIterator<Item = revision::OldOrNew<T>>,
{
}

#[allow(clippy::module_name_repetitions)]
pub struct NewEventIterator<T: Event, It: OldOrNewEventIterator<T>> {
    it: It,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Event, It: OldOrNewEventIterator<T>> AsyncIterator
    for NewEventIterator<T, It>
{
    type Item = T;

    async fn next(&mut self) -> Option<Self::Item> {
        self.it.next().await.map(revision::OldOrNew::to_new)
    }
}
