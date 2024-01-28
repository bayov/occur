use crate::store::inmem::SmartVec;
use crate::store::{read, Deserializer, ReadStream};
use crate::{revision, ErrorWithKind, Event};

#[derive(Clone)]
pub struct InmemReadStream<T, D>
where
    T: Event,
    D: Deserializer<Event = T>,
    D::SerializedEvent: Clone + Send + Sync,
{
    pub(super) events: SmartVec<D::SerializedEvent>,
    pub(super) deserializer: D,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct ReadError {
    kind: read::ErrorKind,
    backtrace: std::backtrace::Backtrace,
}

impl ErrorWithKind for ReadError {
    type Kind = read::ErrorKind;
    fn kind(&self) -> Self::Kind { self.kind }
}

type ReadResult<T> = Result<T, ReadError>;

impl<T, D> ReadStream for InmemReadStream<T, D>
where
    T: Event,
    D: Deserializer<Event = T>,
    D::SerializedEvent: Clone + Send + Sync,
{
    type Event = T;
    type Error = ReadError;

    async fn read_unconverted(
        &mut self,
        options: read::Options,
    ) -> ReadResult<impl futures::Stream<Item = revision::OldOrNew<T>>> {
        let events = self.events.read().await;
        let start = match options.position {
            read::Position::First => 0,
            read::Position::Last => events.len() - 1,
            read::Position::CommitNumber(number) => number as usize,
        };
        if start >= events.len() {
            return Err(ReadError {
                kind: read::ErrorKind::CommitNotFound,
                backtrace: std::backtrace::Backtrace::capture(),
            });
        }
        let limit = options.limit.unwrap_or(usize::MAX);
        let deserializer = &self.deserializer;
        let mut deserialized_events = Vec::new();
        {
            match options.direction {
                read::Direction::Forward => {
                    events[start..]
                        .iter()
                        .take(limit)
                        .cloned()
                        .map(|event| deserializer.deserialize(event))
                        .collect_into(&mut deserialized_events);
                }
                read::Direction::Backward => {
                    events[0..start]
                        .iter()
                        .rev()
                        .take(limit)
                        .cloned()
                        .map(|event| deserializer.deserialize(event))
                        .collect_into(&mut deserialized_events);
                }
            };
        }
        Ok(futures::stream::iter(deserialized_events))
    }
}
