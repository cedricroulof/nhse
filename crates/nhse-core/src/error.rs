use thiserror::Error;

/// Errors produced while reading, editing or writing save data.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("file not found in save: {0}")]
    MissingFile(String),
    #[error("{0}")]
    UnknownRevision(String),
    #[error("{0}")]
    Invalid(String),
    #[error("index {index} is out of range (max {max})")]
    OutOfRange { index: usize, max: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
