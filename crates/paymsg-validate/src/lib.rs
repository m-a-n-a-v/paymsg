//! Validation engine for SWIFT MT and ISO 20022 messages.
//!
//! This crate provides:
//! - Schema-level validation (mandatory fields, formats, lengths)
//! - Business rule validation (loaded from JSON specs)
//! - Cross-field validation
//! - Reference data validation (currencies, BICs, IBANs)

use paymsg_core::PaymsgError;

/// Result type for validation operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

// Placeholder for future implementation
