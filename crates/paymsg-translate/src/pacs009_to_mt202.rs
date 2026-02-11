//! Translate pacs.009 (ISO 20022) to MT202 (SWIFT MT)
//!
//! This module implements the reverse translation from ISO 20022 pacs.009.001.10
//! (FinancialInstitutionCreditTransfer) to SWIFT MT202 (General Financial Institution Transfer).
//!
//! Key transformations:
//! - Merge: date + currency + amount → field 32A
//! - BIC truncation: MX BIC11 → MT BIC8 (strip XXX suffix)
//! - Field mapping: InstgAgt → 52A (Ordering Institution)
//! - Field mapping: Cdtr → 58A (Beneficiary Institution)
//! - Field mapping: CdtrAgt → 57A (Account With Institution)
//! - Field mapping: IntrmyAgt1 → 56A (Intermediary)
//! - Data loss tracking: fields present in MX but not representable in MT

use crate::types::{
    AmountConverter, BicNormalizer, DateConverter, DataLossCategory, DataLossWarning,
    TranslationResult,
};
use paymsg_core::PaymsgError;
use paymsg_iso20022::pacs009::Document as Pacs009Document;
use paymsg_mt::blocks::{ApplicationHeader, BasicHeader, Direction, TextBlock, UserHeader};
use paymsg_mt::fields::MtField;
use paymsg_mt::MtMessage;
use std::collections::HashMap;

/// Translate a pacs.009 message to MT202
pub fn translate(pacs009: &Pacs009Document) -> Result<TranslationResult<MtMessage>, PaymsgError> {
    let mut warnings = Vec::new();

    // Extract the first (and typically only) transaction from pacs.009
    let fi_credit_transfer = &pacs009.fi_credit_transfer;
    let grp_hdr = &fi_credit_transfer.group_header;

    if fi_credit_transfer.credit_transfer_transaction_information.is_empty() {
        return Err(PaymsgError::TranslationError(
            "pacs.009 has no credit transfer transactions".to_string(),
        ));
    }

    // MT202 is single transaction, pacs.009 can have multiple
    if fi_credit_transfer.credit_transfer_transaction_information.len() > 1 {
        warnings.push(DataLossWarning::new(
            "CdtTrfTxInf",
            DataLossCategory::NoEquivalent,
            format!(
                "pacs.009 has {} transactions, MT202 only supports 1. Only first transaction will be translated.",
                fi_credit_transfer.credit_transfer_transaction_information.len()
            ),
        ));
    }

    let tx_info = &fi_credit_transfer.credit_transfer_transaction_information[0];

    // Build Block 1: Basic Header
    let block1 = build_block1(&grp_hdr.instructing_agent, &mut warnings)?;

    // Build Block 2: Application Header
    let block2 = build_block2(&grp_hdr.instructed_agent, &mut warnings)?;

    // Build Block 3: User Header (optional, for UETR)
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
                "Instructing agent BICFI is required for MT202".to_string(),
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
        "Session and sequence numbers defaulted to 0 (not present in pacs.009)",
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
                "Instructed agent BICFI is required for MT202".to_string(),
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
        "Priority defaulted to Normal (not present in pacs.009)",
    ));

    Ok(ApplicationHeader {
        direction: Direction::Input,
        message_type: "202".to_string(),
        bic: destination_bic,
        priority: "N".to_string(),
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Build Block 3: User Header (for UETR if present)
fn build_block3(
    tx_info: &paymsg_iso20022::pacs009::CreditTransferTransactionInformation,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<UserHeader, PaymsgError> {
    let mut tags = HashMap::new();

    // Add UETR if present
    if let Some(ref uetr) = tx_info.payment_id.uetr {
        tags.insert("121".to_string(), uetr.clone());
    }

    // Could add other Block 3 tags if needed (e.g., 108:MUR for message user reference)

    Ok(UserHeader { tags })
}

/// Build Block 4: Text Block with MT202 fields
fn build_block4(
    tx_info: &paymsg_iso20022::pacs009::CreditTransferTransactionInformation,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<TextBlock, PaymsgError> {
    let mut fields = Vec::new();

    // Field 20: Transaction Reference Number (from InstrId)
    let transaction_ref = tx_info
        .payment_id
        .instruction_id
        .as_ref()
        .ok_or_else(|| {
            PaymsgError::TranslationError("pacs.009 InstrId is required for MT202".to_string())
        })?;

    // Truncate to 16 chars (MT202 limit)
    let truncated_ref = if transaction_ref.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Field20",
                DataLossCategory::Truncation,
                format!(
                    "Transaction reference truncated from {} to 16 chars",
                    transaction_ref.len()
                ),
            )
            .with_original_value(transaction_ref),
        );
        &transaction_ref[..16]
    } else {
        transaction_ref
    };

    fields.push(MtField::new("20", truncated_ref));

    // Field 21: Related Reference (from EndToEndId, or InstrId if UETR was used for EndToEndId)
    let related_ref = &tx_info.payment_id.end_to_end_id;
    let truncated_related_ref = if related_ref.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Field21",
                DataLossCategory::Truncation,
                format!(
                    "Related reference truncated from {} to 16 chars",
                    related_ref.len()
                ),
            )
            .with_original_value(related_ref),
        );
        &related_ref[..16]
    } else {
        related_ref
    };

    fields.push(MtField::new("21", truncated_related_ref));

    // Field 32A: Value Date, Currency Code, Amount
    let settlement_date = tx_info
        .interbank_settlement_date
        .as_ref()
        .ok_or_else(|| {
            PaymsgError::TranslationError(
                "pacs.009 IntrBkSttlmDt is required for MT202".to_string(),
            )
        })?;

    // Convert date from YYYY-MM-DD to YYMMDD
    let date_mt = DateConverter::mx_to_mt(settlement_date)?;

    // Get currency and amount
    let currency = &tx_info.interbank_settlement_amount.currency;
    let amount_decimal = &tx_info.interbank_settlement_amount.value;

    // Convert Decimal to string and then to MT format (comma decimal)
    let amount_str = amount_decimal.to_string();
    let amount_mt = AmountConverter::mx_to_mt(&amount_str);

    // Format: YYMMDDCCCAMOUNT
    let field_32a_value = format!("{}{}{}", date_mt, currency, amount_mt);

    fields.push(MtField::new("32A", field_32a_value));

    // Field 52A or 52D: Ordering Institution (from InstgAgt)
    add_field_52(&mut fields, &tx_info.instructing_agent, warnings)?;

    // Field 56A or 56D: Intermediary Institution (from IntrmyAgt1, optional)
    if let Some(ref intermediary) = tx_info.intermediary_agent_1 {
        add_field_56(&mut fields, intermediary, warnings)?;
    }

    // Field 57A or 57D: Account With Institution (from CdtrAgt)
    add_field_57(&mut fields, &tx_info.creditor_agent, warnings)?;

    // Field 58A or 58D: Beneficiary Institution (from Cdtr)
    add_field_58(&mut fields, &tx_info.creditor, warnings)?;

    // Field 72: Sender to Receiver Information (from InstrForNxtAgt, optional)
    if let Some(ref instructions) = tx_info.instruction_for_next_agent {
        add_field_72(&mut fields, instructions, warnings)?;
    }

    // Serialize fields to text block content
    let content = paymsg_mt::serializer::serialize_fields(&fields);

    Ok(TextBlock { content })
}

/// Add field 52A or 52D (Ordering Institution)
fn add_field_52(
    fields: &mut Vec<MtField>,
    party: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<(), PaymsgError> {
    if let Some(ref bic) = party.financial_institution_id.bic {
        // Use 52A with BIC
        let bic8 = BicNormalizer::to_bic8(bic);
        fields.push(MtField::new("52A", bic8));
    } else if let Some(ref name) = party.financial_institution_id.name {
        // Use 52D with name and address
        let mut lines = vec![name.clone()];
        if let Some(ref addr) = party.financial_institution_id.postal_address {
            if let Some(ref addr_lines) = addr.address_line {
                lines.extend(addr_lines.iter().take(3).cloned()); // MT allows 4 lines total (name + 3 address)
            }
        }
        let value = lines.join("\n");
        fields.push(MtField::new("52D", value));
    } else {
        return Err(PaymsgError::TranslationError(
            "Instructing agent must have BIC or Name for field 52".to_string(),
        ));
    }

    Ok(())
}

/// Add field 56A or 56D (Intermediary Institution)
fn add_field_56(
    fields: &mut Vec<MtField>,
    party: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<(), PaymsgError> {
    if let Some(ref bic) = party.financial_institution_id.bic {
        // Use 56A with BIC
        let bic8 = BicNormalizer::to_bic8(bic);
        fields.push(MtField::new("56A", bic8));
    } else if let Some(ref name) = party.financial_institution_id.name {
        // Use 56D with name and address
        let mut lines = vec![name.clone()];
        if let Some(ref addr) = party.financial_institution_id.postal_address {
            if let Some(ref addr_lines) = addr.address_line {
                lines.extend(addr_lines.iter().take(3).cloned());
            }
        }
        let value = lines.join("\n");
        fields.push(MtField::new("56D", value));
    } else {
        warnings.push(DataLossWarning::new(
            "Field56",
            DataLossCategory::OptionalFieldOmitted,
            "Intermediary agent has no BIC or Name, omitting field 56",
        ));
    }

    Ok(())
}

/// Add field 57A or 57D (Account With Institution)
fn add_field_57(
    fields: &mut Vec<MtField>,
    party: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<(), PaymsgError> {
    if let Some(ref bic) = party.financial_institution_id.bic {
        // Use 57A with BIC
        let bic8 = BicNormalizer::to_bic8(bic);
        fields.push(MtField::new("57A", bic8));
    } else if let Some(ref name) = party.financial_institution_id.name {
        // Use 57D with name and address
        let mut lines = vec![name.clone()];
        if let Some(ref addr) = party.financial_institution_id.postal_address {
            if let Some(ref addr_lines) = addr.address_line {
                lines.extend(addr_lines.iter().take(3).cloned());
            }
        }
        let value = lines.join("\n");
        fields.push(MtField::new("57D", value));
    } else {
        return Err(PaymsgError::TranslationError(
            "Creditor agent must have BIC or Name for field 57".to_string(),
        ));
    }

    Ok(())
}

/// Add field 58A or 58D (Beneficiary Institution)
fn add_field_58(
    fields: &mut Vec<MtField>,
    party: &paymsg_iso20022::pacs008::BranchAndFinancialInstitutionIdentification,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<(), PaymsgError> {
    if let Some(ref bic) = party.financial_institution_id.bic {
        // Use 58A with BIC
        let bic8 = BicNormalizer::to_bic8(bic);
        fields.push(MtField::new("58A", bic8));
    } else if let Some(ref name) = party.financial_institution_id.name {
        // Use 58D with name and address
        let mut lines = vec![name.clone()];
        if let Some(ref addr) = party.financial_institution_id.postal_address {
            if let Some(ref addr_lines) = addr.address_line {
                lines.extend(addr_lines.iter().take(3).cloned());
            }
        }
        let value = lines.join("\n");
        fields.push(MtField::new("58D", value));
    } else {
        return Err(PaymsgError::TranslationError(
            "Creditor must have BIC or Name for field 58".to_string(),
        ));
    }

    Ok(())
}

/// Add field 72 (Sender to Receiver Information)
fn add_field_72(
    fields: &mut Vec<MtField>,
    instructions: &[paymsg_iso20022::pacs008::InstructionForNextAgent],
    warnings: &mut Vec<DataLossWarning>,
) -> Result<(), PaymsgError> {
    // Get first instruction (MT202 field 72 allows up to 6 lines of 35 chars = 210 chars max)
    if let Some(instr) = instructions.first() {
        if let Some(ref info) = instr.instruction_information {
            // Truncate to 210 chars if needed
            let truncated = if info.len() > 210 {
                warnings.push(
                    DataLossWarning::new(
                        "Field72",
                        DataLossCategory::Truncation,
                        format!(
                            "Instruction info truncated to 210 chars, original was {} chars",
                            info.len()
                        ),
                    )
                    .with_original_value(info),
                );
                &info[..210]
            } else {
                info.as_str()
            };

            fields.push(MtField::new("72", truncated));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bic_normalization() {
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFFXXX"), "DEUTDEFF");
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFFABC"), "DEUTDEFFABC");
        assert_eq!(BicNormalizer::to_bic8("DEUTDEFF"), "DEUTDEFF");
    }
}
