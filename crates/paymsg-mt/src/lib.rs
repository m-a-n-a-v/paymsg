//! SWIFT MT message parsing and serialization.
//!
//! This crate provides parsing and serialization for SWIFT MT messages:
//! - MT103 (Customer Credit Transfer)
//! - MT202 (Financial Institution Credit Transfer)
//! - MT940 (Customer Statement)
//! - MT942 (Interim Transaction Report)

use paymsg_core::PaymsgError;

pub mod blocks;
pub mod parser;

// Re-export main types
pub use blocks::{
    ApplicationHeader, BasicHeader, Direction, TextBlock, Trailer, UserHeader,
};
pub use parser::MtMessage;

/// Result type for MT operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;
