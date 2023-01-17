use std::io;

/// Convenient [`Result`] alias for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Error when parsing the header of a demo file.
#[derive(thiserror::Error, Debug)]
pub enum HeaderParsingError {
    #[error("invalid demo type (expected: {expected}, found: {found})")]
    InvalidDemoType {
        expected: &'static str,
        found: String,
    },
    #[error("invalid demo protocol (expected: {expected}, found: {found})")]
    InvalidDemoProtocol { expected: u32, found: u32 },
    #[error("invalid game (expected: {expected}, found: {found})")]
    InvalidGame {
        expected: &'static str,
        found: String,
    },
}

/// Error type for this library.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Protobuf(#[from] protobuf::Error),
    #[error(transparent)]
    HeaderParsing(#[from] HeaderParsingError),
    #[error("unknown packet command found: {0}")]
    UnknownPacketCommand(u8),
}
