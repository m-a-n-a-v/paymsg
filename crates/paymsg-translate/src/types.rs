/// Translation engine core types
///
/// This module defines the core types for the translation engine including
/// translation results, data loss warnings, and common transformation utilities.
use paymsg_core::PaymsgError;
use serde::{Deserialize, Serialize};

/// Result of a translation operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationResult<T> {
    /// The translated message
    pub message: T,

    /// List of data loss warnings encountered during translation
    pub warnings: Vec<DataLossWarning>,
}

impl<T> TranslationResult<T> {
    /// Create a new translation result with no warnings.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_translate::TranslationResult;
    ///
    /// let result = TranslationResult::new("translated message");
    /// assert!(!result.has_warnings());
    /// ```
    pub fn new(message: T) -> Self {
        Self {
            message,
            warnings: Vec::new(),
        }
    }

    /// Create a translation result with warnings.
    ///
    /// Use this when translation succeeded but some data was lost or modified.
    pub fn with_warnings(message: T, warnings: Vec<DataLossWarning>) -> Self {
        Self { message, warnings }
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: DataLossWarning) {
        self.warnings.push(warning);
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Warning about data loss during translation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataLossWarning {
    /// Field or element that experienced data loss
    pub field_path: String,

    /// Category of data loss
    pub category: DataLossCategory,

    /// Human-readable description of what was lost
    pub description: String,

    /// Original value that was lost or truncated (if applicable)
    pub original_value: Option<String>,
}

impl DataLossWarning {
    /// Create a new data loss warning.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_translate::{DataLossWarning, DataLossCategory};
    ///
    /// let warning = DataLossWarning::new(
    ///     "Field70",
    ///     DataLossCategory::Truncation,
    ///     "Value truncated to 140 characters"
    /// ).with_original_value("Original long value...");
    /// ```
    pub fn new(
        field_path: impl Into<String>,
        category: DataLossCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            field_path: field_path.into(),
            category,
            description: description.into(),
            original_value: None,
        }
    }

    /// Set the original value that was lost
    pub fn with_original_value(mut self, value: impl Into<String>) -> Self {
        self.original_value = Some(value.into());
        self
    }
}

/// Category of data loss during translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataLossCategory {
    /// Field was truncated to fit length limit
    Truncation,

    /// Field not present in target format
    NoEquivalent,

    /// Field has no meaningful mapping (semantic mismatch)
    SemanticMismatch,

    /// Precision loss in numeric conversion
    PrecisionLoss,

    /// Character set limitation
    CharacterSetLimitation,

    /// Optional field omitted due to target format constraints
    OptionalFieldOmitted,
}

/// BIC normalization utilities
pub struct BicNormalizer;

impl BicNormalizer {
    /// Convert BIC8 to BIC11 by appending "XXX"
    ///
    /// MT messages often use BIC8 (head office), while MX messages prefer BIC11.
    /// BIC11 with branch "XXX" also represents head office.
    pub fn to_bic11(bic: &str) -> String {
        if bic.len() == 8 {
            format!("{}XXX", bic)
        } else {
            bic.to_string()
        }
    }

    /// Convert BIC11 to BIC8 by removing "XXX" branch code
    ///
    /// When translating MX to MT, BIC11 with "XXX" branch can be shortened to BIC8.
    pub fn to_bic8(bic: &str) -> String {
        if bic.len() == 11 && bic.ends_with("XXX") {
            bic[..8].to_string()
        } else {
            bic.to_string()
        }
    }

    /// Check if a BIC represents head office
    ///
    /// Either BIC8 or BIC11 ending with "XXX"
    pub fn is_head_office(bic: &str) -> bool {
        bic.len() == 8 || (bic.len() == 11 && bic.ends_with("XXX"))
    }
}

/// Amount conversion utilities
pub struct AmountConverter;

impl AmountConverter {
    /// Convert MT amount format (comma decimal) to MX format (period decimal)
    ///
    /// Example: "1234567,89" → "1234567.89"
    pub fn mt_to_mx(mt_amount: &str) -> String {
        mt_amount.replace(',', ".")
    }

    /// Convert MX amount format (period decimal) to MT format (comma decimal)
    ///
    /// Example: "1234567.89" → "1234567,89"
    pub fn mx_to_mt(mx_amount: &str) -> String {
        mx_amount.replace('.', ",")
    }
}

/// Date conversion utilities
pub struct DateConverter;

impl DateConverter {
    /// Convert MT date (YYMMDD) to MX date (YYYY-MM-DD).
    ///
    /// Uses SWIFT convention: YY >= 50 → 19YY, YY < 50 → 20YY
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_translate::DateConverter;
    ///
    /// let mx_date = DateConverter::mt_to_mx("260210").unwrap();
    /// assert_eq!(mx_date, "2026-02-10");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::ParseError` if the date format is invalid.
    pub fn mt_to_mx(mt_date: &str) -> Result<String, PaymsgError> {
        if mt_date.len() != 6 {
            return Err(PaymsgError::ParseError(format!(
                "Invalid MT date format: expected YYMMDD, got {}",
                mt_date
            )));
        }

        let yy: u32 = mt_date[0..2].parse().map_err(|_| {
            PaymsgError::ParseError(format!("Invalid year in MT date: {}", mt_date))
        })?;
        let mm = &mt_date[2..4];
        let dd = &mt_date[4..6];

        // SWIFT convention: YY >= 50 → 19YY, YY < 50 → 20YY
        let yyyy = if yy >= 50 { 1900 + yy } else { 2000 + yy };

        Ok(format!("{:04}-{}-{}", yyyy, mm, dd))
    }

    /// Convert MX date (YYYY-MM-DD) to MT date (YYMMDD).
    ///
    /// Takes last 2 digits of year for YY.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_translate::DateConverter;
    ///
    /// let mt_date = DateConverter::mx_to_mt("2026-02-10").unwrap();
    /// assert_eq!(mt_date, "260210");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::ParseError` if the date format is invalid.
    pub fn mx_to_mt(mx_date: &str) -> Result<String, PaymsgError> {
        // Parse ISO 8601 date format
        let parts: Vec<&str> = mx_date.split('-').collect();
        if parts.len() != 3 {
            return Err(PaymsgError::ParseError(format!(
                "Invalid MX date format: expected YYYY-MM-DD, got {}",
                mx_date
            )));
        }

        let yyyy: u32 = parts[0].parse().map_err(|_| {
            PaymsgError::ParseError(format!("Invalid year in MX date: {}", mx_date))
        })?;
        let mm = parts[1];
        let dd = parts[2];

        // Take last 2 digits of year
        let yy = yyyy % 100;

        Ok(format!("{:02}{}{}", yy, mm, dd))
    }
}

/// Charge bearer code conversion utilities
pub struct ChargeBearerConverter;

impl ChargeBearerConverter {
    /// Convert MT 71A charge bearer code to MX ChrgBr code.
    ///
    /// MT → MX:
    /// - SHA → SHAR (shared)
    /// - OUR → DEBT (debtor/ordering customer pays all)
    /// - BEN → CRED (creditor/beneficiary pays all)
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::TranslationError` for unknown codes.
    pub fn mt_to_mx(mt_code: &str) -> Result<String, PaymsgError> {
        match mt_code {
            "SHA" => Ok("SHAR".to_string()),
            "OUR" => Ok("DEBT".to_string()),
            "BEN" => Ok("CRED".to_string()),
            _ => Err(PaymsgError::TranslationError(format!(
                "Unknown MT charge bearer code: {}",
                mt_code
            ))),
        }
    }

    /// Convert MX ChrgBr code to MT 71A code.
    ///
    /// MX → MT:
    /// - SHAR → SHA (shared)
    /// - DEBT → OUR (debtor pays all)
    /// - CRED → BEN (creditor pays all)
    /// - SLEV → SHA (service level, default to shared)
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::TranslationError` for unknown codes.
    pub fn mx_to_mt(mx_code: &str) -> Result<String, PaymsgError> {
        match mx_code {
            "SHAR" => Ok("SHA".to_string()),
            "DEBT" => Ok("OUR".to_string()),
            "CRED" => Ok("BEN".to_string()),
            "SLEV" => Ok("SHA".to_string()), // Service level → shared (default)
            _ => Err(PaymsgError::TranslationError(format!(
                "Unknown MX charge bearer code: {}",
                mx_code
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_result_new() {
        let result = TranslationResult::new("test_message");
        assert_eq!(result.message, "test_message");
        assert!(result.warnings.is_empty());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_translation_result_with_warnings() {
        let warning = DataLossWarning::new("Field20", DataLossCategory::Truncation, "Value truncated");
        let result = TranslationResult::with_warnings("message", vec![warning.clone()]);
        assert_eq!(result.message, "message");
        assert_eq!(result.warnings.len(), 1);
        assert!(result.has_warnings());
        assert_eq!(result.warnings[0].field_path, "Field20");
    }

    #[test]
    fn test_bic_normalizer_to_bic11() {
        assert_eq!(BicNormalizer::to_bic11("DEUTDEFF"), "DEUTDEFFXXX");
        assert_eq!(BicNormalizer::to_bic11("DEUTDEFFABC"), "DEUTDEFFABC");
    }

    #[test]
    fn test_bic_normalizer_to_bic8() {
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFFXXX"), "DEUTDEFF");
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFFABC"), "DEUTDEFFABC");
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFF"), "DEUTDEFF");
    }

    #[test]
    fn test_bic_normalizer_is_head_office() {
        assert!(BicNormalizer::is_head_office("DEUTDEFF"));
        assert!(BicNormalizer::is_head_office("DEUTDEFFXXX"));
        assert!(!BicNormalizer::is_head_office("DEUTDEFFABC"));
    }

    #[test]
    fn test_amount_converter_mt_to_mx() {
        assert_eq!(AmountConverter::mt_to_mx("1234567,89"), "1234567.89");
        assert_eq!(AmountConverter::mt_to_mx("100,00"), "100.00");
    }

    #[test]
    fn test_amount_converter_mx_to_mt() {
        assert_eq!(AmountConverter::mx_to_mt("1234567.89"), "1234567,89");
        assert_eq!(AmountConverter::mx_to_mt("100.00"), "100,00");
    }

    #[test]
    fn test_date_converter_mt_to_mx() {
        assert_eq!(DateConverter::mt_to_mx("260210").unwrap(), "2026-02-10");
        assert_eq!(DateConverter::mt_to_mx("500101").unwrap(), "1950-01-01");
        assert_eq!(DateConverter::mt_to_mx("491231").unwrap(), "2049-12-31");
    }

    #[test]
    fn test_date_converter_mx_to_mt() {
        assert_eq!(DateConverter::mx_to_mt("2026-02-10").unwrap(), "260210");
        assert_eq!(DateConverter::mx_to_mt("1950-01-01").unwrap(), "500101");
        assert_eq!(DateConverter::mx_to_mt("2049-12-31").unwrap(), "491231");
    }

    #[test]
    fn test_charge_bearer_converter_mt_to_mx() {
        assert_eq!(ChargeBearerConverter::mt_to_mx("SHA").unwrap(), "SHAR");
        assert_eq!(ChargeBearerConverter::mt_to_mx("OUR").unwrap(), "DEBT");
        assert_eq!(ChargeBearerConverter::mt_to_mx("BEN").unwrap(), "CRED");
        assert!(ChargeBearerConverter::mt_to_mx("XXX").is_err());
    }

    #[test]
    fn test_charge_bearer_converter_mx_to_mt() {
        assert_eq!(ChargeBearerConverter::mx_to_mt("SHAR").unwrap(), "SHA");
        assert_eq!(ChargeBearerConverter::mx_to_mt("DEBT").unwrap(), "OUR");
        assert_eq!(ChargeBearerConverter::mx_to_mt("CRED").unwrap(), "BEN");
        assert_eq!(ChargeBearerConverter::mx_to_mt("SLEV").unwrap(), "SHA");
        assert!(ChargeBearerConverter::mx_to_mt("XXXX").is_err());
    }
}
