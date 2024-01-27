use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use futures::{self};
use futures_locks::RwLock;

use crate::error::ErrorWithKind;
use crate::store::{commit, read};
use crate::{revision, store, Deserializer, Event, Serializer};

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Default)]
pub struct Store<T, S, D>
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

impl<T, S, D> Store<T, S, D>
where
    T: Event,
    S: Serializer<Event = T>,
    D: Deserializer<Event = T, SerializedEvent = S::SerializedEvent>,
    S::SerializedEvent: Clone + Send + Sync,
{
    #[must_use]
    pub fn new(serializer: S, deserializer: D) -> Self {
        Self { events_by_stream_id: HashMap::new(), serializer, deserializer }
    }
}

impl<T, S, D> store::Store for Store<T, S, D>
where
    T: Event,
    S: Serializer<Event = T>,
    D: Deserializer<Event = T, SerializedEvent = S::SerializedEvent>,
    S::SerializedEvent: Clone + Send + Sync,
{
    type Event = T;
    type WriteStream = WriteStream<T, S>;
    type ReadStream = ReadStream<T, D>;

    fn write_stream(&mut self, id: T::StreamId) -> Self::WriteStream {
        let events = self.events_by_stream_id.entry(id).or_default();
        WriteStream {
            events: events.clone(),
            serializer: self.serializer.clone(),
        }
    }

    fn read_stream(&mut self, id: T::StreamId) -> Self::ReadStream {
        let events = self.events_by_stream_id.entry(id).or_default();
        ReadStream {
            events: events.clone(),
            deserializer: self.deserializer.clone(),
        }
    }
}

#[derive(Clone)]
pub struct WriteStream<T, S>
where
    T: Event,
    S: Serializer<Event = T>,
    S::SerializedEvent: Clone + Send + Sync,
{
    events: SmartVec<S::SerializedEvent>,
    serializer: S,
}

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct CommitError {
    kind: commit::ErrorKind,
    source: Option<std::num::TryFromIntError>,
    backtrace: std::backtrace::Backtrace,
}

impl ErrorWithKind for CommitError {
    type Kind = commit::ErrorKind;
    fn kind(&self) -> Self::Kind { self.kind }
}

type CommitResult<T> = Result<T, CommitError>;

impl<T, S> store::Commit for WriteStream<T, S>
where
    T: Event,
    S: Serializer<Event = T>,
    S::SerializedEvent: Clone + Send + Sync,
{
    type Event = T;
    type Error = CommitError;

    fn commit_old_or_new(
        &mut self,
        event: revision::OldOrNewRef<'_, Self::Event>,
        condition: commit::Condition,
    ) -> impl Future<Output = CommitResult<commit::Number>> + Send {
        let serialized_event = self.serializer.serialize(event);
        async move {
            let mut events = self.events.write().await;
            let commit_number = next_commit_number(events.len(), condition)?;
            events.push(serialized_event);
            Ok(commit_number)
        }
    }

    fn commit_many<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a Self::Event>,
        condition: commit::Condition,
    ) -> impl Future<Output = CommitResult<Option<commit::Number>>> + Send {
        let events_to_commit: Vec<_> = events
            .into_iter()
            .map(revision::OldOrNewRef::New)
            .map(|event| self.serializer.serialize(event))
            .collect();
        async move {
            if events_to_commit.is_empty() {
                return Ok(None);
            }
            let mut events = self.events.write().await;
            let commit_number = next_commit_number(events.len(), condition)?;
            events.extend(events_to_commit);
            Ok(Some(commit_number))
        }
    }
}

#[derive(Clone)]
pub struct ReadStream<T, D>
where
    T: Event,
    D: Deserializer<Event = T>,
    D::SerializedEvent: Clone + Send + Sync,
{
    events: SmartVec<D::SerializedEvent>,
    deserializer: D,
}

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

impl<T, D> store::Read for ReadStream<T, D>
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
            read::Position::Commit(number) => number as usize,
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

fn next_commit_number(
    n_events: usize,
    condition: commit::Condition,
) -> CommitResult<commit::Number> {
    let commit_number =
        u32::try_from(n_events).map_err(|source| CommitError {
            kind: commit::ErrorKind::StreamFull,
            source: Some(source),
            backtrace: std::backtrace::Backtrace::capture(),
        })?;
    match condition {
        commit::Condition::None => {}
        commit::Condition::AssignCommitNumber(assign_commit_number) => {
            if commit_number != assign_commit_number {
                return Err(CommitError {
                    kind: commit::ErrorKind::ConditionNotMet,
                    source: None,
                    backtrace: std::backtrace::Backtrace::capture(),
                });
            }
        }
    }
    Ok(commit_number)
}
