//! MX (ISO 20022) message schema validation.

use crate::{ValidationIssue, ValidationResult, Validator};
use paymsg_iso20022::{camt052, camt053, pacs008, pacs009};

/// MX schema validator.
pub struct MxSchemaValidator;

impl MxSchemaValidator {
    /// Create a new MX schema validator.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MxSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate pacs.008 message.
impl Validator<pacs008::Document> for MxSchemaValidator {
    fn validate(&self, message: &pacs008::Document) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate Group Header
        let grp_hdr = &message.fi_to_fi_customer_credit_transfer.group_header;

        // MsgId is mandatory
        if grp_hdr.message_id.trim().is_empty() {
            result.add_issue(
                ValidationIssue::error("MX_MISSING_MSG_ID", "Missing or empty MsgId")
                    .with_field_path("GrpHdr.MsgId")
                    .with_suggestion("MsgId must be a unique message identifier"),
            );
        }

        // CreDtTm is mandatory (always present in struct, but check it's not default)
        if grp_hdr.creation_date_time.is_empty() {
            result.add_issue(
                ValidationIssue::error("MX_MISSING_CRE_DT_TM", "Missing or empty CreDtTm")
                    .with_field_path("GrpHdr.CreDtTm")
                    .with_suggestion("CreDtTm must be a valid ISO 8601 datetime"),
            );
        }

        // NbOfTxs must match actual transaction count
        let nb_of_txs = grp_hdr.number_of_transactions.parse::<usize>().unwrap_or(0);
        let actual_txs = message.fi_to_fi_customer_credit_transfer.credit_transfer_transaction_information.len();
        if nb_of_txs != actual_txs {
            result.add_issue(
                ValidationIssue::error(
                    "MX_TRANSACTION_COUNT_MISMATCH",
                    format!(
                        "NbOfTxs ({}) does not match actual transaction count ({})",
                        nb_of_txs, actual_txs
                    ),
                )
                .with_field_path("GrpHdr.NbOfTxs")
                .with_suggestion(format!("NbOfTxs should be {}", actual_txs)),
            );
        }

        // Validate each transaction
        for (i, tx) in message
            .fi_to_fi_customer_credit_transfer
            .credit_transfer_transaction_information
            .iter()
            .enumerate()
        {
            let tx_path = format!("CdtTrfTxInf[{}]", i);

            // Payment identification is mandatory
            if tx.payment_id.end_to_end_id.trim().is_empty() {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_MISSING_END_TO_END_ID",
                        "Missing or empty EndToEndId",
                    )
                    .with_field_path(format!("{}.PmtId.EndToEndId", tx_path))
                    .with_suggestion("EndToEndId is mandatory for transaction identification"),
                );
            }

            // IntrBkSttlmAmt is mandatory and must be positive
            if tx.interbank_settlement_amount.value <= rust_decimal::Decimal::ZERO {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_INVALID_AMOUNT",
                        "IntrBkSttlmAmt must be greater than zero",
                    )
                    .with_field_path(format!("{}.IntrBkSttlmAmt", tx_path))
                    .with_suggestion("Amount must be a positive value"),
                );
            }

            // Currency must be 3 uppercase letters
            if !is_valid_currency_code(&tx.interbank_settlement_amount.currency) {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_INVALID_CURRENCY",
                        format!("Invalid currency code: {}", tx.interbank_settlement_amount.currency),
                    )
                    .with_field_path(format!("{}.IntrBkSttlmAmt.Ccy", tx_path))
                    .with_suggestion("Currency must be a valid ISO 4217 3-letter code"),
                );
            }

            // Debtor name is mandatory
            if tx.debtor.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_DEBTOR", "Missing or empty Debtor name")
                        .with_field_path(format!("{}.Dbtr.Nm", tx_path))
                        .with_suggestion("Debtor name is mandatory"),
                );
            }

            // Creditor name is mandatory
            if tx.creditor.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_CREDITOR", "Missing or empty Creditor name")
                        .with_field_path(format!("{}.Cdtr.Nm", tx_path))
                        .with_suggestion("Creditor name is mandatory"),
                );
            }
        }

        result
    }
}

/// Validate pacs.009 message.
impl Validator<pacs009::Document> for MxSchemaValidator {
    fn validate(&self, message: &pacs009::Document) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate Group Header
        let grp_hdr = &message.fi_credit_transfer.group_header;

        // MsgId is mandatory
        if grp_hdr.message_id.trim().is_empty() {
            result.add_issue(
                ValidationIssue::error("MX_MISSING_MSG_ID", "Missing or empty MsgId")
                    .with_field_path("GrpHdr.MsgId")
                    .with_suggestion("MsgId must be a unique message identifier"),
            );
        }

        // NbOfTxs must match actual transaction count
        let nb_of_txs = grp_hdr.number_of_transactions.parse::<usize>().unwrap_or(0);
        let actual_txs = message.fi_credit_transfer.credit_transfer_transaction_information.len();
        if nb_of_txs != actual_txs {
            result.add_issue(
                ValidationIssue::error(
                    "MX_TRANSACTION_COUNT_MISMATCH",
                    format!(
                        "NbOfTxs ({}) does not match actual transaction count ({})",
                        nb_of_txs, actual_txs
                    ),
                )
                .with_field_path("GrpHdr.NbOfTxs")
                .with_suggestion(format!("NbOfTxs should be {}", actual_txs)),
            );
        }

        // Validate each transaction
        for (i, tx) in message.fi_credit_transfer.credit_transfer_transaction_information.iter().enumerate() {
            let tx_path = format!("CdtTrfTxInf[{}]", i);

            // IntrBkSttlmAmt is mandatory and must be positive
            if tx.interbank_settlement_amount.value <= rust_decimal::Decimal::ZERO {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_INVALID_AMOUNT",
                        "IntrBkSttlmAmt must be greater than zero",
                    )
                    .with_field_path(format!("{}.IntrBkSttlmAmt", tx_path))
                    .with_suggestion("Amount must be a positive value"),
                );
            }

            // Currency must be 3 uppercase letters
            if !is_valid_currency_code(&tx.interbank_settlement_amount.currency) {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_INVALID_CURRENCY",
                        format!("Invalid currency code: {}", tx.interbank_settlement_amount.currency),
                    )
                    .with_field_path(format!("{}.IntrBkSttlmAmt.Ccy", tx_path))
                    .with_suggestion("Currency must be a valid ISO 4217 3-letter code"),
                );
            }
        }

        result
    }
}

/// Validate camt.053 message.
impl Validator<camt053::Document> for MxSchemaValidator {
    fn validate(&self, message: &camt053::Document) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate Group Header
        let grp_hdr = &message.bank_to_customer_statement.group_header;

        // MsgId is mandatory
        if grp_hdr.message_id.trim().is_empty() {
            result.add_issue(
                ValidationIssue::error("MX_MISSING_MSG_ID", "Missing or empty MsgId")
                    .with_field_path("GrpHdr.MsgId")
                    .with_suggestion("MsgId must be a unique message identifier"),
            );
        }

        // Validate each statement
        for (i, stmt) in message.bank_to_customer_statement.statement.iter().enumerate() {
            let stmt_path = format!("Stmt[{}]", i);

            // Statement ID is mandatory
            if stmt.id.trim().is_empty() {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_STMT_ID", "Missing or empty statement Id")
                        .with_field_path(format!("{}.Id", stmt_path))
                        .with_suggestion("Statement Id is mandatory"),
                );
            }

            // Account identification is mandatory (either IBAN or Other)
            if stmt.account.id.iban.is_none() && stmt.account.id.other.is_none() {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_ACCOUNT", "Missing account identification")
                        .with_field_path(format!("{}.Acct.Id", stmt_path))
                        .with_suggestion("Either IBAN or Other account identification is required"),
                );
            }

            // At least one balance is required
            if stmt.balance.is_empty() {
                result.add_issue(
                    ValidationIssue::error(
                        "MX_MISSING_BALANCES",
                        "Statement must contain at least one balance",
                    )
                    .with_field_path(format!("{}.Bal", stmt_path))
                    .with_suggestion("At least opening or closing balance is required"),
                );
            }
        }

        result
    }
}

/// Validate camt.052 message.
impl Validator<camt052::Document> for MxSchemaValidator {
    fn validate(&self, message: &camt052::Document) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate Group Header
        let grp_hdr = &message.bank_to_customer_account_report.group_header;

        // MsgId is mandatory
        if grp_hdr.message_id.trim().is_empty() {
            result.add_issue(
                ValidationIssue::error("MX_MISSING_MSG_ID", "Missing or empty MsgId")
                    .with_field_path("GrpHdr.MsgId")
                    .with_suggestion("MsgId must be a unique message identifier"),
            );
        }

        // Validate each report
        for (i, rpt) in message.bank_to_customer_account_report.report.iter().enumerate() {
            let rpt_path = format!("Rpt[{}]", i);

            // Report ID is mandatory
            if rpt.id.trim().is_empty() {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_RPT_ID", "Missing or empty report Id")
                        .with_field_path(format!("{}.Id", rpt_path))
                        .with_suggestion("Report Id is mandatory"),
                );
            }

            // Account identification is mandatory (either IBAN or Other)
            if rpt.account.id.iban.is_none() && rpt.account.id.other.is_none() {
                result.add_issue(
                    ValidationIssue::error("MX_MISSING_ACCOUNT", "Missing account identification")
                        .with_field_path(format!("{}.Acct.Id", rpt_path))
                        .with_suggestion("Either IBAN or Other account identification is required"),
                );
            }
        }

        result
    }
}

/// Validate currency code format.
fn is_valid_currency_code(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_currency_code() {
        assert!(is_valid_currency_code("USD"));
        assert!(is_valid_currency_code("EUR"));
        assert!(is_valid_currency_code("GBP"));

        assert!(!is_valid_currency_code("usd")); // Lowercase
        assert!(!is_valid_currency_code("US")); // Too short
        assert!(!is_valid_currency_code("USDD")); // Too long
        assert!(!is_valid_currency_code("US1")); // Contains digit
    }
}
