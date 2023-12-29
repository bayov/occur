use crate::Event;

/// The result of folding an event stream.
pub trait Entity<T: Event> {
    fn new(id: T::Id, event: T) -> Option<Self>
    where
        Self: Sized;

    #[must_use]
    fn fold(self, event: T) -> Self
    where
        Self: Sized;
}
