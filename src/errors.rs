use std::{error::Error as StdError, fmt, sync::PoisonError};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Tried to create a bucket that already exists
    BucketExists,
    /// Tried to get a bucket that does not exist
    BucketMissing,
    /// Tried to delete a key / value pair that does not exist
    KeyValueMissing,
    /// Tried to get a bucket but found a key / value pair instead, or tried to put a key / value pair but found an existing bucket
    IncompatibleValue,
    /// Tried to write to a read only transaction
    ReadOnlyTx,
    /// Wrapper around a [`std::io::Error`] that occurred while opening the file or writing to it
    Io(std::io::ErrorKind, &'static str),
    /// Wrapper around a [`PoisonError`]
    Sync(&'static str),
    /// Error returned when the DB is found to be in an invalid state
    InvalidDB(String),
    /// Errors that can occur during allocation
    Alloc(std::alloc::LayoutError),

    /// Unsupported operation
    Unsupported(&'static str),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BucketExists => write!(f, "Bucket already exists"),
            Error::BucketMissing => write!(f, "Bucket does not exist"),
            Error::KeyValueMissing => write!(f, "Key / Value pair does not exist"),
            Error::IncompatibleValue => write!(f, "Value not compatible"),
            Error::ReadOnlyTx => write!(f, "Cannot write in a read-only transaction"),
            Error::Io(e) => write!(f, "IO Error: {}", e),
            Error::Sync(s) => write!(f, "Sync Error: {}", s),
            Error::InvalidDB(s) => write!(f, "Invalid DB: {}", s),
            Error::Alloc(e) => write!(f, "Allocation error: {}", e),
            Error::Unsupported(s) => write!(f, "Unsupported operation: {}", s),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::alloc::LayoutError> for Error {
    fn from(err: std::alloc::LayoutError) -> Error {
        Error::Alloc(err)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Error {
        Error::Sync("lock poisoned")
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::BucketExists, Error::BucketExists) => true,
            (Error::BucketMissing, Error::BucketMissing) => true,
            (Error::KeyValueMissing, Error::KeyValueMissing) => true,
            (Error::IncompatibleValue, Error::IncompatibleValue) => true,
            (Error::ReadOnlyTx, Error::ReadOnlyTx) => true,
            (Error::Sync(s1), Error::Sync(s2)) => s1 == s2,
            (Error::InvalidDB(s1), Error::InvalidDB(s2)) => s1 == s2,
            (Error::Unsupported(s1), Error::Unsupported(s2)) => s1 == s2,
            _ => false,
        }
    }
}