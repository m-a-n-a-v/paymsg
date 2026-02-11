//! SWIFT MT message parsing and serialization.
//!
//! This crate provides parsing and serialization for SWIFT MT messages:
//! - MT103 (Customer Credit Transfer)
//! - MT202 (Financial Institution Credit Transfer)
//! - MT940 (Customer Statement)
//! - MT942 (Interim Transaction Report)

use paymsg_core::PaymsgError;

pub mod blocks;
pub mod fields;
pub mod parser;

// Re-export main types
pub use blocks::{
    ApplicationHeader, BasicHeader, Direction, TextBlock, Trailer, UserHeader,
};
pub use fields::{
    load_all_mt_specs, load_mt_spec, parse_block4_fields, parse_mt_amount, parse_mt_date_yymmdd,
    MtField, MtFieldSpec, MtMessageSpec, MtSubfieldSpec,
};
pub use parser::MtMessage;

/// Result type for MT operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;
