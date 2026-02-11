//! Core types and utilities for the paymsg library.
//!
//! This crate provides foundational types used throughout the paymsg library:
//! - [`Amount`]: Financial amounts with precise decimal arithmetic
//! - [`Currency`]: ISO 4217 currency codes
//! - [`Bic`]: Business Identifier Codes (SWIFT codes)
//! - [`Iban`]: International Bank Account Numbers
//! - [`Date`] and [`DateTime`]: Date and time types for financial messages
//! - [`MessageType`]: Enumeration of supported SWIFT MT and ISO 20022 MX message types
//! - [`PaymsgError`]: Error types for the library

pub mod amount;
pub mod bic;
pub mod currency;
pub mod date;
pub mod error;
pub mod iban;
pub mod message_type;

// Re-export main types for convenience
pub use amount::Amount;
pub use bic::Bic;
pub use currency::Currency;
pub use date::{Date, DateTime};
pub use error::{PaymsgError, Result};
pub use iban::Iban;
pub use message_type::{MessageCategory, MessageType};

/// Prelude module for commonly used types.
pub mod prelude {
    pub use crate::{
        Amount, Bic, Currency, Date, DateTime, Iban, MessageCategory, MessageType, PaymsgError,
        Result,
    };
}
