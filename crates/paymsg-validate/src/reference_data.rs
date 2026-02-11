//! Reference data validators for currency, BIC, IBAN, and cross-field validation.

use paymsg_core::{Bic, Iban, specs::SpecRegistries};
use paymsg_iso20022::pacs008;
use crate::types::{ValidationIssue, ValidationResult, Validator, Severity};
use std::str::FromStr;
use regex::Regex;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

/// Validator for reference data and cross-field validation.
pub struct ReferenceDataValidator {
    specs: SpecRegistries,
}

impl ReferenceDataValidator {
    /// Create a new reference data validator.
    ///
    /// # Arguments
    /// * `specs` - Loaded spec registries (currencies, countries, IBAN formats)
    pub fn new(specs: SpecRegistries) -> Self {
        Self { specs }
    }

    /// Validate a currency code against ISO 4217.
    ///
    /// Returns issues if:
    /// - Currency code doesn't exist in ISO 4217
    /// - Currency code is not active
    fn validate_currency_code(&self, code: &str, field_path: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check if currency exists
        match self.specs.currencies.lookup_by_code(code) {
            Some(spec) => {
                if !spec.is_active {
                    issues.push(ValidationIssue {
                        id: "REF-001".to_string(),
                        severity: Severity::Warning,
                        field_path: Some(field_path.to_string()),
                        message: format!("Currency '{}' is not active", code),
                        suggestion: Some("Use an active ISO 4217 currency code".to_string()),
                    });
                }
            }
            None => {
                issues.push(ValidationIssue {
                    id: "REF-002".to_string(),
                    severity: Severity::Error,
                    field_path: Some(field_path.to_string()),
                    message: format!("Currency '{}' is not a valid ISO 4217 code", code),
                    suggestion: Some("Use a valid ISO 4217 currency code (e.g., USD, EUR, GBP)".to_string()),
                });
            }
        }

        issues
    }

    /// Validate amount decimal places match currency specification.
    ///
    /// Returns issues if:
    /// - Decimal places exceed currency spec (e.g., 3 decimals for USD which requires 2)
    /// - Decimal places are less than currency spec (minor issue)
    fn validate_currency_decimal_places(&self, amount: Decimal, currency: &str, field_path: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some(spec) = self.specs.currencies.lookup_by_code(currency) {
            let amount_decimals = amount.scale();
            let required_decimals = spec.decimal_places as u32;

            if amount_decimals > required_decimals {
                issues.push(ValidationIssue {
                    id: "REF-003".to_string(),
                    severity: Severity::Error,
                    field_path: Some(field_path.to_string()),
                    message: format!(
                        "Amount {} has {} decimal places, but {} requires {}",
                        amount, amount_decimals, currency, required_decimals
                    ),
                    suggestion: Some(format!(
                        "Round amount to {} decimal places for {}",
                        required_decimals, currency
                    )),
                });
            } else if amount_decimals < required_decimals {
                issues.push(ValidationIssue {
                    id: "REF-004".to_string(),
                    severity: Severity::Info,
                    field_path: Some(field_path.to_string()),
                    message: format!(
                        "Amount {} has {} decimal places, but {} typically uses {}",
                        amount, amount_decimals, currency, required_decimals
                    ),
                    suggestion: Some(format!(
                        "Consider using {} decimal places for {}",
                        required_decimals, currency
                    )),
                });
            }
        }

        issues
    }

    /// Validate BIC structure and country code.
    ///
    /// Returns issues if:
    /// - BIC format is invalid (not 8 or 11 chars, invalid characters)
    /// - Country code doesn't exist in ISO 3166
    fn validate_bic(&self, bic: &str, field_path: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Try to parse BIC
        match Bic::from_str(bic) {
            Ok(bic_parsed) => {
                // Check country code exists
                let country = &bic_parsed.country;
                if self.specs.countries.lookup_by_alpha2(country).is_none() {
                    issues.push(ValidationIssue {
                        id: "REF-006".to_string(),
                        severity: Severity::Error,
                        field_path: Some(field_path.to_string()),
                        message: format!("BIC contains invalid country code '{}'", country),
                        suggestion: Some("Use a valid ISO 3166 country code".to_string()),
                    });
                }
            }
            Err(_) => {
                issues.push(ValidationIssue {
                    id: "REF-005".to_string(),
                    severity: Severity::Error,
                    field_path: Some(field_path.to_string()),
                    message: format!("BIC '{}' has invalid format", bic),
                    suggestion: Some("BIC must be 8 or 11 characters: INST(4) + COUNTRY(2) + LOCATION(2) + [BRANCH(3)]".to_string()),
                });
            }
        }

        issues
    }

    /// Validate IBAN structure, check digits, and country-specific format.
    ///
    /// Returns issues if:
    /// - IBAN format is invalid (check digit validation fails)
    /// - IBAN length doesn't match country specification
    /// - BBAN format doesn't match country regex pattern
    fn validate_iban(&self, iban: &str, field_path: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Try to parse IBAN (this validates check digits via Mod-97)
        match Iban::from_str(iban) {
            Ok(iban_parsed) => {
                let country = &iban_parsed.country;

                // Check country-specific format
                if let Some(format_spec) = self.specs.iban_formats.lookup(country) {
                    let iban_str = iban_parsed.to_string();

                    // Validate length
                    if iban_str.len() != format_spec.length {
                        issues.push(ValidationIssue {
                            id: "REF-008".to_string(),
                            severity: Severity::Error,
                            field_path: Some(field_path.to_string()),
                            message: format!(
                                "IBAN for {} should be {} characters, but got {}",
                                country, format_spec.length, iban_str.len()
                            ),
                            suggestion: Some(format!("Verify IBAN length for country {}", country)),
                        });
                    }

                    // Validate BBAN format if regex pattern available
                    if !format_spec.bban_format.is_empty() {
                        // BBAN is everything after country code and check digits (first 4 chars)
                        let bban = &iban_str[4..];

                        if let Ok(regex) = Regex::new(&format_spec.bban_format) {
                            if !regex.is_match(bban) {
                                issues.push(ValidationIssue {
                                    id: "REF-009".to_string(),
                                    severity: Severity::Error,
                                    field_path: Some(field_path.to_string()),
                                    message: format!("IBAN BBAN format invalid for country {}", country),
                                    suggestion: Some(format!(
                                        "BBAN must match pattern: {}. Example: {}",
                                        format_spec.bban_format,
                                        format_spec.example
                                    )),
                                });
                            }
                        }
                    }
                } else {
                    issues.push(ValidationIssue {
                        id: "REF-010".to_string(),
                        severity: Severity::Warning,
                        field_path: Some(field_path.to_string()),
                        message: format!("No IBAN format specification found for country '{}'", country),
                        suggestion: Some("Country may not support IBAN".to_string()),
                    });
                }
            }
            Err(_) => {
                issues.push(ValidationIssue {
                    id: "REF-007".to_string(),
                    severity: Severity::Error,
                    field_path: Some(field_path.to_string()),
                    message: format!("IBAN '{}' has invalid format or check digits", iban),
                    suggestion: Some("Verify IBAN check digits using Mod-97 algorithm".to_string()),
                });
            }
        }

        issues
    }

    /// Validate date is not in the past (for settlement dates).
    ///
    /// Returns issues if:
    /// - Date is in the past
    /// - Date is too far in the future (> 1 year)
    fn validate_settlement_date(&self, date_str: &str, field_path: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let today = Utc::now().date_naive();

            if date < today {
                issues.push(ValidationIssue {
                    id: "REF-011".to_string(),
                    severity: Severity::Warning,
                    field_path: Some(field_path.to_string()),
                    message: format!("Settlement date {} is in the past", date),
                    suggestion: Some("Settlement dates should typically be today or in the future".to_string()),
                });
            }

            // Check if date is too far in future (> 1 year)
            let max_future = today + chrono::Duration::days(365);
            if date > max_future {
                issues.push(ValidationIssue {
                    id: "REF-012".to_string(),
                    severity: Severity::Warning,
                    field_path: Some(field_path.to_string()),
                    message: format!("Settlement date {} is more than 1 year in the future", date),
                    suggestion: Some("Verify settlement date is correct".to_string()),
                });
            }
        }

        issues
    }

    /// Validate pacs.008 message cross-field logic.
    fn validate_pacs008_cross_fields(&self, doc: &pacs008::Document) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        let msg = &doc.fi_to_fi_customer_credit_transfer;

        // Validate each transaction
        for (idx, tx) in msg.credit_transfer_transaction_information.iter().enumerate() {
            let tx_prefix = format!("CdtTrfTxInf[{}]", idx);

            // Validate settlement amount currency
            let settlement_ccy = &tx.interbank_settlement_amount.currency;
            issues.extend(self.validate_currency_code(settlement_ccy, &format!("{}/IntrBkSttlmAmt/@Ccy", tx_prefix)));
            issues.extend(self.validate_currency_decimal_places(
                tx.interbank_settlement_amount.value,
                settlement_ccy,
                &format!("{}/IntrBkSttlmAmt", tx_prefix),
            ));

            // If instructed amount present, validate it
            if let Some(ref instd_amt) = tx.instructed_amount {
                let instd_ccy = &instd_amt.currency;
                issues.extend(self.validate_currency_code(instd_ccy, &format!("{}/InstdAmt/@Ccy", tx_prefix)));
                issues.extend(self.validate_currency_decimal_places(
                    instd_amt.value,
                    instd_ccy,
                    &format!("{}/InstdAmt", tx_prefix),
                ));

                // If currencies differ, exchange rate should be present
                if settlement_ccy != instd_ccy && tx.exchange_rate.is_none() {
                    issues.push(ValidationIssue {
                        id: "REF-013".to_string(),
                        severity: Severity::Error,
                        field_path: Some(format!("{}/XchgRate", tx_prefix)),
                        message: format!(
                            "Exchange rate required when settlement currency ({}) differs from instructed currency ({})",
                            settlement_ccy, instd_ccy
                        ),
                        suggestion: Some("Provide exchange rate when currencies differ".to_string()),
                    });
                }
            }

            // Validate settlement date if present
            if let Some(ref sttlm_dt) = tx.interbank_settlement_date {
                issues.extend(self.validate_settlement_date(sttlm_dt, &format!("{}/IntrBkSttlmDt", tx_prefix)));
            }

            // Validate BICs
            let dbtr_agt = &tx.debtor_agent;
            if let Some(ref bic) = &dbtr_agt.financial_institution_id.bic {
                issues.extend(self.validate_bic(bic, &format!("{}/DbtrAgt/FinInstnId/BICFI", tx_prefix)));
            }

            let cdtr_agt = &tx.creditor_agent;
            if let Some(ref bic) = &cdtr_agt.financial_institution_id.bic {
                issues.extend(self.validate_bic(bic, &format!("{}/CdtrAgt/FinInstnId/BICFI", tx_prefix)));
            }

            // Validate IBANs
            let dbtr_acct = &tx.debtor_account;
            if let Some(ref iban) = &dbtr_acct.id.iban {
                issues.extend(self.validate_iban(iban, &format!("{}/DbtrAcct/Id/IBAN", tx_prefix)));
            }

            let cdtr_acct = &tx.creditor_account;
            if let Some(ref iban) = &cdtr_acct.id.iban {
                issues.extend(self.validate_iban(iban, &format!("{}/CdtrAcct/Id/IBAN", tx_prefix)));
            }
        }

        // Validate group header settlement amount if present
        if let Some(ref total_amt) = msg.group_header.total_interbank_settlement_amount {
            let total_ccy = &total_amt.currency;
            issues.extend(self.validate_currency_code(total_ccy, "GrpHdr/TtlIntrBkSttlmAmt/@Ccy"));
            issues.extend(self.validate_currency_decimal_places(
                total_amt.value,
                total_ccy,
                "GrpHdr/TtlIntrBkSttlmAmt",
            ));
        }

        // Validate instructing/instructed agents BICs
        let instg_agt = &msg.group_header.instructing_agent;
        if let Some(ref bic) = &instg_agt.financial_institution_id.bic {
            issues.extend(self.validate_bic(bic, "GrpHdr/InstgAgt/FinInstnId/BICFI"));
        }

        let instd_agt = &msg.group_header.instructed_agent;
        if let Some(ref bic) = &instd_agt.financial_institution_id.bic {
            issues.extend(self.validate_bic(bic, "GrpHdr/InstdAgt/FinInstnId/BICFI"));
        }

        issues
    }
}

impl Validator<pacs008::Document> for ReferenceDataValidator {
    fn validate(&self, message: &pacs008::Document) -> ValidationResult {
        let issues = self.validate_pacs008_cross_fields(message);
        ValidationResult { issues }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paymsg_core::specs::SpecLoader;
    use std::path::PathBuf;

    fn get_specs_path() -> PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let crate_dir = PathBuf::from(manifest_dir);
        // From crates/paymsg-validate go up 2 levels to workspace root, then to sibling ../paymsg-specs
        crate_dir.parent().unwrap().parent().unwrap().parent().unwrap().join("paymsg-specs")
    }

    fn load_specs() -> SpecRegistries {
        let specs_path = get_specs_path();
        let loader = SpecLoader::new(Some(specs_path));
        loader.load_all().expect("Failed to load specs")
    }

    #[test]
    fn test_validate_currency_code_valid() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        let issues = validator.validate_currency_code("USD", "Test/Currency");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_currency_code_invalid() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        let issues = validator.validate_currency_code("XXX", "Test/Currency");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "REF-002");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_validate_currency_decimal_places_correct() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // USD has 2 decimal places
        let amount = Decimal::from_str("100.50").unwrap();
        let issues = validator.validate_currency_decimal_places(amount, "USD", "Test/Amount");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_currency_decimal_places_too_many() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // USD has 2 decimal places, 3 is too many
        let amount = Decimal::from_str("100.123").unwrap();
        let issues = validator.validate_currency_decimal_places(amount, "USD", "Test/Amount");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "REF-003");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_validate_bic_valid() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        let issues = validator.validate_bic("DEUTDEFF", "Test/BIC");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_bic_invalid_format() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        let issues = validator.validate_bic("INVALID", "Test/BIC");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "REF-005");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_validate_iban_valid() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // German IBAN
        let issues = validator.validate_iban("DE89370400440532013000", "Test/IBAN");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_iban_invalid_check_digits() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // Invalid check digits (should be 89, not 99)
        let issues = validator.validate_iban("DE99370400440532013000", "Test/IBAN");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "REF-007");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_validate_settlement_date_future() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // Date in the future (1 week from now)
        let future_date = (Utc::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
        let issues = validator.validate_settlement_date(&future_date, "Test/Date");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_settlement_date_past() {
        let specs = load_specs();
        let validator = ReferenceDataValidator::new(specs);

        // Date in the past
        let issues = validator.validate_settlement_date("2020-01-01", "Test/Date");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "REF-011");
        assert_eq!(issues[0].severity, Severity::Warning);
    }
}
