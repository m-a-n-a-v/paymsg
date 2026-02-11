/// MT103 to pacs.008 translation
///
/// This module implements the translation from SWIFT MT103 (Single Customer Credit Transfer)
/// to ISO 20022 pacs.008.001.10 (FIToFICustomerCreditTransfer).

use chrono::Utc;
use paymsg_core::{Iban, PaymsgError};
use paymsg_iso20022::pacs008;
use paymsg_mt::{MtField, MtMessage};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::types::{
    AmountConverter, BicNormalizer, ChargeBearerConverter, DataLossWarning,
    DateConverter, TranslationResult,
};

/// Translate MT103 message to pacs.008 document
pub fn translate(mt_message: &MtMessage) -> Result<TranslationResult<pacs008::Document>, PaymsgError> {
    let mut warnings = Vec::new();

    // Parse Block 4 fields
    let fields = mt_message.block4.parse_fields()?;
    let field_map = build_field_map(&fields);

    // Build GroupHeader
    let group_header = build_group_header(mt_message, &field_map, &mut warnings)?;

    // Build CreditTransferTransactionInformation
    let credit_transfer = build_credit_transfer_transaction(mt_message, &field_map, &mut warnings)?;

    // Create pacs.008 Document
    let document = pacs008::Document {
        fi_to_fi_customer_credit_transfer: pacs008::FIToFICstmrCdtTrf {
            group_header,
            credit_transfer_transaction_information: vec![credit_transfer],
        },
    };

    Ok(TranslationResult::with_warnings(document, warnings))
}

/// Build field map for easy lookup
fn build_field_map(fields: &[MtField]) -> std::collections::HashMap<String, Vec<MtField>> {
    let mut map: std::collections::HashMap<String, Vec<MtField>> = std::collections::HashMap::new();
    for field in fields {
        map.entry(field.tag.clone()).or_default().push(field.clone());
    }
    map
}

/// Build GroupHeader from MT103 message
fn build_group_header(
    mt_message: &MtMessage,
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<pacs008::GroupHeader, PaymsgError> {
    // MsgId: Generate unique UUID or use field 20
    let msg_id = if let Some(field20) = field_map.get("20").and_then(|v| v.first()) {
        format!("MT103-{}", field20.value.trim())
    } else {
        Uuid::new_v4().to_string()
    };

    // CreDtTm: Use current timestamp (no equivalent in MT103)
    let creation_date_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // NbOfTxs: Always "1" for MT103 (single transaction)
    let number_of_transactions = "1".to_string();

    // InstgAgt: Extract from Block 1 (sender's BIC)
    let instructing_agent = extract_instructing_agent(&mt_message.block1)?;

    // InstdAgt: Extract from Block 2 (receiver's BIC)
    let instructed_agent = extract_instructed_agent(&mt_message.block2)?;

    // TtlIntrBkSttlmAmt: Optional, use field 32A amount if present
    let total_interbank_settlement_amount = if let Some(field32a) = field_map.get("32A").and_then(|v| v.first()) {
        if let (Some(currency), Some(amount)) = (
            field32a.subfields.get("currency"),
            field32a.subfields.get("amount"),
        ) {
            let amount_mx = AmountConverter::mt_to_mx(amount);
            Some(pacs008::ActiveCurrencyAndAmount {
                currency: currency.clone(),
                value: Decimal::from_str(&amount_mx).map_err(|e| {
                    PaymsgError::ParseError(format!("Invalid amount in field 32A: {}", e))
                })?,
            })
        } else {
            None
        }
    } else {
        None
    };

    // IntrBkSttlmDt: Optional, use field 32A date if present
    let interbank_settlement_date = if let Some(field32a) = field_map.get("32A").and_then(|v| v.first()) {
        field32a
            .subfields
            .get("date")
            .map(|date| DateConverter::mt_to_mx(date))
            .transpose()?
    } else {
        None
    };

    Ok(pacs008::GroupHeader {
        message_id: msg_id,
        creation_date_time,
        number_of_transactions,
        total_interbank_settlement_amount,
        interbank_settlement_date,
        settlement_information: None,
        instructing_agent,
        instructed_agent,
    })
}

/// Extract instructing agent (sender) from Block 1
fn extract_instructing_agent(
    block1: &paymsg_mt::blocks::BasicHeader,
) -> Result<pacs008::BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Extract BIC from logical terminal address (first 8 chars)
    let bic = if block1.logical_terminal_address.len() >= 8 {
        BicNormalizer::to_bic11(&block1.logical_terminal_address[..8])
    } else {
        return Err(PaymsgError::ParseError(
            "Invalid logical terminal address in Block 1".to_string(),
        ));
    };

    Ok(pacs008::BranchAndFinancialInstitutionIdentification {
        financial_institution_id: pacs008::FinancialInstitutionIdentification {
            bic: Some(bic),
            name: None,
            postal_address: None,
        },
    })
}

/// Extract instructed agent (receiver) from Block 2
fn extract_instructed_agent(
    block2: &paymsg_mt::blocks::ApplicationHeader,
) -> Result<pacs008::BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Use the BIC field (contains destination BIC for Input or sender BIC for Output)
    let bic = BicNormalizer::to_bic11(&block2.bic);

    Ok(pacs008::BranchAndFinancialInstitutionIdentification {
        financial_institution_id: pacs008::FinancialInstitutionIdentification {
            bic: Some(bic),
            name: None,
            postal_address: None,
        },
    })
}

/// Build CreditTransferTransactionInformation from MT103 fields
fn build_credit_transfer_transaction(
    mt_message: &MtMessage,
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<pacs008::CreditTransferTransactionInformation, PaymsgError> {
    // PaymentIdentification
    let payment_id = build_payment_identification(mt_message, field_map)?;

    // PaymentTypeInformation (optional)
    let payment_type_information = build_payment_type_information(field_map);

    // IntrBkSttlmAmt (mandatory) - from field 32A
    let (interbank_settlement_amount, interbank_settlement_date) = extract_field_32a(field_map)?;

    // InstdAmt (optional) - from field 33B
    let instructed_amount = extract_field_33b(field_map)?;

    // XchgRate (optional) - from field 36
    let exchange_rate = extract_field_36(field_map)?;

    // ChrgBr (mandatory) - from field 71A
    let charge_bearer = extract_field_71a(field_map)?;

    // Charges information - from fields 71F, 71G
    let charges_information = build_charges_information(field_map)?;

    // Debtor - from field 50K/50A/50F
    let debtor = build_debtor_party(field_map)?;

    // DebtorAccount - from field 50K/50A/50F account line
    let debtor_account = build_debtor_account(field_map)?;

    // DebtorAgent - from field 52A/52D
    let debtor_agent = build_debtor_agent(field_map)?;

    // IntrmyAgt1 (optional) - from field 56A/56C/56D
    let intermediary_agent1 = build_intermediary_agent(field_map);

    // CdtrAgt (mandatory) - from field 57A/57B/57C/57D
    let creditor_agent = build_creditor_agent(field_map)?;

    // Creditor - from field 59/59A/59F
    let creditor = build_creditor_party(field_map)?;

    // CreditorAccount - from field 59/59A/59F account line
    let creditor_account = build_creditor_account(field_map)?;

    // InstrForCdtrAgt (optional) - from field 23E
    let instruction_for_creditor_agent = build_instruction_for_creditor_agent(field_map)?;

    // InstrForNxtAgt (optional) - from field 72
    let instruction_for_next_agent = build_instruction_for_next_agent(field_map)?;

    // Purpose (optional) - from field 26T
    let purpose = build_purpose(field_map);

    // RegulatoryReporting (optional) - from field 77B
    let regulatory_reporting = build_regulatory_reporting(field_map)?;

    // RemittanceInformation (optional) - from field 70
    let remittance_information = build_remittance_information(field_map)?;

    Ok(pacs008::CreditTransferTransactionInformation {
        payment_id,
        payment_type_information,
        interbank_settlement_amount,
        interbank_settlement_date,
        instructed_amount,
        exchange_rate,
        charge_bearer,
        charges_information: if charges_information.is_empty() {
            None
        } else {
            Some(charges_information)
        },
        intermediary_agent_1: intermediary_agent1,
        intermediary_agent_2: None,
        intermediary_agent_3: None,
        debtor,
        debtor_account,
        debtor_agent,
        creditor_agent,
        creditor,
        creditor_account,
        instruction_for_creditor_agent: if instruction_for_creditor_agent.is_empty() {
            None
        } else {
            Some(instruction_for_creditor_agent)
        },
        instruction_for_next_agent: if instruction_for_next_agent.is_empty() {
            None
        } else {
            Some(instruction_for_next_agent)
        },
        purpose,
        regulatory_reporting: if regulatory_reporting.is_empty() {
            None
        } else {
            Some(regulatory_reporting)
        },
        remittance_information,
    })
}

/// Build PaymentIdentification
fn build_payment_identification(
    mt_message: &MtMessage,
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::PaymentIdentification, PaymsgError> {
    // InstrId: from field 20
    let instruction_id = field_map
        .get("20")
        .and_then(|v| v.first())
        .map(|f| f.value.trim().to_string());

    // EndToEndId: from Block 3 tag 121 (UETR) if present, otherwise from field 20
    let end_to_end_id = if let Some(ref block3) = mt_message.block3 {
        block3.tags.get("121").cloned()
            .or_else(|| instruction_id.clone())
            .unwrap_or_else(|| "NOTPROVIDED".to_string())
    } else {
        instruction_id.clone().unwrap_or_else(|| "NOTPROVIDED".to_string())
    };

    // UETR: from Block 3 tag 121 if present
    let uetr = mt_message
        .block3
        .as_ref()
        .and_then(|b3| b3.user_tags.get("121").cloned());

    // TxId: optional, not in MT103
    let transaction_id = None;

    Ok(pacs008::PaymentIdentification {
        instruction_id,
        end_to_end_id,
        transaction_id,
        uetr,
    })
}

/// Build PaymentTypeInformation (optional)
fn build_payment_type_information(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Option<pacs008::PaymentTypeInformation> {
    // LocalInstrument from field 23B
    let local_instrument = field_map.get("23B").and_then(|v| v.first()).map(|f| {
        pacs008::LocalInstrument {
            code: None,
            proprietary: Some(f.value.trim().to_string()),
        }
    });

    if local_instrument.is_some() {
        Some(pacs008::PaymentTypeInformation {
            instruction_priority: None,
            service_level: None,
            local_instrument,
            category_purpose: None,
        })
    } else {
        None
    }
}

/// Extract field 32A (Value Date, Currency, Amount)
fn extract_field_32a(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<(pacs008::ActiveCurrencyAndAmount, Option<String>), PaymsgError> {
    let field32a = field_map
        .get("32A")
        .and_then(|v| v.first())
        .ok_or_else(|| PaymsgError::TranslationError("Missing mandatory field 32A".to_string()))?;

    let currency = field32a
        .subfields
        .get("currency")
        .ok_or_else(|| PaymsgError::TranslationError("Missing currency in field 32A".to_string()))?
        .clone();

    let amount_mt = field32a
        .subfields
        .get("amount")
        .ok_or_else(|| PaymsgError::TranslationError("Missing amount in field 32A".to_string()))?;

    let amount_mx = AmountConverter::mt_to_mx(amount_mt);
    let value = Decimal::from_str(&amount_mx)
        .map_err(|e| PaymsgError::ParseError(format!("Invalid amount in field 32A: {}", e)))?;

    let date = field32a
        .subfields
        .get("date")
        .map(|d| DateConverter::mt_to_mx(d))
        .transpose()?;

    Ok((
        pacs008::ActiveCurrencyAndAmount { currency, value },
        date,
    ))
}

/// Extract field 33B (Instructed Amount) - optional
fn extract_field_33b(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Option<pacs008::ActiveOrHistoricCurrencyAndAmount>, PaymsgError> {
    if let Some(field33b) = field_map.get("33B").and_then(|v| v.first()) {
        let currency = field33b
            .subfields
            .get("currency")
            .ok_or_else(|| PaymsgError::TranslationError("Missing currency in field 33B".to_string()))?
            .clone();

        let amount_mt = field33b
            .subfields
            .get("amount")
            .ok_or_else(|| PaymsgError::TranslationError("Missing amount in field 33B".to_string()))?;

        let amount_mx = AmountConverter::mt_to_mx(amount_mt);
        let value = Decimal::from_str(&amount_mx)
            .map_err(|e| PaymsgError::ParseError(format!("Invalid amount in field 33B: {}", e)))?;

        Ok(Some(pacs008::ActiveOrHistoricCurrencyAndAmount {
            currency,
            value,
        }))
    } else {
        Ok(None)
    }
}

/// Extract field 36 (Exchange Rate) - optional
fn extract_field_36(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Option<String>, PaymsgError> {
    Ok(field_map
        .get("36")
        .and_then(|v| v.first())
        .map(|f| AmountConverter::mt_to_mx(f.value.trim())))
}

/// Extract field 71A (Details of Charges)
fn extract_field_71a(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<String, PaymsgError> {
    let field71a = field_map
        .get("71A")
        .and_then(|v| v.first())
        .ok_or_else(|| PaymsgError::TranslationError("Missing mandatory field 71A".to_string()))?;

    ChargeBearerConverter::mt_to_mx(field71a.value.trim())
}

/// Build charges information from fields 71F and 71G
fn build_charges_information(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Vec<pacs008::ChargesInformation>, PaymsgError> {
    let mut charges = Vec::new();

    // Field 71F: Sender's charges (can repeat)
    if let Some(field71f_list) = field_map.get("71F") {
        for field in field71f_list {
            if let (Some(currency), Some(amount_mt)) = (
                field.subfields.get("currency"),
                field.subfields.get("amount"),
            ) {
                let amount_mx = AmountConverter::mt_to_mx(amount_mt);
                let value = Decimal::from_str(&amount_mx)
                    .map_err(|e| PaymsgError::ParseError(format!("Invalid amount in field 71F: {}", e)))?;

                charges.push(pacs008::ChargesInformation {
                    amount: pacs008::ActiveOrHistoricCurrencyAndAmount {
                        currency: currency.clone(),
                        value,
                    },
                    agent: None, // Optional: could be set to sender's institution
                });
            }
        }
    }

    // Field 71G: Receiver's charges
    if let Some(field71g) = field_map.get("71G").and_then(|v| v.first()) {
        if let (Some(currency), Some(amount_mt)) = (
            field71g.subfields.get("currency"),
            field71g.subfields.get("amount"),
        ) {
            let amount_mx = AmountConverter::mt_to_mx(amount_mt);
            let value = Decimal::from_str(&amount_mx)
                .map_err(|e| PaymsgError::ParseError(format!("Invalid amount in field 71G: {}", e)))?;

            charges.push(pacs008::ChargesInformation {
                amount: pacs008::ActiveOrHistoricCurrencyAndAmount {
                    currency: currency.clone(),
                    value,
                },
                agent: None, // Optional: could be set to receiver's institution
            });
        }
    }

    Ok(charges)
}

/// Build Debtor party from field 50K/50A/50F
fn build_debtor_party(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::PartyIdentification, PaymsgError> {
    // Try 50K (most common), then 50A, then 50F
    if let Some(field50k) = field_map.get("50K").and_then(|v| v.first()) {
        return build_party_from_field_k(field50k);
    }

    if let Some(field50a) = field_map.get("50A").and_then(|v| v.first()) {
        return build_party_from_field_a(field50a);
    }

    Err(PaymsgError::TranslationError(
        "Missing mandatory field 50 (Ordering Customer)".to_string(),
    ))
}

/// Build party from field option K (Name & Address)
fn build_party_from_field_k(field: &MtField) -> Result<pacs008::PartyIdentification, PaymsgError> {
    let name = field
        .subfields
        .get("name_address")
        .map(|na| {
            // Take first line or first 140 chars
            na.lines().next().unwrap_or("").chars().take(140).collect()
        });

    let postal_address = field.subfields.get("name_address").map(|na| {
        // Skip first line (used for name), take remaining as address
        let lines: Vec<&str> = na.lines().collect();
        let address_lines: Vec<String> = lines.iter().skip(1).take(7).map(|s| s.to_string()).collect();

        pacs008::PostalAddress {
            address_line: if address_lines.is_empty() {
                None
            } else {
                Some(address_lines)
            },
            street_name: None,
            building_number: None,
            post_code: None,
            town_name: None,
            country_sub_division: None,
            country: None,
        }
    });

    Ok(pacs008::PartyIdentification {
        name,
        postal_address,
        id: None,
        country_of_residence: None,
    })
}

/// Build party from field option A (Account/BIC)
fn build_party_from_field_a(field: &MtField) -> Result<pacs008::PartyIdentification, PaymsgError> {
    let bic = field
        .subfields
        .get("bic")
        .map(|b| BicNormalizer::to_bic11(b));

    let id = bic.map(|bic_value| pacs008::Party {
        organization_id: Some(pacs008::OrganizationIdentification {
            any_bic: Some(bic_value),
            other: None,
        }),
        private_id: None,
    });

    Ok(pacs008::PartyIdentification {
        name: None,
        postal_address: None,
        id,
        country_of_residence: None,
    })
}

/// Build DebtorAccount
fn build_debtor_account(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::CashAccount, PaymsgError> {
    // Try to extract account from field 50K, 50A, or 50F
    let account_str = field_map
        .get("50K")
        .or_else(|| field_map.get("50A"))
        .or_else(|| field_map.get("50F"))
        .and_then(|v| v.first())
        .and_then(|f| f.subfields.get("account"))
        .map(|a| a.trim().to_string());

    if let Some(acc) = account_str {
        // Check if IBAN format
        let account_id = if Iban::from_str(&acc).is_ok() {
            pacs008::AccountIdentification {
                iban: Some(acc),
                other: None,
            }
        } else {
            pacs008::AccountIdentification {
                iban: None,
                other: Some(pacs008::GenericAccountIdentification {
                    id: acc,
                    scheme_name: None,
                    issuer: None,
                }),
            }
        };

        Ok(pacs008::CashAccount {
            id: account_id,
            account_type: None,
            currency: None,
            name: None,
        })
    } else {
        // No account specified - create a minimal structure
        Err(PaymsgError::TranslationError(
            "Missing account in field 50".to_string(),
        ))
    }
}

/// Build DebtorAgent from field 52A/52D
fn build_debtor_agent(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Try 52A (BIC), then 52D (Name & Address)
    if let Some(field52a) = field_map.get("52A").and_then(|v| v.first()) {
        let bic = field52a
            .subfields
            .get("bic")
            .map(|b| BicNormalizer::to_bic11(b));

        return Ok(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic,
                name: None,
                postal_address: None,
            },
        });
    }

    if let Some(field52d) = field_map.get("52D").and_then(|v| v.first()) {
        let name_address = field52d.subfields.get("name_address");
        let name = name_address
            .and_then(|na| na.lines().next())
            .map(|s| s.chars().take(140).collect());

        return Ok(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic: None,
                name,
                postal_address: None,
            },
        });
    }

    Err(PaymsgError::TranslationError(
        "Missing mandatory field 52 (Ordering Institution)".to_string(),
    ))
}

/// Build IntermediaryAgent from field 56A/56C/56D (optional)
fn build_intermediary_agent(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Option<pacs008::BranchAndFinancialInstitutionIdentification> {
    // Try 56A (BIC), then 56C (Party ID), then 56D (Name & Address)
    if let Some(field56a) = field_map.get("56A").and_then(|v| v.first()) {
        let bic = field56a.subfields.get("bic").map(|b| BicNormalizer::to_bic11(b));

        return Some(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic,
                name: None,
                postal_address: None,
            },
        });
    }

    if let Some(field56c) = field_map.get("56C").and_then(|v| v.first()) {
        let member_id = field56c.value.trim().strip_prefix('/').unwrap_or(field56c.value.trim());

        return Some(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic: None,
                name: Some(member_id.to_string()), // Use name field to store party identifier
                postal_address: None,
            },
        });
    }

    None
}

/// Build CreditorAgent from field 57A/57B/57C/57D
fn build_creditor_agent(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::BranchAndFinancialInstitutionIdentification, PaymsgError> {
    // Try 57A (BIC), then 57C (Party ID), then 57D (Name & Address)
    if let Some(field57a) = field_map.get("57A").and_then(|v| v.first()) {
        let bic = field57a.subfields.get("bic").map(|b| BicNormalizer::to_bic11(b));

        return Ok(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic,
                name: None,
                postal_address: None,
            },
        });
    }

    if let Some(field57c) = field_map.get("57C").and_then(|v| v.first()) {
        let member_id = field57c.value.trim().strip_prefix('/').unwrap_or(field57c.value.trim());

        return Ok(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic: None,
                name: Some(member_id.to_string()), // Use name field to store party identifier
                postal_address: None,
            },
        });
    }

    if let Some(field57d) = field_map.get("57D").and_then(|v| v.first()) {
        let name_address = field57d.subfields.get("name_address");
        let name = name_address
            .and_then(|na| na.lines().next())
            .map(|s| s.chars().take(140).collect());

        return Ok(pacs008::BranchAndFinancialInstitutionIdentification {
            financial_institution_id: pacs008::FinancialInstitutionIdentification {
                bic: None,
                name,
                postal_address: None,
            },
        });
    }

    Err(PaymsgError::TranslationError(
        "Missing mandatory field 57 (Account With Institution)".to_string(),
    ))
}

/// Build Creditor party from field 59/59A/59F
fn build_creditor_party(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::PartyIdentification, PaymsgError> {
    // Try 59 (most common), then 59A
    if let Some(field59) = field_map.get("59").and_then(|v| v.first()) {
        return build_party_from_field_k(field59);
    }

    if let Some(field59a) = field_map.get("59A").and_then(|v| v.first()) {
        return build_party_from_field_a(field59a);
    }

    Err(PaymsgError::TranslationError(
        "Missing mandatory field 59 (Beneficiary Customer)".to_string(),
    ))
}

/// Build CreditorAccount
fn build_creditor_account(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<pacs008::CashAccount, PaymsgError> {
    // Try to extract account from field 59, 59A, or 59F
    let account_str = field_map
        .get("59")
        .or_else(|| field_map.get("59A"))
        .or_else(|| field_map.get("59F"))
        .and_then(|v| v.first())
        .and_then(|f| f.subfields.get("account"))
        .map(|a| a.trim().to_string());

    if let Some(acc) = account_str {
        // Check if IBAN format
        let account_id = if Iban::from_str(&acc).is_ok() {
            pacs008::AccountIdentification {
                iban: Some(acc),
                other: None,
            }
        } else {
            pacs008::AccountIdentification {
                iban: None,
                other: Some(pacs008::GenericAccountIdentification {
                    id: acc,
                    scheme_name: None,
                    issuer: None,
                }),
            }
        };

        Ok(pacs008::CashAccount {
            id: account_id,
            account_type: None,
            currency: None,
            name: None,
        })
    } else {
        Err(PaymsgError::TranslationError(
            "Missing account in field 59".to_string(),
        ))
    }
}

/// Build InstructionForCreditorAgent from field 23E (can repeat)
fn build_instruction_for_creditor_agent(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Vec<pacs008::InstructionForCreditorAgent>, PaymsgError> {
    let mut instructions = Vec::new();

    if let Some(field23e_list) = field_map.get("23E") {
        for field in field23e_list {
            // Parse instruction code (format: CODE or CODE/additional info)
            let value = field.value.trim();
            let parts: Vec<&str> = value.split('/').collect();
            let code = parts[0];
            let info = if parts.len() > 1 {
                Some(parts[1..].join("/"))
            } else {
                None
            };

            // Map MT instruction codes to MX codes
            let mx_code = match code {
                "PHOB" | "PHOI" | "TELB" | "TELI" => Some("PHON".to_string()),
                "HOLD" => Some("HOLD".to_string()),
                _ => None,
            };

            instructions.push(pacs008::InstructionForCreditorAgent {
                code: mx_code,
                instruction_information: info,
            });
        }
    }

    Ok(instructions)
}

/// Build InstructionForNextAgent from field 72 (optional)
fn build_instruction_for_next_agent(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Vec<pacs008::InstructionForNextAgent>, PaymsgError> {
    if let Some(field72) = field_map.get("72").and_then(|v| v.first()) {
        let text = field72.subfields.get("text").cloned()
            .unwrap_or_else(|| field72.value.trim().to_string());

        // Truncate to 140 chars if needed (MX limit)
        let truncated: String = text.chars().take(140).collect();

        Ok(vec![pacs008::InstructionForNextAgent {
            code: None,
            instruction_information: Some(truncated),
        }])
    } else {
        Ok(Vec::new())
    }
}

/// Build Purpose from field 26T (optional)
fn build_purpose(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Option<pacs008::Purpose> {
    field_map.get("26T").and_then(|v| v.first()).map(|field| {
        pacs008::Purpose {
            code: None,
            proprietary: Some(field.value.trim().to_string()),
        }
    })
}

/// Build RegulatoryReporting from field 77B (optional)
fn build_regulatory_reporting(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Vec<pacs008::RegulatoryReporting>, PaymsgError> {
    if let Some(field77b) = field_map.get("77B").and_then(|v| v.first()) {
        let value = field77b.value.trim();

        // Split into multiple details (max 35 chars each)
        let mut details = Vec::new();
        for chunk in value.chars().collect::<Vec<_>>().chunks(35) {
            let chunk_str: String = chunk.iter().collect();
            details.push(pacs008::StructuredRegulatoryReporting {
                information: Some(chunk_str),
                type_code: None,
                amount: None,
                country: None,
                date: None,
            });
        }

        Ok(vec![pacs008::RegulatoryReporting { details }])
    } else {
        Ok(Vec::new())
    }
}

/// Build RemittanceInformation from field 70 (optional)
fn build_remittance_information(
    field_map: &std::collections::HashMap<String, Vec<MtField>>,
) -> Result<Option<pacs008::RemittanceInformation>, PaymsgError> {
    if let Some(field70) = field_map.get("70").and_then(|v| v.first()) {
        let text = field70.subfields.get("text").cloned()
            .unwrap_or_else(|| field70.value.trim().to_string());

        // Concatenate lines with space, max 140 chars per unstructured element
        let concatenated = text.replace('\n', " ");
        let truncated: String = concatenated.chars().take(140).collect();

        Ok(Some(pacs008::RemittanceInformation {
            unstructured: Some(vec![truncated]),
            structured: None,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_field_map() {
        let fields = vec![
            MtField {
                tag: "20".to_string(),
                value: "REF123".to_string(),
                subfields: std::collections::HashMap::new(),
            },
            MtField {
                tag: "32A".to_string(),
                value: "260210EUR1000,00".to_string(),
                subfields: std::collections::HashMap::new(),
            },
        ];

        let map = build_field_map(&fields);
        assert!(map.contains_key("20"));
        assert!(map.contains_key("32A"));
        assert_eq!(map.get("20").unwrap().len(), 1);
    }
}
