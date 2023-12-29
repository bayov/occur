use crate::Event;

/// The result of folding an event stream.
pub trait Entity<T: Event> {
    /// Creates a new entity from the provided stream `id` and `event`.
    ///
    /// An event represents a recorded fact and cannot fail to be applied,
    /// meaning that non-creation events should simply be ignored and return
    /// [`None`].
    fn new(id: T::StreamId, event: T) -> Option<Self>
    where
        Self: Sized;

    /// Folds the existing entity using the provided `event`.
    ///
    /// An event represents a recorded fact and cannot fail to be applied,
    /// meaning that events that cannot be used to fold the current entity
    /// should simply be ignored and return [`None`].
    #[must_use]
    fn fold(self, event: T) -> Self
    where
        Self: Sized;
}
