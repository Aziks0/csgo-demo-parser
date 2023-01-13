use std::io;

/// Convenient [`Result`] alias for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for this library.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Protobuf(#[from] protobuf::Error),
}
