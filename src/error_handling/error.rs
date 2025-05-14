use thiserror::Error;

#[derive(Debug, Error)]
pub enum WcError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Memory map error: {0}")]
    Mmap(String),
}

impl WcError {
    pub fn invalid_argument<T: Into<String>>(msg: T) -> Self {
        WcError::InvalidArgument(msg.into())
    }

    pub fn file_not_found<T: Into<String>>(file: T) -> Self {
        WcError::FileNotFound(file.into())
    }

    pub fn permission_denied<T: Into<String>>(file: T) -> Self {
        WcError::PermissionDenied(file.into())
    }
}


pub type WcResult<T> = Result<T, WcError>;