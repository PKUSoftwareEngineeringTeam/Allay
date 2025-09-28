use thiserror::Error;

/// Represents the different types of errors that can occur in the server.
///
/// # Variants
///
/// - `IOError(std::io::Error)` - An error that occurs due to an I/O operation. This variant wraps
///   around the standard library's [std::io::Error] type, providing a more specific context for
///   I/O-related failures within the server.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Represents the result of a server operation.
pub type ServerResult<T> = Result<T, ServerError>;
