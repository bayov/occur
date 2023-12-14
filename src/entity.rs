use crate::StreamDescriptor;

pub trait Entity<T: StreamDescriptor> {
    fn new(id: T::Id, event: T::Event) -> Result<Self, T::Error>
    where
        Self: Sized;

    fn apply(self, event: T::Event) -> Result<Self, T::Error>
    where
        Self: Sized;
}
