//! Errors that can occur when using this crate.
use byteseeker::Error as SeekError;
use thiserror::Error;

use std::io::Error as IoError;
use std::result::Result as StdResult;

/// A type alias for [`Result`]<T, [`enum@ErrorKind`]>.
///
/// [`Result`]: std::result::Result
pub type Result<T> = StdResult<T, ErrorKind>;

/// The concrete type of an error.
#[derive(Error, Debug)]
pub enum ErrorKind {
    /// Occurs if the given input to be merged is empty.
    #[error("expected at least one source to be merged, but found nothing")]
    NothingPassed,

    /// Occurs if the configured skip option is invalid.
    #[error("the skip options given are not valid to apply to the given sources")]
    InvalidSkip,

    /// Occurs if the given path is not a valid file path.
    #[error("the path provided at index {0} is not a valid file path")]
    InvalidPath(usize),

    /// Represents an error that originates from [`ByteSeeker`].
    ///
    /// [`ByteSeeker`]: byteseeker::ByteSeeker
    #[error(transparent)]
    ByteSeek(#[from] SeekError),

    /// Represents an [`I/O error`].
    ///
    /// [`I/O error`]: std::io::Error
    #[error(transparent)]
    Io(#[from] IoError),
}
