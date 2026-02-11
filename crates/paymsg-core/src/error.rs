//! Error types for the paymsg library.

use thiserror::Error;

/// The main error type for paymsg operations.
#[derive(Error, Debug)]
pub enum PaymsgError {
    /// Invalid amount value
    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    /// Invalid currency code
    #[error("invalid currency code: {0}")]
    InvalidCurrency(String),

    /// Invalid BIC (Business Identifier Code)
    #[error("invalid BIC: {0}")]
    InvalidBic(String),

    /// Invalid IBAN (International Bank Account Number)
    #[error("invalid IBAN: {0}")]
    InvalidIban(String),

    /// Invalid date
    #[error("invalid date: {0}")]
    InvalidDate(String),

    /// Invalid message type
    #[error("invalid message type: {0}")]
    InvalidMessageType(String),

    /// Validation error
    #[error("validation error: {0}")]
    ValidationError(String),

    /// Parse error
    #[error("parse error: {0}")]
    ParseError(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Translation error
    #[error("translation error: {0}")]
    TranslationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Spec loading error
    #[error("failed to load spec file {file}: {reason}")]
    SpecLoadError { file: String, reason: String },

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result type for paymsg operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;
