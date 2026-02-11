//! Translate camt.053 (ISO 20022) to MT940 (SWIFT)
//!
//! This module implements reverse translation from ISO 20022 camt.053.001.10
//! (BankToCustomerStatement) to SWIFT MT940 (Customer Statement Message).
//!
//! Key transformations:
//! - Merge: Balance elements → fields 60F/60M/62F/62M/64/65 based on BalTp code
//! - Merge: Entry elements → field 61 (statement lines)
//! - Merge: LglSeqNb + ElctrncSeqNb → field 28C
//! - Merge: Account + Servicer BIC → field 25P (or just field 25)
//! - Field mapping: Stmt/Id → field 20
//! - Field mapping: Entry details → field 86
//! - Balance type code mapping: OPBD → 60F/60M, CLBD → 62F/62M, CLAV → 64, FWAV → 65
//! - Credit/debit indicator mapping: CRDT → C, DBIT → D (with reversal handling)

use crate::types::{
    AmountConverter, BicNormalizer, DateConverter, DataLossCategory, DataLossWarning,
    TranslationResult,
};
use paymsg_core::PaymsgError;
use paymsg_iso20022::camt053::Document;
use paymsg_mt::{ApplicationHeader, BasicHeader, Direction, MtField, MtMessage, TextBlock};

/// Translate a camt.053 message to MT940.
///
/// Converts an ISO 20022 camt.053 (Bank to Customer Statement) message
/// to a SWIFT MT940 (Customer Statement) message.
///
/// # Errors
///
/// Returns `PaymsgError::TranslationError` if:
/// - Required fields are missing in camt.053
/// - Field values cannot be converted
pub fn translate(camt053: &Document) -> Result<TranslationResult<MtMessage>, PaymsgError> {
    let mut warnings = Vec::new();

    // Get the first statement (MT940 is single statement)
    let stmt = camt053
        .bank_to_customer_statement
        .statement
        .first()
        .ok_or_else(|| {
            PaymsgError::TranslationError("camt.053 must have at least one statement".to_string())
        })?;

    // Build MT message blocks
    let block1 = build_block1(&mut warnings)?;
    let block2 = build_block2(&mut warnings)?;
    let block3 = None; // Optional, not typically populated from camt.053
    let block4 = build_block4(camt053, stmt, &mut warnings)?;
    let block5 = None; // Optional trailer

    let mt940 = MtMessage {
        block1,
        block2,
        block3,
        block4,
        block5,
    };

    Ok(TranslationResult::with_warnings(mt940, warnings))
}

/// Build Block 1: Basic Header
fn build_block1(warnings: &mut Vec<DataLossWarning>) -> Result<BasicHeader, PaymsgError> {
    // Default values for Block 1 (not present in camt.053)
    warnings.push(DataLossWarning::new(
        "Block1",
        DataLossCategory::NoEquivalent,
        "Block 1 fields defaulted (not present in camt.053)",
    ));

    Ok(BasicHeader {
        application_id: "F".to_string(),
        service_id: "01".to_string(),
        logical_terminal_address: "BANKBEBBAXXX".to_string(), // Default, should be configured
        session_number: "0000".to_string(),
        sequence_number: "000000".to_string(),
    })
}

/// Build Block 2: Application Header
fn build_block2(warnings: &mut Vec<DataLossWarning>) -> Result<ApplicationHeader, PaymsgError> {
    // Default values for Block 2
    warnings.push(DataLossWarning::new(
        "Block2",
        DataLossCategory::NoEquivalent,
        "Block 2 fields defaulted (not present in camt.053)",
    ));

    Ok(ApplicationHeader {
        direction: Direction::Input,
        message_type: "940".to_string(),
        bic: "CUSTBEBBAXXX".to_string(), // Default, should be configured
        priority: "N".to_string(),
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Build Block 4: Text Block with statement fields
fn build_block4(
    _camt053: &Document,
    stmt: &paymsg_iso20022::camt053::AccountStatement,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<TextBlock, PaymsgError> {
    let mut fields = Vec::new();

    // Field 20: Statement Reference
    let stmt_id = stmt.id.clone();
    let stmt_id_truncated = if stmt_id.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Field20",
                DataLossCategory::Truncation,
                format!("Statement ID truncated from {} to 16 chars", stmt_id.len()),
            )
            .with_original_value(&stmt_id),
        );
        stmt_id[..16].to_string()
    } else {
        stmt_id
    };
    fields.push(MtField::new("20", stmt_id_truncated));

    // Field 25/25P: Account Identification
    let account_field = build_field_25(stmt, warnings)?;
    fields.push(account_field);

    // Field 28C: Statement Number/Sequence Number
    let field_28c = build_field_28c(stmt, warnings)?;
    fields.push(field_28c);

    // Extract all balance fields
    let balance_fields = build_balance_fields(&stmt.balance, warnings)?;
    fields.extend(balance_fields);

    // Extract all entry fields (field 61 + field 86)
    let entry_fields = build_entry_fields(stmt, warnings)?;
    fields.extend(entry_fields);

    // Build text block content from fields
    let mut text_content = String::new();
    for field in &fields {
        text_content.push_str(&format!(":{}:{}\r\n", field.tag, field.value));
    }
    text_content.push('-');

    Ok(TextBlock {
        content: text_content,
    })
}

/// Build field 25 or 25P (Account Identification)
fn build_field_25(
    stmt: &paymsg_iso20022::camt053::AccountStatement,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    let account = &stmt.account;

    // Get account number (IBAN or Other)
    let account_number = if let Some(ref iban) = account.id.iban {
        iban.clone()
    } else if let Some(ref other) = account.id.other {
        other.id.clone()
    } else {
        return Err(PaymsgError::TranslationError(
            "camt.053 account must have IBAN or Other identification".to_string(),
        ));
    };

    // Check if servicer BIC is present
    if let Some(ref servicer) = account.servicer {
        if let Some(ref bic) = servicer.financial_institution_identification.bic {
            // Use field 25P format: BIC/Account
            let bic8 = BicNormalizer::to_bic8(bic);
            let value = format!("{}/{}", bic8, account_number);
            return Ok(MtField::new("25P", value));
        }
    }

    // No servicer BIC, use simple field 25
    Ok(MtField::new("25", account_number))
}

/// Build field 28C (Statement Number/Sequence Number)
fn build_field_28c(
    stmt: &paymsg_iso20022::camt053::AccountStatement,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    let legal_seq_nb = stmt
        .legal_sequence_number
        .map(|n| n.to_string())
        .unwrap_or_else(|| "1".to_string());

    let value = if let Some(ref electronic_seq_nb) = stmt.electronic_sequence_number {
        format!("{}/{}", legal_seq_nb, electronic_seq_nb)
    } else {
        legal_seq_nb
    };

    Ok(MtField::new("28C", value))
}

/// Build balance fields (60F/60M/62F/62M/64/65)
fn build_balance_fields(
    balances: &[paymsg_iso20022::camt053::Balance],
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<MtField>, PaymsgError> {
    let mut fields = Vec::new();

    for balance in balances {
        let bal_type_code = balance
            .balance_type
            .code_or_proprietary
            .code
            .as_ref()
            .ok_or_else(|| {
                PaymsgError::TranslationError("Balance type code is required".to_string())
            })?;

        // Determine field tag based on balance type
        let tag = match bal_type_code.as_str() {
            "OPBD" => {
                // Opening balance: check sub-type for FINAL vs INTERIM
                if let Some(ref sub_type) = balance.balance_type.sub_type {
                    if let Some(ref prtry) = sub_type.proprietary {
                        if prtry == "INTERIM" {
                            "60M"
                        } else {
                            "60F"
                        }
                    } else {
                        "60F" // Default to final
                    }
                } else {
                    "60F" // Default to final
                }
            }
            "CLBD" => {
                // Closing balance: check sub-type for FINAL vs INTERIM
                if let Some(ref sub_type) = balance.balance_type.sub_type {
                    if let Some(ref prtry) = sub_type.proprietary {
                        if prtry == "INTERIM" {
                            "62M"
                        } else {
                            "62F"
                        }
                    } else {
                        "62F" // Default to final
                    }
                } else {
                    "62F" // Default to final
                }
            }
            "CLAV" => "64", // Closing available balance
            "FWAV" => "65", // Forward available balance
            _ => {
                return Err(PaymsgError::TranslationError(format!(
                    "Unsupported balance type code: {}",
                    bal_type_code
                )));
            }
        };

        // Build balance field value: [D|C]YYMMDDCCCAMOUNT
        let dc_mark = match balance.credit_debit_indicator.as_str() {
            "CRDT" => "C",
            "DBIT" => "D",
            _ => {
                return Err(PaymsgError::TranslationError(format!(
                    "Invalid credit/debit indicator: {}",
                    balance.credit_debit_indicator
                )));
            }
        };

        // Extract date
        let date_str = balance
            .date
            .date
            .as_ref()
            .ok_or_else(|| PaymsgError::TranslationError("Balance date is required".to_string()))?;
        let date_mt = DateConverter::mx_to_mt(date_str)?;

        // Extract currency and amount
        let currency = &balance.amount.currency;
        let amount_str = balance.amount.value.to_string();
        let amount_mt = AmountConverter::mx_to_mt(&amount_str);

        let value = format!("{}{}{}{}", dc_mark, date_mt, currency, amount_mt);
        fields.push(MtField::new(tag, value));
    }

    Ok(fields)
}

/// Build entry fields (field 61 + field 86 pairs)
fn build_entry_fields(
    stmt: &paymsg_iso20022::camt053::AccountStatement,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<MtField>, PaymsgError> {
    let mut fields = Vec::new();

    if let Some(ref entries) = stmt.entry {
        for entry in entries {
            // Build field 61
            let field_61 = build_field_61(entry, warnings)?;
            fields.push(field_61);

            // Build field 86 if there's remittance info or additional info
            if let Some(field_86) = build_field_86(entry, warnings)? {
                fields.push(field_86);
            }
        }
    }

    Ok(fields)
}

/// Build field 61 (Statement Line) from Entry
fn build_field_61(
    entry: &paymsg_iso20022::camt053::Entry,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    // Field 61 format: YYMMDD[MMDD][D|C|RD|RC][FundsCode]AMOUNT[Type][CustomerRef][//BankRef][\nSupplementary]

    // Extract value date
    let value_date_str = entry
        .value_date
        .as_ref()
        .and_then(|vd| vd.date.as_ref())
        .ok_or_else(|| PaymsgError::TranslationError("Entry value date is required".to_string()))?;
    let value_date_mt = DateConverter::mx_to_mt(value_date_str)?;

    // Extract booking date (optional)
    let booking_date_mt = if let Some(ref booking_date) = entry.booking_date {
        if let Some(ref bd_str) = booking_date.date {
            let bd_mt = DateConverter::mx_to_mt(bd_str)?;
            // Extract MMDD from YYMMDD
            Some(bd_mt[2..6].to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Extract D/C mark with reversal indicator
    let dc_mark = match entry.credit_debit_indicator.as_str() {
        "CRDT" => {
            if entry.reversal_indicator == Some(true) {
                "RC"
            } else {
                "C"
            }
        }
        "DBIT" => {
            if entry.reversal_indicator == Some(true) {
                "RD"
            } else {
                "D"
            }
        }
        _ => {
            return Err(PaymsgError::TranslationError(format!(
                "Invalid credit/debit indicator: {}",
                entry.credit_debit_indicator
            )));
        }
    };

    // Extract amount
    let amount_str = entry.amount.value.to_string();
    let amount_mt = AmountConverter::mx_to_mt(&amount_str);

    // Extract transaction type (from BkTxCd/Prtry/Cd)
    let transaction_type = entry
        .bank_transaction_code
        .proprietary
        .as_ref()
        .map(|p| p.code.clone());

    // Extract customer reference (from NtryDtls/TxDtls/Refs/AcctSvcrRef)
    let customer_ref = entry
        .entry_details
        .as_ref()
        .and_then(|ed| ed.first())
        .and_then(|ed| ed.transaction_details.as_ref())
        .and_then(|td| td.first())
        .and_then(|td| td.references.as_ref())
        .and_then(|refs| refs.account_servicer_reference.as_ref());

    // Extract bank reference (from NtryRef)
    let bank_ref = entry.entry_reference.as_ref();

    // Build field 61 value
    let mut value = String::new();
    value.push_str(&value_date_mt);

    // Add booking date if present
    if let Some(ref bd) = booking_date_mt {
        value.push_str(bd);
    }

    value.push_str(dc_mark);
    value.push_str(&amount_mt);

    // Add transaction type if present
    if let Some(ref tt) = transaction_type {
        value.push_str(tt);
    }

    // Add customer reference if present
    if let Some(cust_ref) = customer_ref {
        value.push_str(cust_ref);
    }

    // Add bank reference if present
    if let Some(bank_ref_val) = bank_ref {
        value.push_str("//");
        value.push_str(bank_ref_val);
    }

    // Add supplementary details if present (from AddtlNtryInf)
    if let Some(ref addtl_info) = entry.additional_entry_info {
        value.push('\n');
        value.push_str(addtl_info);
    }

    Ok(MtField::new("61", value))
}

/// Build field 86 (Information to Account Owner) from Entry
fn build_field_86(
    entry: &paymsg_iso20022::camt053::Entry,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<MtField>, PaymsgError> {
    // Extract remittance info from entry details
    let rmt_info = entry
        .entry_details
        .as_ref()
        .and_then(|ed| ed.first())
        .and_then(|ed| ed.transaction_details.as_ref())
        .and_then(|td| td.first())
        .and_then(|td| td.remittance_information.as_ref());

    if let Some(rmt) = rmt_info {
        if let Some(ref unstrd) = rmt.unstructured {
            // Join all unstructured lines
            let text = unstrd.join("\n");

            // Truncate to 390 chars (6 lines × 65 chars)
            let text_truncated = if text.len() > 390 {
                text[..390].to_string()
            } else {
                text
            };

            return Ok(Some(MtField::new("86", text_truncated)));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bic_normalization() {
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFFXXX"), "DEUTDEFF");
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFF"), "DEUTDEFF");
    }

    #[test]
    fn test_date_conversion() {
        assert_eq!(DateConverter::mx_to_mt("2026-02-10").unwrap(), "260210");
    }

    #[test]
    fn test_amount_conversion() {
        assert_eq!(AmountConverter::mx_to_mt("1234.56"), "1234,56");
    }
}
