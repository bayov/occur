#[cfg(feature = "backtrace")] use std::backtrace::Backtrace;
use std::error::Error;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Error, Debug)]
#[error("stream {id} not found")]
pub struct StreamNotFound<ID> {
    pub id: ID,
    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}
