//! Error type with stable codes.

use std::fmt;

/// Stable error codes. `as_str` gives the code printed by tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// The backend (screen source) failed or is not available here.
    Backend,
    /// No screen with that id.
    UnknownScreen,
    /// No window with that id or title.
    UnknownWindow,
    /// More than one window matches the title.
    AmbiguousWindow,
    /// No primary screen reported by the source.
    NoPrimaryScreen,
    /// The window is minimized and cannot be captured.
    WindowMinimized,
    /// No capture with that number in the session.
    UnknownCapture,
    /// The folder or file could not be read or written.
    Io,
    /// The target file already exists; nothing is overwritten.
    AlreadyExists,
    /// The image data is inconsistent (size does not match pixels).
    BadImage,
    /// PNG encoding failed.
    Png,
    /// ZIP writing failed.
    Zip,
    /// The capture list (`index.tsv`) is damaged.
    BadIndex,
}

impl ErrorCode {
    /// Stable text code, for example `E_UNKNOWN_SCREEN`.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::Backend => "E_BACKEND",
            ErrorCode::UnknownScreen => "E_UNKNOWN_SCREEN",
            ErrorCode::UnknownWindow => "E_UNKNOWN_WINDOW",
            ErrorCode::AmbiguousWindow => "E_AMBIGUOUS_WINDOW",
            ErrorCode::NoPrimaryScreen => "E_NO_PRIMARY_SCREEN",
            ErrorCode::WindowMinimized => "E_WINDOW_MINIMIZED",
            ErrorCode::UnknownCapture => "E_UNKNOWN_CAPTURE",
            ErrorCode::Io => "E_IO",
            ErrorCode::AlreadyExists => "E_ALREADY_EXISTS",
            ErrorCode::BadImage => "E_BAD_IMAGE",
            ErrorCode::Png => "E_PNG",
            ErrorCode::Zip => "E_ZIP",
            ErrorCode::BadIndex => "E_BAD_INDEX",
        }
    }
}

/// The one error type of this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureError {
    code: ErrorCode,
    message: String,
}

impl CaptureError {
    /// Creates an error with a code and a human readable message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        CaptureError {
            code,
            message: message.into(),
        }
    }

    /// The stable code.
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// The message (English, for people).
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for CaptureError {}

impl From<std::io::Error> for CaptureError {
    fn from(e: std::io::Error) -> Self {
        let code = if e.kind() == std::io::ErrorKind::AlreadyExists {
            ErrorCode::AlreadyExists
        } else {
            ErrorCode::Io
        };
        CaptureError::new(code, e.to_string())
    }
}

/// Result alias of this crate.
pub type Result<T> = std::result::Result<T, CaptureError>;
