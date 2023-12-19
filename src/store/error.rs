#[cfg(feature = "backtrace")] use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("stream {id} not found")]
pub struct StreamNotFound<ID> {
    pub id: ID,
    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}
