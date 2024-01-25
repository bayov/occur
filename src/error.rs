/// A standard error type that also has a `.kind()` method.
///
/// Used to create errors that follow a similar pattern to [`std::io::Error`],
/// which has a `.kind()` method returning an [`std::io::ErrorKind`].
#[allow(clippy::module_name_repetitions)]
pub trait ErrorWithKind: std::error::Error {
    /// The kind type; should be an enum with possible error kinds as variants,
    /// and typically a variant named `Other` for all other unexpected errors.
    type Kind;

    /// Returns the kind of this error.
    fn kind(&self) -> Self::Kind;
}
