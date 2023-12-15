use crate::StreamDescription;

pub trait Entity<T: StreamDescription> {
    fn new(id: T::Id, event: T::Event) -> Option<Self>
    where
        Self: Sized;

    fn apply(self, event: T::Event) -> Self
    where
        Self: Sized;
}
