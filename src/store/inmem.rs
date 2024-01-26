use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use futures::{self};
use futures_locks::RwLock;

use crate::error::ErrorWithKind;
use crate::store::{commit, read, Result as StoreResult};
use crate::{revision, store, Event};

#[derive(Default)]
pub struct Store<T: Event> {
    events_by_stream_id: HashMap<T::StreamId, Stream<T>>,
}

impl<T: Event> Store<T> {
    #[must_use]
    pub fn new() -> Self { Self { events_by_stream_id: HashMap::default() } }
}

impl<T: Event> store::Store for Store<T> {
    type Event = T;
    type EventStream = Stream<T>;

    fn stream(&mut self, id: T::StreamId) -> Self::EventStream {
        self.events_by_stream_id.entry(id).or_default().clone()
    }
}

type SmartVec<T> = Arc<RwLock<Vec<T>>>;

#[derive(Clone)]
pub struct Stream<T: Event> {
    events: SmartVec<revision::OldOrNew<T>>,
}

impl<T: Event> Stream<T> {
    #[must_use]
    fn new() -> Self { Self { events: Arc::default() } }
}

impl<T: Event> Default for Stream<T> {
    fn default() -> Self { Self::new() }
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

impl<T: Event> store::Commit for Stream<T> {
    type Event = T;
    type Error = CommitError;

    fn commit_old_or_new(
        &mut self,
        event: revision::OldOrNewRef<'_, Self::Event>,
        condition: commit::Condition,
    ) -> impl Future<Output = CommitResult<commit::Number>> + Send {
        let event = event.to_owned();
        async move {
            let mut events = self.events.write().await;
            let commit_number = next_commit_number(events.len(), condition)?;
            events.push(event);
            Ok(commit_number)
        }
    }

    fn commit_many<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a Self::Event>,
        condition: commit::Condition,
    ) -> impl Future<Output = CommitResult<Option<commit::Number>>> + Send {
        let events_to_commit: Vec<_> =
            events.into_iter().cloned().map(revision::OldOrNew::New).collect();
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

impl<T: Event> store::Read for Stream<T> {
    type Event = T;

    async fn read_unconverted(
        &self,
        options: read::Options,
    ) -> StoreResult<impl futures::Stream<Item = revision::OldOrNew<T>>> {
        let events = self.events.read().await;
        let start = match options.position {
            read::Position::Start => 0,
            read::Position::End => events.len(),
            read::Position::Commit(number) => number as usize,
        };
        let limit = options.limit.unwrap_or(usize::MAX);
        let events: Vec<_> = match options.direction {
            read::Direction::Forward => {
                events[start..].iter().take(limit).cloned().collect()
            }
            read::Direction::Backward => {
                events[0..start].iter().rev().take(limit).cloned().collect()
            }
        };
        Ok(futures::stream::iter(events))
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
