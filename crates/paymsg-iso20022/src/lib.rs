//! ISO 20022 (MX) message parsing and serialization.
//!
//! This crate provides XML parsing and serialization for ISO 20022 messages:
//! - pacs.008.001.10 (Customer Credit Transfer)
//! - pacs.009.001.10 (Financial Institution Credit Transfer)
//! - camt.052.001.10 (Bank to Customer Account Report)
//! - camt.053.001.10 (Bank to Customer Statement)

use paymsg_core::PaymsgError;

/// Result type for ISO 20022 operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

// Placeholder for future implementation
