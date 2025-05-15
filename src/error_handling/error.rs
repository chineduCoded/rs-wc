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

#[cfg(test)]
mod error_tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_conversions() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "test error");
        let wc_error: WcError = io_error.into();
        assert!(matches!(wc_error, WcError::Io(_)));

        // Use a truly invalid UTF-8 sequence
        let utf8_error = std::str::from_utf8(&[0xC0, 0x80]).unwrap_err(); // Overlong encoding of NUL byte
        let wc_error: WcError = utf8_error.into();
        assert!(matches!(wc_error, WcError::Utf8(_)));
    }

    #[test]
    fn test_custom_errors() {
        let not_found = WcError::file_not_found("test.txt");
        assert_eq!(not_found.to_string(), "File not found: test.txt");

        let denied = WcError::permission_denied("/root/file");
        assert_eq!(denied.to_string(), "Permission denied: /root/file");
    }
}