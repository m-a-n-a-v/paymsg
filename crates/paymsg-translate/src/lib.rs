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

// Core types
pub mod types;

// MT to MX translators
// TODO: Complete implementation requires extending pacs008 structures with additional fields
// See progress.txt for details on missing fields (AdrLine, ClrSysMmbId, etc.)
#[cfg(feature = "incomplete")]
pub mod mt103_to_pacs008;

// MX to MT translators
pub mod pacs008_to_mt103;

// MT202 ↔ pacs.009 translators
pub mod mt202_to_pacs009;
pub mod pacs009_to_mt202;

// MT940 ↔ camt.053 translators
pub mod mt940_to_camt053;
pub mod camt053_to_mt940;

// MT942 ↔ camt.052 translators
pub mod mt942_to_camt052;
pub mod camt052_to_mt942;

// Re-exports
pub use types::{
    AmountConverter, BicNormalizer, ChargeBearerConverter, DataLossCategory, DataLossWarning,
    DateConverter, TranslationResult,
};

// Translation functions
// TODO: Enable once pacs008 structures are extended
#[cfg(feature = "incomplete")]
pub use mt103_to_pacs008::translate as translate_mt103_to_pacs008;

pub use pacs008_to_mt103::translate as translate_pacs008_to_mt103;

// MT202 ↔ pacs.009 translation functions
pub use mt202_to_pacs009::translate as translate_mt202_to_pacs009;
pub use pacs009_to_mt202::translate as translate_pacs009_to_mt202;

// MT940 ↔ camt.053 translation functions
pub use mt940_to_camt053::translate as translate_mt940_to_camt053;
pub use camt053_to_mt940::translate as translate_camt053_to_mt940;

// MT942 ↔ camt.052 translation functions
pub use mt942_to_camt052::translate as translate_mt942_to_camt052;
pub use camt052_to_mt942::translate as translate_camt052_to_mt942;
