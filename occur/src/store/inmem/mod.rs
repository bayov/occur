use std::collections::HashMap;
use std::sync::Arc;

use futures_locks::RwLock;
pub use read::ReadError;
pub use serialization::no_serialization;
pub use write::WriteError;

use crate::store::inmem::read::InmemReadStream;
use crate::store::inmem::write::InmemWriteStream;
use crate::store::serialization::Serialization;
use crate::store::{Deserializer, Serializer};
use crate::{Event, Store};

mod read;
mod serialization;
mod write;

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct InmemStore<T, S, D>
where
    T: Event,
    S: Serializer<Event = T>,
    D: Deserializer<Event = T, SerializedEvent = S::SerializedEvent>,
    S::SerializedEvent: Clone + Send + Sync,
{
    events_by_stream_id: HashMap<T::StreamId, SmartVec<S::SerializedEvent>>,
    serializer: S,
    deserializer: D,
}

impl<T, S, D> InmemStore<T, S, D>
where
    T: Event,
    S: Serializer<Event = T>,
    D: Deserializer<Event = T, SerializedEvent = S::SerializedEvent>,
    S::SerializedEvent: Clone + Send + Sync,
{
    pub fn new(serialization: Serialization<S, D>) -> Self {
        let Serialization { serializer, deserializer } = serialization;
        Self { events_by_stream_id: HashMap::new(), serializer, deserializer }
    }
}

impl<T, S, D> Store for InmemStore<T, S, D>
where
    T: Event,
    S: Serializer<Event = T>,
    D: Deserializer<Event = T, SerializedEvent = S::SerializedEvent>,
    S::SerializedEvent: Clone + Send + Sync,
{
    type Event = T;
    type WriteStream = InmemWriteStream<T, S>;
    type ReadStream = InmemReadStream<T, D>;

    fn write_stream(&mut self, id: T::StreamId) -> Self::WriteStream {
        let events = self.events_by_stream_id.entry(id).or_default();
        InmemWriteStream {
            events: events.clone(),
            serializer: self.serializer.clone(),
        }
    }

    fn read_stream(&mut self, id: T::StreamId) -> Self::ReadStream {
        let events = self.events_by_stream_id.entry(id).or_default();
        InmemReadStream {
            events: events.clone(),
            deserializer: self.deserializer.clone(),
        }
    }
}
