//! Translation engine for converting between SWIFT MT and ISO 20022 MX messages.
//!
//! This crate provides bidirectional translation:
//! - MT103 ↔ pacs.008
//! - MT202 ↔ pacs.009
//! - MT940 ↔ camt.053
//! - MT942 ↔ camt.052

use paymsg_core::PaymsgError;

/// Result type for translation operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

// Placeholder for future implementation
