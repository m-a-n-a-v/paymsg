//! Translate MT202 (SWIFT) to pacs.009 (ISO 20022)
//!
//! This module implements translation from SWIFT MT202 (General Financial Institution Transfer)
//! to ISO 20022 pacs.009.001.10 (FinancialInstitutionCreditTransfer).
//!
//! Key transformations:
//! - Split: field 32A → date + currency + amount
//! - BIC normalization: MT BIC8 → MX BIC11 (append XXX)
//! - Field mapping: 52A (Ordering Institution) → InstgAgt
//! - Field mapping: 58A (Beneficiary Institution) → Cdtr
//! - Field mapping: 57A (Account With Institution) → CdtrAgt
//! - Field mapping: 56A (Intermediary) → IntrmyAgt1
//! - Field mapping: 20 (Transaction Reference) → InstrId
//! - Field mapping: 21 (Related Reference) → EndToEndId or InstrForNxtAgt

use crate::types::{
    AmountConverter, BicNormalizer, DateConverter, DataLossCategory, DataLossWarning,
    TranslationResult,
};
use chrono::Utc;
use paymsg_core::PaymsgError;
use paymsg_iso20022::pacs008::{
    ActiveCurrencyAndAmount, BranchAndFinancialInstitutionIdentification,
    FinancialInstitutionIdentification, InstructionForNextAgent, PaymentIdentification,
};
use paymsg_iso20022::pacs009::{CreditTransferTransactionInformation, Document, FICdtTrf, GroupHeader};
use paymsg_mt::MtMessage;
use std::collections::HashMap;
use uuid::Uuid;

/// Translate an MT202 message to pacs.009.
///
/// Converts a SWIFT MT202 (General Financial Institution Transfer) message
/// to an ISO 20022 pacs.009 (Financial Institution Credit Transfer) message.
///
/// # Errors
///
/// Returns `PaymsgError::TranslationError` if:
/// - Required fields are missing in MT202
/// - Field values cannot be parsed or converted
pub fn translate(mt202: &MtMessage) -> Result<TranslationResult<Document>, PaymsgError> {
    let mut warnings = Vec::new();

    // Build Group Header from message envelope
    let grp_hdr = build_group_header(mt202, &mut warnings)?;

    // Build Credit Transfer Transaction Information from Block 4 fields
    let tx_info = build_transaction_info(mt202, &mut warnings)?;

    let fi_credit_transfer = FICdtTrf {
        group_header: grp_hdr,
        credit_transfer_transaction_information: vec![tx_info],
    };

    let document = Document {
        fi_credit_transfer,
    };

    Ok(TranslationResult::with_warnings(document, warnings))
}

/// Build Group Header from MT202 message envelope (Blocks 1-3)
fn build_group_header(
    mt202: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<GroupHeader, PaymsgError> {
    // Generate message ID (not present in MT202)
    let message_id = format!("MT202-{}", Uuid::new_v4());
    warnings.push(DataLossWarning::new(
        "GrpHdr/MsgId",
        DataLossCategory::NoEquivalent,
        "Generated message ID (not present in MT202)",
    ));

    // Creation date/time - use current time or derive from field 13C if present
    let creation_date_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    warnings.push(DataLossWarning::new(
        "GrpHdr/CreDtTm",
        DataLossCategory::NoEquivalent,
        "Set to current timestamp (MT202 has no message creation timestamp)",
    ));

    // Number of transactions = 1 (MT202 is single transaction)
    let number_of_transactions = "1".to_string();

    // Extract instructing agent from Block 1 logical terminal address
    let instructing_agent = extract_instructing_agent(&mt202.block1, warnings)?;

    // Extract instructed agent from Block 2 destination
    let instructed_agent = extract_instructed_agent(&mt202.block2, warnings)?;

    Ok(GroupHeader {
        message_id,
        creation_date_time,
        number_of_transactions,
        total_interbank_settlement_amount: None, // Optional, can be derived from field 32A
        interbank_settlement_date: None,         // Handled at transaction level
        settlement_information: None,
        instructing_agent,
        instructed_agent,
    })
}

/// Extract instructing agent from Block 1
fn extract_instructing_agent(
    block1: &paymsg_mt::BasicHeader,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Extract BIC from logical terminal address (positions 0-7 or 0-10)
    let lt_addr = &block1.logical_terminal_address;
    let bic = if lt_addr.len() >= 8 {
        &lt_addr[0..8]
    } else {
        return Err(PaymsgError::TranslationError(
            "Invalid logical terminal address in Block 1".to_string(),
        ));
    };

    // Normalize to BIC11
    let bic11 = BicNormalizer::to_bic11(bic);

    Ok(BranchAndFinancialInstitutionIdentification {
        financial_institution_id: FinancialInstitutionIdentification {
            bic: Some(bic11),
            name: None,
            postal_address: None,
        },
    })
}

/// Extract instructed agent from Block 2
fn extract_instructed_agent(
    block2: &paymsg_mt::ApplicationHeader,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Extract BIC from block2.bic field
    let bic_raw = &block2.bic;

    // BIC may be padded (12 chars), extract first 8 or 11
    let bic = if bic_raw.len() >= 11 && !bic_raw[8..11].chars().all(|c: char| c == 'X' || c.is_whitespace()) {
        &bic_raw[0..11]
    } else if bic_raw.len() >= 8 {
        &bic_raw[0..8]
    } else {
        return Err(PaymsgError::TranslationError(
            "Invalid BIC in Block 2".to_string(),
        ));
    };

    // Normalize to BIC11
    let bic11 = BicNormalizer::to_bic11(bic);

    Ok(BranchAndFinancialInstitutionIdentification {
        financial_institution_id: FinancialInstitutionIdentification {
            bic: Some(bic11),
            name: None,
            postal_address: None,
        },
    })
}

/// Build transaction information from Block 4 fields
fn build_transaction_info(
    mt202: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<CreditTransferTransactionInformation, PaymsgError> {
    // Parse fields from Block 4
    let parsed_fields = mt202.block4.parse_fields()?;
    let fields: HashMap<String, String> = parsed_fields
        .iter()
        .map(|f| (f.tag.clone(), f.value.clone()))
        .collect();

    // Field 20: Transaction Reference → InstrId
    let transaction_ref = fields
        .get("20")
        .ok_or_else(|| PaymsgError::TranslationError("MT202 field 20 is mandatory".to_string()))?;

    // Truncate to 35 chars if needed (MT allows 16, MX allows 35)
    let instr_id = transaction_ref.to_string();

    // Field 21: Related Reference → EndToEndId (or use UETR if present)
    let related_ref = fields
        .get("21")
        .ok_or_else(|| PaymsgError::TranslationError("MT202 field 21 is mandatory".to_string()))?;

    // Check for UETR in Block 3
    let uetr = mt202
        .block3
        .as_ref()
        .and_then(|b3| b3.tags.get("121"))
        .cloned();

    // EndToEndId: priority is UETR > field 21 > field 20
    let end_to_end_id = if let Some(ref uetr_val) = uetr {
        uetr_val.clone()
    } else {
        related_ref.clone()
    };

    let payment_id = PaymentIdentification {
        instruction_id: Some(instr_id),
        end_to_end_id,
        transaction_id: None,
        uetr,
    };

    // Field 32A: Value Date, Currency, Amount → split into date and amount
    let field_32a = fields
        .get("32A")
        .ok_or_else(|| PaymsgError::TranslationError("MT202 field 32A is mandatory".to_string()))?;

    let (settlement_date, currency, amount_str) = parse_field_32a(field_32a)?;

    // Convert date from YYMMDD to YYYY-MM-DD
    let settlement_date_mx = DateConverter::mt_to_mx(&settlement_date)?;

    // Convert amount from comma decimal to period decimal
    let amount_mx_str = AmountConverter::mt_to_mx(&amount_str);

    // Parse to Decimal
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let amount_decimal = Decimal::from_str(&amount_mx_str).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount format: {} ({})", amount_mx_str, e))
    })?;

    let interbank_settlement_amount = ActiveCurrencyAndAmount {
        currency: currency.clone(),
        value: amount_decimal,
    };

    // Field 52A/52D: Ordering Institution → InstgAgt
    let instructing_agent = extract_field_52(&fields, warnings)?;

    // Field 58A/58D: Beneficiary Institution → Cdtr
    let creditor = extract_field_58(&fields, warnings)?;

    // Field 57A/57D: Account With Institution → CdtrAgt
    let creditor_agent = extract_field_57(&fields, warnings)?;

    // Field 56A/56D: Intermediary Institution → IntrmyAgt1
    let intermediary_agent_1 = extract_field_56(&fields, warnings)?;

    // InstdAgt: use same as GrpHdr (from Block 2)
    let instructed_agent = extract_instructed_agent(&mt202.block2, warnings)?;

    // Field 72: Sender to Receiver Information → InstrForNxtAgt
    let instruction_for_next_agent = extract_field_72(&fields, warnings)?;

    // ChrgBr: default to SHAR (MT202 has no charge bearer field)
    let charge_bearer = Some("SHAR".to_string());
    warnings.push(DataLossWarning::new(
        "CdtTrfTxInf/ChrgBr",
        DataLossCategory::NoEquivalent,
        "Defaulted to SHAR (shared charges). MT202 has no charge bearer field.",
    ));

    Ok(CreditTransferTransactionInformation {
        payment_id,
        payment_type_information: None,
        interbank_settlement_amount,
        interbank_settlement_date: Some(settlement_date_mx),
        settlement_time_indication: None,
        settlement_time_request: None,
        instructed_amount: None,
        exchange_rate: None,
        charge_bearer,
        charges_information: None,
        instructing_agent,
        instructed_agent,
        intermediary_agent_1,
        intermediary_agent_2: None,
        intermediary_agent_3: None,
        creditor,
        creditor_account: None, // Could be derived from field 58 party identifier
        creditor_agent,
        instruction_for_next_agent,
        purpose: None,
        regulatory_reporting: None,
        remittance_information: None,
    })
}

/// Parse field 32A: YYMMDDCCCAMOUNT
/// Returns (date, currency, amount)
fn parse_field_32a(value: &str) -> Result<(String, String, String), PaymsgError> {
    if value.len() < 9 {
        return Err(PaymsgError::ParseError(format!(
            "Field 32A too short: {}",
            value
        )));
    }

    let date = value[0..6].to_string(); // YYMMDD
    let currency = value[6..9].to_string(); // CCC
    let amount = value[9..].to_string(); // amount with comma

    Ok((date, currency, amount))
}

/// Extract field 52A or 52D (Ordering Institution)
fn extract_field_52(
    fields: &HashMap<String, String>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Try 52A first (BIC)
    if let Some(value) = fields.get("52A") {
        return parse_party_bic_field(value, "52A");
    }

    // Try 52D (Name & Address)
    if let Some(value) = fields.get("52D") {
        return parse_party_name_address_field(value, "52D", warnings);
    }

    Err(PaymsgError::TranslationError(
        "MT202 field 52A or 52D is mandatory (Ordering Institution)".to_string(),
    ))
}

/// Extract field 57A/57B/57D (Account With Institution)
fn extract_field_57(
    fields: &HashMap<String, String>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Try 57A first (BIC)
    if let Some(value) = fields.get("57A") {
        return parse_party_bic_field(value, "57A");
    }

    // Try 57B (Location)
    if let Some(value) = fields.get("57B") {
        return parse_party_name_address_field(value, "57B", warnings);
    }

    // Try 57D (Name & Address)
    if let Some(value) = fields.get("57D") {
        return parse_party_name_address_field(value, "57D", warnings);
    }

    Err(PaymsgError::TranslationError(
        "MT202 field 57A/57B/57D is mandatory (Account With Institution)".to_string(),
    ))
}

/// Extract field 58A or 58D (Beneficiary Institution)
fn extract_field_58(
    fields: &HashMap<String, String>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Try 58A first (BIC)
    if let Some(value) = fields.get("58A") {
        return parse_party_bic_field(value, "58A");
    }

    // Try 58D (Name & Address)
    if let Some(value) = fields.get("58D") {
        return parse_party_name_address_field(value, "58D", warnings);
    }

    Err(PaymsgError::TranslationError(
        "MT202 field 58A or 58D is mandatory (Beneficiary Institution)".to_string(),
    ))
}

/// Extract field 56A or 56D (Intermediary Institution) - optional
fn extract_field_56(
    fields: &HashMap<String, String>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<BranchAndFinancialInstitutionIdentification>, PaymsgError> {
    // Try 56A first (BIC)
    if let Some(value) = fields.get("56A") {
        return Ok(Some(parse_party_bic_field(value, "56A")?));
    }

    // Try 56D (Name & Address)
    if let Some(value) = fields.get("56D") {
        return Ok(Some(parse_party_name_address_field(value, "56D", warnings)?));
    }

    // Field 56 is optional
    Ok(None)
}

/// Parse party field with BIC (option A)
/// Format: optional party identifier line (starts with /), then BIC
fn parse_party_bic_field(
    value: &str,
    field_tag: &str,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    if value.is_empty() {
        return Err(PaymsgError::TranslationError(format!(
            "Field {} is empty",
            field_tag
        )));
    }

    // Split by newlines
    let lines: Vec<&str> = value.lines().collect();

    // Find BIC line (may be first line or after party identifier)
    let bic_line = if !lines.is_empty() && lines[0].starts_with('/') {
        // Party identifier present, BIC is on second line
        lines.get(1).ok_or_else(|| {
            PaymsgError::TranslationError(format!("Field {} missing BIC after party identifier", field_tag))
        })?
    } else {
        // BIC is first line
        lines.first().ok_or_else(|| {
            PaymsgError::TranslationError(format!("Field {} has no BIC", field_tag))
        })?
    };

    let bic = bic_line.trim();
    let bic11 = BicNormalizer::to_bic11(bic);

    Ok(BranchAndFinancialInstitutionIdentification {
        financial_institution_id: FinancialInstitutionIdentification {
            bic: Some(bic11),
            name: None,
            postal_address: None,
        },
    })
}

/// Parse party field with Name & Address (option D or B)
/// Format: optional party identifier line (starts with /), then name, then address lines
fn parse_party_name_address_field(
    value: &str,
    field_tag: &str,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<BranchAndFinancialInstitutionIdentification, PaymsgError> {
    if value.is_empty() {
        return Err(PaymsgError::TranslationError(format!(
            "Field {} is empty",
            field_tag
        )));
    }

    // Split by newlines
    let all_lines: Vec<&str> = value.lines().collect();

    // Skip party identifier if present
    let content_lines: Vec<&str> = all_lines
        .iter()
        .skip_while(|line| line.starts_with('/'))
        .copied()
        .collect();

    if content_lines.is_empty() {
        return Err(PaymsgError::TranslationError(format!(
            "Field {} has no name/address after party identifier",
            field_tag
        )));
    }

    // First line is name, remaining lines are address
    let name = content_lines[0].to_string();
    let address_lines: Vec<String> = content_lines[1..].iter().map(|s| s.to_string()).collect();

    let postal_address = if !address_lines.is_empty() {
        Some(paymsg_iso20022::pacs008::PostalAddress {
            street_name: None,
            building_number: None,
            post_code: None,
            town_name: None,
            country_subdivision: None,
            country: None,
            address_line: Some(address_lines),
        })
    } else {
        None
    };

    Ok(BranchAndFinancialInstitutionIdentification {
        financial_institution_id: FinancialInstitutionIdentification {
            bic: None,
            name: Some(name),
            postal_address,
        },
    })
}

/// Extract field 72 (Sender to Receiver Information) - optional
fn extract_field_72(
    fields: &HashMap<String, String>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<Vec<InstructionForNextAgent>>, PaymsgError> {
    if let Some(value) = fields.get("72") {
        // Truncate to 140 chars (MX limit)
        let truncated = if value.len() > 140 {
            warnings.push(
                DataLossWarning::new(
                    "CdtTrfTxInf/InstrForNxtAgt/InstrInf",
                    DataLossCategory::Truncation,
                    format!(
                        "Field 72 truncated from {} to 140 chars",
                        value.len()
                    ),
                )
                .with_original_value(value),
            );
            value[..140].to_string()
        } else {
            value.clone()
        };

        let instruction = InstructionForNextAgent {
            code: None,
            instruction_information: Some(truncated),
        };

        Ok(Some(vec![instruction]))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_32a() {
        let result = parse_field_32a("260210USD5000000,00").unwrap();
        assert_eq!(result.0, "260210");
        assert_eq!(result.1, "USD");
        assert_eq!(result.2, "5000000,00");
    }

    #[test]
    fn test_parse_field_32a_invalid() {
        let result = parse_field_32a("260210US");
        assert!(result.is_err());
    }
}
