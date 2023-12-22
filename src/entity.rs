use crate::stream_desc;

/// The result of folding an event stream.
pub trait Entity<D: stream_desc::StreamDesc> {
    fn new(id: D::Id, event: D::Event) -> Option<Self>
    where
        Self: Sized;

    #[must_use]
    fn fold(self, event: D::Event) -> Self
    where
        Self: Sized;
}
