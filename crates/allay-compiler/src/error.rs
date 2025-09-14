//! Error types for the Allay compiler.

use thiserror::Error;

/// Errors that can occur during compilation.
#[derive(Debug, Error)]
pub enum CompileError {
    /// IO error when reading files.
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    /// Unsupported file type. Only markdown (.md) and HTML (.html) are supported.
    #[error("File type not supported: {0}")]
    FileTypeNotSupported(String),

    /// Template parsing error.
    #[error("Template parsing error: {0}")]
    ParsingError(#[from] Box<pest::error::Error<crate::parser::Rule>>),
}
