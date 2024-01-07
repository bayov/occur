#[cfg(feature = "backtrace")] use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("stream {id} not found")]
pub struct StreamNotFound<ID> {
    pub id: ID,
    #[cfg(feature = "backtrace")]
    pub backtrace: Backtrace,
}

// TODO: Error handling.
//  * `thiserror` to define types.
//  * Consider generic composition to allow user-defined errors to extend.
//  * App should use `eyre` (anyhow fork) or `error-stack`.
