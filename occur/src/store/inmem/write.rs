use std::future::Future;

use crate::store::inmem::SmartVec;
use crate::store::{write, CommitNumber, Serializer, WriteStream};
use crate::{revision, ErrorWithKind, Event};

#[derive(Clone)]
pub struct InmemWriteStream<T, S>
where
    T: Event,
    S: Serializer<Event = T>,
    S::SerializedEvent: Clone + Send + Sync,
{
    pub(super) events: SmartVec<S::SerializedEvent>,
    pub(super) serializer: S,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct WriteError {
    kind: write::ErrorKind,
    source: Option<std::num::TryFromIntError>,
    backtrace: std::backtrace::Backtrace,
}

impl ErrorWithKind for WriteError {
    type Kind = write::ErrorKind;
    fn kind(&self) -> Self::Kind { self.kind }
}

type CommitResult<T> = Result<T, WriteError>;

impl<T, S> WriteStream for InmemWriteStream<T, S>
where
    T: Event,
    S: Serializer<Event = T>,
    S::SerializedEvent: Clone + Send + Sync,
{
    type Event = T;
    type Error = WriteError;

    fn commit_old_or_new(
        &mut self,
        event: revision::OldOrNewRef<'_, Self::Event>,
        condition: write::Condition,
    ) -> impl Future<Output = CommitResult<CommitNumber>> + Send {
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
        condition: write::Condition,
    ) -> impl Future<Output = CommitResult<Option<CommitNumber>>> + Send {
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

fn next_commit_number(
    n_events: usize,
    condition: write::Condition,
) -> CommitResult<CommitNumber> {
    let commit_number =
        u32::try_from(n_events).map_err(|source| WriteError {
            kind: write::ErrorKind::StreamFull,
            source: Some(source),
            backtrace: std::backtrace::Backtrace::capture(),
        })?;
    match condition {
        write::Condition::None => {}
        write::Condition::AssignCommitNumber(assign_commit_number) => {
            if commit_number != assign_commit_number {
                return Err(WriteError {
                    kind: write::ErrorKind::ConditionNotMet,
                    source: None,
                    backtrace: std::backtrace::Backtrace::capture(),
                });
            }
        }
    }
    Ok(commit_number)
}
