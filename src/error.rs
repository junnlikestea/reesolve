use std::error::Error;
use std::fmt;
use std::io;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ReeError {
    Serde(serde_json::Error),
    CsvErr(String),
    Io(io::Error),
    SendErr(String),
    JoinErr(tokio::task::JoinError),
    ParseInt(std::num::ParseIntError),
}

impl Error for ReeError {}

impl fmt::Display for ReeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReeError::Serde(ref err) => err.fmt(f),
            ReeError::Io(ref err) => err.fmt(f),
            ReeError::SendErr(ref err) => err.fmt(f),
            ReeError::JoinErr(ref err) => err.fmt(f),
            ReeError::CsvErr(ref err) => err.fmt(f),
            ReeError::ParseInt(ref err) => err.fmt(f),
        }
    }
}

impl From<serde_json::Error> for ReeError {
    fn from(err: serde_json::Error) -> Self {
        ReeError::Serde(err)
    }
}

impl From<io::Error> for ReeError {
    fn from(err: io::Error) -> Self {
        ReeError::Io(err)
    }
}

impl From<tokio::task::JoinError> for ReeError {
    fn from(err: tokio::task::JoinError) -> Self {
        ReeError::JoinErr(err)
    }
}

impl<T> From<mpsc::error::SendError<T>> for ReeError {
    fn from(err: mpsc::error::SendError<T>) -> Self {
        ReeError::SendErr(err.to_string())
    }
}

impl<T> From<csv::IntoInnerError<T>> for ReeError {
    fn from(err: csv::IntoInnerError<T>) -> Self {
        ReeError::CsvErr(err.to_string())
    }
}

impl From<std::num::ParseIntError> for ReeError {
    fn from(err: std::num::ParseIntError) -> Self {
        ReeError::ParseInt(err)
    }
}
