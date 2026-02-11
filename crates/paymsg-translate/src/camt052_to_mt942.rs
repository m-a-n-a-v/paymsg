//! Translate camt.052 (ISO 20022) to MT942 (SWIFT)
//!
//! This module implements reverse translation from ISO 20022 camt.052.001.10
//! (BankToCustomerAccountReport) to SWIFT MT942 (Interim Transaction Report).
//!
//! Key transformations:
//! - Merge: TxsSummary → fields 90D/90C (debit and credit totals)
//! - Merge: Entry elements → field 61 (statement lines)
//! - Merge: LglSeqNb + ElctrncSeqNb → field 28C
//! - Merge: CreDtTm → field 13D (date/time indication)
//! - Merge: Account + Servicer BIC → field 25P (or just field 25)
//! - Field mapping: Rpt/Id → field 20
//! - Field mapping: Entry details → field 86
//! - NO balance fields (MT942 has no opening/closing balances)
//! - Floor limit from RptgSrc/Prtry or AddtlRptInf → field 34F

use crate::types::{
    AmountConverter, BicNormalizer, DateConverter, DataLossCategory, DataLossWarning,
    TranslationResult,
};
use chrono::{Datelike, NaiveDateTime, Timelike};
use paymsg_core::PaymsgError;
use paymsg_iso20022::camt052::Document;
use paymsg_mt::{ApplicationHeader, BasicHeader, Direction, MtField, MtMessage, TextBlock};

/// Translate a camt.052 message to MT942
pub fn translate(camt052: &Document) -> Result<TranslationResult<MtMessage>, PaymsgError> {
    let mut warnings = Vec::new();

    // Get the first report (MT942 is single report)
    let rpt = camt052
        .bank_to_customer_account_report
        .report
        .first()
        .ok_or_else(|| {
            PaymsgError::TranslationError("camt.052 must have at least one report".to_string())
        })?;

    // Build MT message blocks
    let block1 = build_block1(&mut warnings)?;
    let block2 = build_block2(&mut warnings)?;
    let block3 = None; // Optional, not typically populated from camt.052
    let block4 = build_block4(camt052, rpt, &mut warnings)?;
    let block5 = None; // Optional trailer

    let mt942 = MtMessage {
        block1,
        block2,
        block3,
        block4,
        block5,
    };

    Ok(TranslationResult::with_warnings(mt942, warnings))
}

/// Build Block 1: Basic Header
fn build_block1(warnings: &mut Vec<DataLossWarning>) -> Result<BasicHeader, PaymsgError> {
    // Default values for Block 1 (not present in camt.052)
    warnings.push(DataLossWarning::new(
        "Block1",
        DataLossCategory::NoEquivalent,
        "Block 1 fields defaulted (not present in camt.052)",
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
        "Block 2 fields defaulted (not present in camt.052)",
    ));

    Ok(ApplicationHeader {
        direction: Direction::Input,
        message_type: "942".to_string(),
        bic: "CUSTBEBBAXXX".to_string(), // Default, should be configured
        priority: "N".to_string(),
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Build Block 4: Text Block with report fields
fn build_block4(
    _camt052: &Document,
    rpt: &paymsg_iso20022::camt052::AccountReport,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<TextBlock, PaymsgError> {
    let mut fields = Vec::new();

    // Field 20: Report Reference
    let rpt_id = rpt.id.clone();
    let rpt_id_truncated = if rpt_id.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Field20",
                DataLossCategory::Truncation,
                format!("Report ID truncated from {} to 16 chars", rpt_id.len()),
            )
            .with_original_value(&rpt_id),
        );
        rpt_id[..16].to_string()
    } else {
        rpt_id
    };
    fields.push(MtField::new("20", rpt_id_truncated));

    // Field 25/25P: Account Identification
    let account_field = build_field_25(rpt, warnings)?;
    fields.push(account_field);

    // Field 28C: Report Number/Sequence Number
    let field_28c = build_field_28c(rpt, warnings)?;
    fields.push(field_28c);

    // Field 13D: Date/Time Indication
    let field_13d = build_field_13d(rpt, warnings)?;
    fields.push(field_13d);

    // Field 34F: Floor Limit Indicator (if present)
    if let Some(field_34f) = build_field_34f(rpt, warnings)? {
        fields.push(field_34f);
    }

    // Extract all entry fields (field 61 + field 86)
    let entry_fields = build_entry_fields(rpt, warnings)?;
    fields.extend(entry_fields);

    // Fields 90D/90C: Summary totals
    // Get currency from account for summary fields
    let currency = rpt.account.currency.as_ref().ok_or_else(|| {
        PaymsgError::TranslationError("Account currency is required for summary fields".to_string())
    })?;
    let summary_fields = build_summary_fields(rpt, currency, warnings)?;
    fields.extend(summary_fields);

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
    rpt: &paymsg_iso20022::camt052::AccountReport,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    let account = &rpt.account;

    // Get account number (IBAN or Other)
    let account_number = if let Some(ref iban) = account.id.iban {
        iban.clone()
    } else if let Some(ref other) = account.id.other {
        other.id.clone()
    } else {
        return Err(PaymsgError::TranslationError(
            "camt.052 account must have IBAN or Other identification".to_string(),
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

/// Build field 28C (Report Number/Sequence Number)
fn build_field_28c(
    rpt: &paymsg_iso20022::camt052::AccountReport,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    let legal_seq_nb = rpt
        .legal_sequence_number
        .map(|n| n.to_string())
        .unwrap_or_else(|| "1".to_string());

    let value = if let Some(ref electronic_seq_nb) = rpt.electronic_sequence_number {
        format!("{}/{}", legal_seq_nb, electronic_seq_nb)
    } else {
        legal_seq_nb
    };

    Ok(MtField::new("28C", value))
}

/// Build field 13D (Date/Time Indication)
fn build_field_13d(
    rpt: &paymsg_iso20022::camt052::AccountReport,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    // Convert ISO 8601 date-time to YYMMDD+HHMM format
    let date_time_str = &rpt.creation_date_time;

    // Parse ISO 8601 date-time
    let date_time = NaiveDateTime::parse_from_str(date_time_str, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(date_time_str, "%Y-%m-%dT%H:%M:%S%.fZ"))
        .or_else(|_| NaiveDateTime::parse_from_str(date_time_str, "%Y-%m-%dT%H:%M:%S%.3fZ"))
        .map_err(|_| {
            PaymsgError::ParseError(format!("Invalid date-time format: {}", date_time_str))
        })?;

    // Format as YYMMDD+HHMM
    let yy = date_time.date().year() % 100;
    let mm = date_time.date().month();
    let dd = date_time.date().day();
    let hh = date_time.time().hour();
    let min = date_time.time().minute();

    let value = format!("{:02}{:02}{:02}+{:02}{:02}", yy, mm, dd, hh, min);

    Ok(MtField::new("13D", value))
}

/// Build field 34F (Floor Limit Indicator) if present
fn build_field_34f(
    rpt: &paymsg_iso20022::camt052::AccountReport,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<MtField>, PaymsgError> {
    // Try to extract floor limit from RptgSrc/Prtry or AddtlRptInf
    if let Some(ref reporting_source) = rpt.reporting_source {
        if let Some(ref prtry) = reporting_source.proprietary {
            // Parse proprietary string: "Floor limit: EUR 10000.00"
            if let Some(floor_limit) = parse_floor_limit_from_text(prtry) {
                warnings.push(DataLossWarning::new(
                    "Field34F",
                    DataLossCategory::NoEquivalent,
                    "Reconstructed field 34F from RptgSrc/Prtry",
                ));
                return Ok(Some(MtField::new("34F", floor_limit)));
            }
        }
    }

    if let Some(ref addtl_info) = rpt.additional_report_info {
        if let Some(floor_limit) = parse_floor_limit_from_text(addtl_info) {
            warnings.push(DataLossWarning::new(
                "Field34F",
                DataLossCategory::NoEquivalent,
                "Reconstructed field 34F from AddtlRptInf",
            ));
            return Ok(Some(MtField::new("34F", floor_limit)));
        }
    }

    Ok(None)
}

/// Parse floor limit from text
/// Expected format: "Floor limit: EUR 10000.00" or "Floor limit EUR 10000.00"
fn parse_floor_limit_from_text(text: &str) -> Option<String> {
    // Look for pattern: currency code followed by amount
    let parts: Vec<&str> = text.split_whitespace().collect();

    for i in 0..parts.len().saturating_sub(1) {
        let part = parts[i];
        // Check if this looks like a currency code (3 uppercase letters)
        if part.len() == 3 && part.chars().all(|c| c.is_ascii_uppercase()) {
            // Next part should be the amount
            if let Some(amount_str) = parts.get(i + 1) {
                // Remove any non-numeric chars except decimal point
                let amount_normalized = amount_str.replace(',', "");

                // Convert period to comma for MT format
                let amount_mt = amount_normalized.replace('.', ",");

                return Some(format!("{}{}", part, amount_mt));
            }
        }
    }

    None
}

/// Build entry fields (field 61 + field 86 pairs)
fn build_entry_fields(
    rpt: &paymsg_iso20022::camt052::AccountReport,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<MtField>, PaymsgError> {
    let mut fields = Vec::new();

    if let Some(ref entries) = rpt.entry {
        for entry in entries {
            // Build field 61
            let field_61 = build_field_61(entry, warnings)?;
            fields.push(field_61);

            // Build field 86 (if entry has additional info)
            if let Some(field_86) = build_field_86(entry, warnings)? {
                fields.push(field_86);
            }
        }
    }

    Ok(fields)
}

/// Build field 61 (Statement Line)
fn build_field_61(
    entry: &paymsg_iso20022::camt053::Entry,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<MtField, PaymsgError> {
    // Extract value date
    let value_date_str = entry
        .value_date
        .as_ref()
        .and_then(|vd| vd.date.as_ref())
        .ok_or_else(|| PaymsgError::TranslationError("Entry value date is required".to_string()))?;
    let value_date_mt = DateConverter::mx_to_mt(value_date_str)?;

    // Extract booking date (optional, omit if same as value date)
    let booking_date_mt = entry
        .booking_date
        .as_ref()
        .and_then(|bd| bd.date.as_ref())
        .and_then(|bd_str| {
            if bd_str != value_date_str {
                // Extract MMDD from booking date
                let bd_mt = DateConverter::mx_to_mt(bd_str).ok()?;
                Some(bd_mt[2..6].to_string())
            } else {
                None
            }
        });

    // Extract D/C mark
    let dc_mark = match entry.credit_debit_indicator.as_str() {
        "CRDT" => {
            if entry.reversal_indicator.unwrap_or(false) {
                "RC"
            } else {
                "C"
            }
        }
        "DBIT" => {
            if entry.reversal_indicator.unwrap_or(false) {
                "RD"
            } else {
                "D"
            }
        }
        _ => {
            return Err(PaymsgError::TranslationError(format!(
                "Invalid credit/debit indicator: {}",
                entry.credit_debit_indicator
            )))
        }
    };

    // Extract funds code (from status)
    let funds_code = entry.status.code.as_ref().and_then(|code| {
        if code == "PDNG" {
            Some("D")
        } else {
            None
        }
    });

    // Extract amount
    let amount = &entry.amount;
    let amount_str = amount.value.to_string();
    let amount_mt = AmountConverter::mx_to_mt(&amount_str);

    // Extract transaction type
    let transaction_type = entry
        .bank_transaction_code
        .proprietary
        .as_ref()
        .map(|prtry| prtry.code.clone());

    // Extract customer reference (account servicer reference)
    let customer_ref = entry
        .entry_details
        .as_ref()
        .and_then(|ed| ed.first())
        .and_then(|ed| ed.transaction_details.as_ref())
        .and_then(|td| td.first())
        .and_then(|td| td.references.as_ref())
        .and_then(|refs| refs.account_servicer_reference.as_ref())
        .cloned();

    // Extract bank reference
    let bank_ref = entry
        .account_servicer_reference
        .as_ref()
        .map(|r| format!("//{}", r));

    // Build field value
    let mut value = String::new();
    value.push_str(&value_date_mt);

    if let Some(bd) = booking_date_mt {
        value.push_str(&bd);
    }

    value.push_str(dc_mark);

    if let Some(fc) = funds_code {
        value.push_str(fc);
    }

    value.push_str(&amount_mt);

    if let Some(tt) = transaction_type {
        // Truncate to 4 chars if needed
        if tt.len() > 4 {
            warnings.push(DataLossWarning::new(
                "Field61-TxType",
                DataLossCategory::Truncation,
                format!("Transaction type truncated from {} to 4 chars", tt.len()),
            ));
            value.push_str(&tt[..4]);
        } else {
            value.push_str(&tt);
        }
    }

    if let Some(cr) = customer_ref {
        // Truncate to 16 chars if needed
        if cr.len() > 16 {
            warnings.push(DataLossWarning::new(
                "Field61-CustRef",
                DataLossCategory::Truncation,
                format!("Customer reference truncated from {} to 16 chars", cr.len()),
            ));
            value.push_str(&cr[..16]);
        } else {
            value.push_str(&cr);
        }
    }

    if let Some(br) = bank_ref {
        value.push_str(&br);
    }

    Ok(MtField::new("61", value))
}

/// Build field 86 (Information to Account Owner)
fn build_field_86(
    entry: &paymsg_iso20022::camt053::Entry,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<MtField>, PaymsgError> {
    // Extract remittance information from entry details
    let remittance_info = entry
        .entry_details
        .as_ref()
        .and_then(|ed| ed.first())
        .and_then(|ed| ed.transaction_details.as_ref())
        .and_then(|td| td.first())
        .and_then(|td| td.remittance_information.as_ref())
        .and_then(|ri| ri.unstructured.as_ref())
        .and_then(|us| us.first())
        .cloned();

    if let Some(info) = remittance_info {
        // Truncate to 390 chars (6 lines x 65 chars) if needed
        if info.len() > 390 {
            warnings.push(DataLossWarning::new(
                "Field86",
                DataLossCategory::Truncation,
                format!("Field 86 truncated from {} to 390 chars", info.len()),
            ));
            Ok(Some(MtField::new("86", info[..390].to_string())))
        } else {
            Ok(Some(MtField::new("86", info)))
        }
    } else {
        // Try additional entry info
        if let Some(ref addtl_info) = entry.additional_entry_info {
            if addtl_info.len() > 390 {
                warnings.push(DataLossWarning::new(
                    "Field86",
                    DataLossCategory::Truncation,
                    format!("Field 86 truncated from {} to 390 chars", addtl_info.len()),
                ));
                Ok(Some(MtField::new("86", addtl_info[..390].to_string())))
            } else {
                Ok(Some(MtField::new("86", addtl_info.clone())))
            }
        } else {
            Ok(None)
        }
    }
}

/// Build summary fields (90D and 90C)
fn build_summary_fields(
    rpt: &paymsg_iso20022::camt052::AccountReport,
    currency: &str,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<MtField>, PaymsgError> {
    let mut fields = Vec::new();

    if let Some(ref summary) = rpt.transactions_summary {
        // Field 90D: Number and Sum of Debit Entries
        if let Some(ref total_debit) = summary.total_debit_entries {
            let field_90d = build_field_90d_or_90c(total_debit, currency)?;
            fields.push(MtField::new("90D", field_90d));
        }

        // Field 90C: Number and Sum of Credit Entries
        if let Some(ref total_credit) = summary.total_credit_entries {
            let field_90c = build_field_90d_or_90c(total_credit, currency)?;
            fields.push(MtField::new("90C", field_90c));
        }
    }

    Ok(fields)
}

/// Build field 90D or 90C value
/// Format: NumberOfEntries(5n) + Currency(3a) + TotalAmount(15d)
fn build_field_90d_or_90c(
    total: &paymsg_iso20022::camt053::NumberAndSumOfTransactions,
    currency: &str,
) -> Result<String, PaymsgError> {
    let num_entries = total
        .number_of_entries
        .ok_or_else(|| PaymsgError::TranslationError("Number of entries is required".to_string()))?;

    let sum_value = total
        .sum
        .ok_or_else(|| PaymsgError::TranslationError("Sum is required".to_string()))?;

    let amount_str = sum_value.to_string();
    let amount_mt = AmountConverter::mx_to_mt(&amount_str);

    Ok(format!("{}{}{}", num_entries, currency, amount_mt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_floor_limit_from_text() {
        let text1 = "Floor limit: EUR 10000.00 - transactions below this threshold";
        assert_eq!(
            parse_floor_limit_from_text(text1),
            Some("EUR10000,00".to_string())
        );

        let text2 = "Floor limit EUR 5000.50";
        assert_eq!(
            parse_floor_limit_from_text(text2),
            Some("EUR5000,50".to_string())
        );
    }
}
