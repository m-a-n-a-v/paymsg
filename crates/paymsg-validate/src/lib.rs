//! Validation engine for SWIFT MT and ISO 20022 messages.
//!
//! This crate provides:
//! - Schema-level validation (mandatory fields, formats, lengths)
//! - Business rule validation (loaded from JSON specs)
//! - Cross-field validation
//! - Reference data validation (currencies, BICs, IBANs)

pub mod types;
pub mod mt_schema;
pub mod mx_schema;
pub mod swift_charset;
pub mod business_rules;

pub use types::*;
pub use mt_schema::MtSchemaValidator;
pub use mx_schema::MxSchemaValidator;
pub use swift_charset::{SwiftCharsetValidator, SwiftCharsets};
pub use business_rules::{BusinessRule, BusinessRuleSet, BusinessRuleValidator, load_business_rules};

use paymsg_core::PaymsgError;

/// Result type for validation operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;
