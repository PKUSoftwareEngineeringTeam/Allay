//! Error types for the Allay compiler.

use allay_base::{data::AllayDataError, file::FileError};
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Template parsing error.
    #[error("Template parsing error: {0}")]
    ParsingError(#[from] Box<pest::error::Error<crate::parse::Rule>>),

    /// Short code is inconsistent, i.e., opening and closing tags do not match.
    #[error("Short code {0} is inconsistent")]
    ShortCodeInconsistent(String),

    /// Invalid number format.
    #[error("Invalid number: {0}, error: {1}")]
    InvalidNumber(String, std::num::ParseIntError),
}

/// The result type for parsing.
///
/// This is a type alias for [`Result<T, CompileError>`].
pub type ParseResult<T> = Result<T, ParseError>;

/// Errors that can occur during interpretation.
#[derive(Debug, Error)]
pub enum InterpretError {
    /// Data error when accessing data
    #[error("{0}")]
    DataError(#[from] AllayDataError),

    /// Field not found in the data
    #[error("Field not found: {0:?}")]
    FieldNotFound(String),

    /// Index out of bounds when accessing a list
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}

/// The result type for interpreter.
///
/// This is a type alias for [`Result<T, InterpretError>`]
pub type InterpretResult<T> = Result<T, InterpretError>;

/// Errors that can occur during compilation (parsing + interpretation).
#[derive(Debug, Error)]
pub enum CompileError {
    /// IO error when reading files.
    #[error("File error: {0}")]
    FileError(#[from] FileError),
    /// Unsupported file type. Only markdown (.md) and HTML (.html) are supported.
    #[error("File type not supported: {0}")]
    FileTypeNotSupported(String),
    /// Parsing error
    #[error("{0}")]
    ParseError(#[from] ParseError),
    /// Interpretation error
    #[error("{0}")]
    InterpretError(#[from] InterpretError),
}

/// The result type for compilation.
///
/// This is a type alias for [`Result<T, CompileError>`].
pub type CompileResult<T> = Result<T, CompileError>;
