//! Code generation from XSD schemas (build-time, optional).
//!
//! This crate provides build-time code generation from ISO 20022 XSD schemas
//! to Rust types. This is an optional, stretch-goal feature.

use paymsg_core::PaymsgError;

/// Result type for codegen operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

// Placeholder for future implementation
