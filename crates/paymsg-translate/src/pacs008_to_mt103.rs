//! Translate pacs.008 (ISO 20022) to MT103 (SWIFT MT)
//!
//! This module implements the reverse translation from ISO 20022 pacs.008
//! (FI to FI Customer Credit Transfer) to SWIFT MT103 (Single Customer Credit Transfer).
//!
//! Key transformations:
//! - Merge: date + currency + amount → field 32A
//! - BIC truncation: MX BIC11 → MT BIC8 (remove XXX suffix)
//! - Charge bearer reverse lookup: SHAR→SHA, DEBT→OUR, CRED→BEN
//! - Name/address merge: structured party info → 4x35 MT field lines
//! - Data loss tracking: fields present in MX but not representable in MT

use crate::types::{
    AmountConverter, BicNormalizer, ChargeBearerConverter, DataLossCategory, DataLossWarning,
    DateConverter, TranslationResult,
};
use paymsg_core::PaymsgError;
use paymsg_iso20022::pacs008::Document as Pacs008Document;
use paymsg_mt::blocks::{ApplicationHeader, BasicHeader, Direction, TextBlock, UserHeader};
use paymsg_mt::MtMessage;
use std::collections::HashMap;

/// Translate a pacs.008 message to MT103
pub fn translate(pacs008: &Pacs008Document) -> Result<TranslationResult<MtMessage>, PaymsgError> {
    let mut warnings = Vec::new();

    // Extract the first (and typically only) transaction from pacs.008
    let credit_transfer = &pacs008.fi_to_fi_customer_credit_transfer;
    let grp_hdr = &credit_transfer.group_header;

    if credit_transfer.credit_transfer_transaction_information.is_empty() {
        return Err(PaymsgError::TranslationError(
            "pacs.008 has no credit transfer transactions".to_string(),
        ));
    }

    // MT103 is single transaction, pacs.008 can have multiple
    if credit_transfer.credit_transfer_transaction_information.len() > 1 {
        warnings.push(DataLossWarning::new(
            "CdtTrfTxInf",
            DataLossCategory::NoEquivalent,
            format!(
                "pacs.008 has {} transactions, MT103 only supports 1. Only first transaction will be translated.",
                credit_transfer.credit_transfer_transaction_information.len()
            ),
        ));
    }

    let tx_info = &credit_transfer.credit_transfer_transaction_information[0];

    // Build Block 1: Basic Header
    let block1 = build_block1(&grp_hdr.instructing_agent, &mut warnings)?;

    // Build Block 2: Application Header
    let block2 = build_block2(&grp_hdr.instructed_agent, &mut warnings)?;

    // Build Block 3: User Header (optional)
    let block3 = build_block3(tx_info, &mut warnings)?;

    // Build Block 4: Text Block
    let block4 = build_block4(tx_info, &mut warnings)?;

    // Block 5 is typically empty for translated messages
    let block5 = None;

    let mt_message = MtMessage {
        block1,
        block2,
        block3: if block3.tags.is_empty() {
            None
        } else {
            Some(block3)
        },
        block4,
        block5,
    };

    Ok(TranslationResult::with_warnings(mt_message, warnings))
}

/// Build Block 1: Basic Header
fn build_block1(
    instructing_agent: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<BasicHeader, PaymsgError> {
    // Extract BIC from instructing agent
    let bic = instructing_agent
        .financial_institution_id
        .bic
        .as_ref()
        .ok_or_else(|| {
            PaymsgError::TranslationError(
                "Instructing agent BICFI is required for MT103".to_string(),
            )
        })?;

    // Normalize BIC to BIC8 if it's BIC11 with XXX
    let bic8 = BicNormalizer::to_bic8(bic);

    // Pad to 12 characters for logical terminal address (BIC8 + branch/LT code)
    let logical_terminal_address = if bic8.len() == 8 {
        format!("{}AXXX", bic8)
    } else {
        format!("{:12}", bic8)
    };

    // Session and sequence numbers default to 0 for translated messages
    warnings.push(DataLossWarning::new(
        "Block1.SessionNumber",
        DataLossCategory::OptionalFieldOmitted,
        "Session and sequence numbers defaulted to 0 (not present in pacs.008)",
    ));

    Ok(BasicHeader {
        application_id: "F".to_string(),
        service_id: "01".to_string(),
        logical_terminal_address,
        session_number: "0000".to_string(),
        sequence_number: "000000".to_string(),
    })
}

/// Build Block 2: Application Header
fn build_block2(
    instructed_agent: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<ApplicationHeader, PaymsgError> {
    // Extract BIC from instructed agent
    let bic = instructed_agent
        .financial_institution_id
        .bic
        .as_ref()
        .ok_or_else(|| {
            PaymsgError::TranslationError(
                "Instructed agent BICFI is required for MT103".to_string(),
            )
        })?;

    // Normalize BIC to BIC8 if it's BIC11 with XXX, else keep as is
    let bic_normalized = BicNormalizer::to_bic8(bic);

    // Pad to 12 characters if needed
    let destination_bic = if bic_normalized.len() == 8 {
        format!("{}XXXX", bic_normalized)
    } else {
        format!("{:12}", bic_normalized)
    };

    // Priority defaults to Normal
    warnings.push(DataLossWarning::new(
        "Block2.Priority",
        DataLossCategory::OptionalFieldOmitted,
        "Message priority defaulted to Normal (not derivable from pacs.008)",
    ));

    Ok(ApplicationHeader {
        direction: Direction::Input,
        message_type: "103".to_string(),
        bic: destination_bic,
        priority: "N".to_string(),
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Build Block 3: User Header (optional)
fn build_block3(
    tx_info: &paymsg_iso20022::pacs008::CreditTransferTransactionInformation,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<UserHeader, PaymsgError> {
    let mut tags = HashMap::new();

    // Map UETR if present
    if let Some(uetr) = &tx_info.payment_id.uetr {
        tags.insert("121".to_string(), uetr.clone());
    }

    Ok(UserHeader { tags })
}

/// Build Block 4: Text Block
fn build_block4(
    tx_info: &paymsg_iso20022::pacs008::CreditTransferTransactionInformation,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<TextBlock, PaymsgError> {
    let mut fields = Vec::new();

    // Field :20: Transaction Reference (from InstrId or EndToEndId)
    let reference = tx_info
        .payment_id
        .instruction_id
        .as_ref()
        .unwrap_or(&tx_info.payment_id.end_to_end_id);

    // Truncate to 16 characters (MT103 field 20 limit)
    let reference_truncated = if reference.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Field20",
                DataLossCategory::Truncation,
                format!("Transaction reference truncated from {} to 16 characters", reference.len()),
            )
            .with_original_value(reference),
        );
        &reference[..16]
    } else {
        reference
    };
    fields.push(format!(":20:{}", reference_truncated));

    // Field :23B: Bank Operation Code (default to CRED)
    // This is typically derived from PmtTpInf/LclInstrm but not always present
    let bank_op_code = tx_info
        .payment_type_information
        .as_ref()
        .and_then(|pti| pti.local_instrument.as_ref())
        .and_then(|li| li.proprietary.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("CRED");
    fields.push(format!(":23B:{}", bank_op_code));

    // Field :32A: Value Date, Currency, Amount (merge transform)
    let settlement_date = tx_info
        .interbank_settlement_date
        .as_ref()
        .ok_or_else(|| {
            PaymsgError::TranslationError("IntrBkSttlmDt is required for field 32A".to_string())
        })?;
    let mt_date = DateConverter::mx_to_mt(settlement_date)?;

    let currency = &tx_info.interbank_settlement_amount.currency;
    let amount = &tx_info.interbank_settlement_amount.value;
    let mt_amount = AmountConverter::mx_to_mt(&amount.to_string());

    fields.push(format!(":32A:{}{}{}", mt_date, currency, mt_amount));

    // Field :50K: Ordering Customer (debtor)
    let debtor_field = build_party_field_50k(&tx_info.debtor, warnings)?;
    fields.push(debtor_field);

    // Field :59: Beneficiary Customer (creditor) - no account for now
    let creditor_field = build_party_field_59(&tx_info.creditor, warnings)?;
    fields.push(creditor_field);

    // Field :71A: Details of Charges (charge bearer)
    let charge_bearer_mt = ChargeBearerConverter::mx_to_mt(&tx_info.charge_bearer)?;
    fields.push(format!(":71A:{}", charge_bearer_mt));

    // Join all fields with CRLF
    let content = fields.join("\r\n");

    Ok(TextBlock { content })
}

/// Build field 50K: Ordering Customer
fn build_party_field_50k(
    party: &paymsg_iso20022::pacs008::PartyIdentification,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<String, PaymsgError> {
    // Format: :50K:name_line1
    // name_line2
    // address_line1
    // address_line2
    // Each line max 35 chars, max 4 lines total

    let mut lines = Vec::new();

    // Add name (up to 2 lines, 35 chars each)
    if let Some(name) = &party.name {
        let name_lines = split_into_mt_lines(name, 35, 2);
        lines.extend(name_lines);
    }

    // Add address lines (up to 2 more lines for total of 4)
    if let Some(postal_addr) = &party.postal_address {
        if let Some(addr_lines) = &postal_addr.address_line {
            for addr_line in addr_lines.iter().take(2) {
                let split_lines = split_into_mt_lines(addr_line, 35, 1);
                lines.extend(split_lines);
                if lines.len() >= 4 {
                    break;
                }
            }
        }
    }

    // Warn if we had to truncate
    if let Some(postal_addr) = &party.postal_address {
        if let Some(addr_lines) = &postal_addr.address_line {
            if addr_lines.len() > 2 {
                warnings.push(DataLossWarning::new(
                    "Field50K.Address",
                    DataLossCategory::Truncation,
                    format!(
                        "Address has {} lines, MT103 field 50K supports max 4 total lines (2 for name, 2 for address)",
                        addr_lines.len()
                    ),
                ));
            }
        }
    }

    // Ensure we have at least 1 line
    if lines.is_empty() {
        lines.push("UNKNOWN".to_string());
    }

    // Format field: first line after tag, subsequent lines without tag
    let field = if lines.len() == 1 {
        format!(":50K:{}", lines[0])
    } else {
        let first = &lines[0];
        let rest = lines[1..].join("\r\n");
        format!(":50K:{}\r\n{}", first, rest)
    };

    Ok(field)
}

/// Build field 59: Beneficiary Customer
fn build_party_field_59(
    party: &paymsg_iso20022::pacs008::PartyIdentification,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<String, PaymsgError> {
    // Format: :59:name_line1
    // name_line2
    // address_line1
    // address_line2
    // Max 4 lines, 35 chars each

    let mut lines = Vec::new();

    // Add name
    if let Some(name) = &party.name {
        let name_lines = split_into_mt_lines(name, 35, 2);
        lines.extend(name_lines);
    }

    // Add address lines
    if let Some(postal_addr) = &party.postal_address {
        if let Some(addr_lines) = &postal_addr.address_line {
            for addr_line in addr_lines.iter().take(2) {
                let split_lines = split_into_mt_lines(addr_line, 35, 1);
                lines.extend(split_lines);
                if lines.len() >= 4 {
                    break;
                }
            }
        }
    }

    // Warn if we had to truncate
    if let Some(postal_addr) = &party.postal_address {
        if let Some(addr_lines) = &postal_addr.address_line {
            if addr_lines.len() > 2 {
                warnings.push(DataLossWarning::new(
                    "Field59.Address",
                    DataLossCategory::Truncation,
                    format!(
                        "Address has {} lines, MT103 field 59 supports max 4 total lines",
                        addr_lines.len()
                    ),
                ));
            }
        }
    }

    // Ensure we have at least 1 line
    if lines.is_empty() {
        lines.push("UNKNOWN".to_string());
    }

    // Format field
    let field = if lines.len() == 1 {
        format!(":59:{}", lines[0])
    } else {
        let first = &lines[0];
        let rest = lines[1..].join("\r\n");
        format!(":59:{}\r\n{}", first, rest)
    };

    Ok(field)
}

/// Split a string into MT field lines with max length
///
/// Splits on word boundaries where possible to avoid breaking words
fn split_into_mt_lines(text: &str, max_len: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            // First word in line
            if word.len() > max_len {
                // Word too long, truncate it
                current_line = word[..max_len].to_string();
            } else {
                current_line = word.to_string();
            }
        } else if current_line.len() + 1 + word.len() <= max_len {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Start new line
            lines.push(current_line.clone());
            if lines.len() >= max_lines {
                return lines;
            }
            if word.len() > max_len {
                current_line = word[..max_len].to_string();
            } else {
                current_line = word.to_string();
            }
        }
    }

    // Add last line if not empty
    if !current_line.is_empty() && lines.len() < max_lines {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_mt_lines_short() {
        let result = split_into_mt_lines("ALICE MARTIN", 35, 2);
        assert_eq!(result, vec!["ALICE MARTIN"]);
    }

    #[test]
    fn test_split_into_mt_lines_multiple() {
        let result = split_into_mt_lines(
            "THIS IS A VERY LONG NAME THAT SHOULD BE SPLIT INTO MULTIPLE LINES FOR TESTING",
            35,
            3,
        );
        assert!(result.len() >= 2 && result.len() <= 3);
        for line in &result {
            assert!(line.len() <= 35);
        }
    }

    #[test]
    fn test_split_into_mt_lines_max_lines() {
        let result = split_into_mt_lines(
            "WORD1 WORD2 WORD3 WORD4 WORD5 WORD6 WORD7 WORD8 WORD9 WORD10",
            10,
            2,
        );
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_split_into_mt_lines_long_word() {
        let result = split_into_mt_lines("VERYLONGWORDTHATEXCEEDSTHEMAXIMUMLENGTH", 20, 2);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 20);
    }
}
