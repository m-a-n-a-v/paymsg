//! Translate MT940 (SWIFT) to camt.053 (ISO 20022)
//!
//! This module implements translation from SWIFT MT940 (Customer Statement Message)
//! to ISO 20022 camt.053.001.10 (BankToCustomerStatement).
//!
//! Key transformations:
//! - Split: field 60F/60M/62F/62M/64/65 → Balance elements with different BalTp codes
//! - Split: field 61 → Entry elements with amount, dates, transaction type
//! - Split: field 28C → LglSeqNb + ElctrncSeqNb
//! - Split: field 25P → Account + Servicer BIC
//! - Field mapping: field 20 → Stmt/Id
//! - Field mapping: field 25 → Stmt/Acct
//! - Field mapping: field 86 → Entry details (remittance info or additional info)
//! - Balance type codes: 60F/60M → OPBD, 62F/62M → CLBD, 64 → CLAV, 65 → FWAV
//! - D/C indicator mapping: C → CRDT, D → DBIT, RC/RD → reversal indicators

use crate::types::{
    AmountConverter, BicNormalizer, DateConverter, DataLossCategory, DataLossWarning,
    TranslationResult,
};
use chrono::Utc;
use paymsg_core::{Iban, PaymsgError};
use paymsg_iso20022::camt053::{
    AccountIdentification, AccountStatement, ActiveOrHistoricCurrencyAndAmount, Balance,
    BalanceSubType, BalanceType, BankToCustomerStatement, BankTransactionCode,
    BranchAndFinancialInstitutionIdentification, CashAccount, CodeOrProprietary, DateOrDateTime,
    Document, Entry, EntryDetails, EntryStatus, FinancialInstitutionIdentification,
    GenericAccountIdentification, GroupHeader, ProprietaryBankTransactionCode,
    RemittanceInformation, TransactionDetails, TransactionReferences,
};
use paymsg_mt::MtMessage;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

/// Translate an MT940 message to camt.053.
///
/// Converts a SWIFT MT940 (Customer Statement) message
/// to an ISO 20022 camt.053 (Bank to Customer Statement) message.
///
/// # Errors
///
/// Returns `PaymsgError::TranslationError` if:
/// - Required fields are missing in MT940
/// - Field values cannot be parsed or converted
pub fn translate(mt940: &MtMessage) -> Result<TranslationResult<Document>, PaymsgError> {
    let mut warnings = Vec::new();

    // Build Group Header from message envelope
    let grp_hdr = build_group_header(mt940, &mut warnings)?;

    // Build Account Statement from Block 4 fields
    let stmt = build_account_statement(mt940, &mut warnings)?;

    let bank_to_customer_statement = BankToCustomerStatement {
        group_header: grp_hdr,
        statement: vec![stmt],
    };

    let document = Document {
        bank_to_customer_statement,
    };

    Ok(TranslationResult::with_warnings(document, warnings))
}

/// Build Group Header from MT940 message
fn build_group_header(
    _mt940: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<GroupHeader, PaymsgError> {
    // Generate message ID (not present in MT940)
    let message_id = format!("MT940-{}", Uuid::new_v4());
    warnings.push(DataLossWarning::new(
        "GrpHdr/MsgId",
        DataLossCategory::NoEquivalent,
        "Generated message ID (not present in MT940)",
    ));

    // Creation date/time - use current time
    let creation_date_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    warnings.push(DataLossWarning::new(
        "GrpHdr/CreDtTm",
        DataLossCategory::NoEquivalent,
        "Set to current timestamp (MT940 has no message creation timestamp)",
    ));

    Ok(GroupHeader {
        message_id,
        creation_date_time,
        message_recipient: None,
        message_pagination: None,
        additional_information: None,
    })
}

/// Build Account Statement from Block 4 fields
fn build_account_statement(
    mt940: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<AccountStatement, PaymsgError> {
    // Parse fields from Block 4
    let parsed_fields = mt940.block4.parse_fields()?;

    // Build field map for easy access
    let mut field_map: HashMap<String, Vec<&paymsg_mt::MtField>> = HashMap::new();
    for field in &parsed_fields {
        field_map.entry(field.tag.clone()).or_default().push(field);
    }

    // Field 20: Statement Reference → Stmt/Id
    let statement_id = field_map
        .get("20")
        .and_then(|fields| fields.first())
        .map(|f| f.value.trim().to_string())
        .ok_or_else(|| PaymsgError::TranslationError("MT940 field 20 is mandatory".to_string()))?;

    // Truncate to 16 chars if needed (per mapping spec)
    let statement_id = if statement_id.len() > 16 {
        warnings.push(DataLossWarning::new(
            "Stmt/Id",
            DataLossCategory::Truncation,
            format!("Statement ID truncated from {} to 16 chars", statement_id.len()),
        ).with_original_value(&statement_id));
        statement_id[..16].to_string()
    } else {
        statement_id
    };

    // Field 28C: Statement Number/Sequence Number
    let (legal_seq_nb, electronic_seq_nb) = extract_field_28c(&field_map, warnings)?;

    // Creation date/time - use closing balance date or current time
    let creation_date_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    warnings.push(DataLossWarning::new(
        "Stmt/CreDtTm",
        DataLossCategory::NoEquivalent,
        "Set to current timestamp (MT940 has no creation timestamp)",
    ));

    // Field 25/25P: Account Identification
    let account = extract_account(&field_map, warnings)?;

    // Extract all balances (60F/60M, 62F/62M, 64, 65)
    let balances = extract_balances(&field_map, warnings)?;

    // Get currency from balances (all must be same currency)
    let currency = balances.first()
        .map(|b| b.amount.currency.clone())
        .ok_or_else(|| PaymsgError::TranslationError("No balance fields found in MT940".to_string()))?;

    // Set account currency
    let mut account = account;
    account.currency = Some(currency.clone());

    // Field 61: Statement Lines → Entries
    let entries = extract_entries(&field_map, &currency, warnings)?;

    Ok(AccountStatement {
        id: statement_id,
        electronic_sequence_number: electronic_seq_nb,
        legal_sequence_number: legal_seq_nb,
        creation_date_time,
        from_to_date: None,
        copy_duplicate_indicator: None,
        reporting_source: None,
        account,
        related_account: None,
        interest: None,
        balance: balances,
        transactions_summary: None,
        entry: if entries.is_empty() { None } else { Some(entries) },
        additional_statement_info: None,
    })
}

/// Extract field 28C: Statement Number/Sequence Number
fn extract_field_28c(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<(Option<u64>, Option<String>), PaymsgError> {
    let field_28c = field_map.get("28C").and_then(|fields| fields.first());

    if let Some(field) = field_28c {
        let value = field.value.trim();

        // Parse field 28C: format is StatementNumber[/SequenceNumber]
        if let Some(slash_pos) = value.find('/') {
            let stmt_num_str = &value[..slash_pos];
            let seq_num_str = &value[slash_pos + 1..];

            let legal_seq_nb = stmt_num_str.parse::<u64>().ok();
            let electronic_seq_nb = Some(seq_num_str.to_string());

            Ok((legal_seq_nb, electronic_seq_nb))
        } else {
            // No sequence number, just statement number
            let legal_seq_nb = value.parse::<u64>().ok();
            Ok((legal_seq_nb, None))
        }
    } else {
        Ok((None, None))
    }
}

/// Extract account identification from field 25 or 25P
fn extract_account(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<CashAccount, PaymsgError> {
    // Check for field 25P first (BIC/Account format)
    if let Some(field_25p) = field_map.get("25P").and_then(|fields| fields.first()) {
        let value = field_25p.value.trim();

        // Parse BIC/Account format
        if let Some(slash_pos) = value.find('/') {
            let bic_str = &value[..slash_pos];
            let account_str = &value[slash_pos + 1..];

            // Check if account is IBAN
            let account_id = if Iban::new(account_str).is_ok() {
                AccountIdentification {
                    iban: Some(account_str.to_string()),
                    other: None,
                }
            } else {
                AccountIdentification {
                    iban: None,
                    other: Some(GenericAccountIdentification {
                        id: account_str.to_string(),
                        scheme_name: None,
                        issuer: None,
                    }),
                }
            };

            // Build servicer with BIC
            let servicer = Some(BranchAndFinancialInstitutionIdentification {
                financial_institution_identification: FinancialInstitutionIdentification {
                    bic: Some(BicNormalizer::to_bic11(bic_str)),
                    clearing_system_member_identification: None,
                    name: None,
                    postal_address: None,
                    other: None,
                },
                branch_identification: None,
            });

            return Ok(CashAccount {
                id: account_id,
                account_type: None,
                currency: None,
                name: None,
                owner: None,
                servicer,
            });
        }
    }

    // Check for field 25 (simple account number)
    if let Some(field_25) = field_map.get("25").and_then(|fields| fields.first()) {
        let account_str = field_25.value.trim();

        // Check if account is IBAN
        let account_id = if Iban::new(account_str).is_ok() {
            AccountIdentification {
                iban: Some(account_str.to_string()),
                other: None,
            }
        } else {
            AccountIdentification {
                iban: None,
                other: Some(GenericAccountIdentification {
                    id: account_str.to_string(),
                    scheme_name: None,
                    issuer: None,
                }),
            }
        };

        return Ok(CashAccount {
            id: account_id,
            account_type: None,
            currency: None,
            name: None,
            owner: None,
            servicer: None,
        });
    }

    Err(PaymsgError::TranslationError(
        "MT940 must have field 25 or 25P for account identification".to_string(),
    ))
}

/// Extract all balance fields
fn extract_balances(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<Balance>, PaymsgError> {
    let mut balances = Vec::new();

    // Opening balance: 60F (final) or 60M (intermediate)
    if let Some(field_60f) = field_map.get("60F").and_then(|fields| fields.first()) {
        balances.push(parse_balance_field(field_60f, "OPBD", Some("FINAL"), warnings)?);
    } else if let Some(field_60m) = field_map.get("60M").and_then(|fields| fields.first()) {
        balances.push(parse_balance_field(field_60m, "OPBD", Some("INTERIM"), warnings)?);
    } else {
        return Err(PaymsgError::TranslationError(
            "MT940 must have either field 60F or 60M for opening balance".to_string(),
        ));
    }

    // Closing balance: 62F (final) or 62M (intermediate)
    if let Some(field_62f) = field_map.get("62F").and_then(|fields| fields.first()) {
        balances.push(parse_balance_field(field_62f, "CLBD", Some("FINAL"), warnings)?);
    } else if let Some(field_62m) = field_map.get("62M").and_then(|fields| fields.first()) {
        balances.push(parse_balance_field(field_62m, "CLBD", Some("INTERIM"), warnings)?);
    } else {
        return Err(PaymsgError::TranslationError(
            "MT940 must have either field 62F or 62M for closing balance".to_string(),
        ));
    }

    // Closing available balance: 64 (optional)
    if let Some(field_64) = field_map.get("64").and_then(|fields| fields.first()) {
        balances.push(parse_balance_field(field_64, "CLAV", None, warnings)?);
    }

    // Forward available balance: 65 (optional, can repeat)
    if let Some(fields_65) = field_map.get("65") {
        for field_65 in fields_65 {
            balances.push(parse_balance_field(field_65, "FWAV", None, warnings)?);
        }
    }

    Ok(balances)
}

/// Parse a balance field (60F/60M/62F/62M/64/65)
fn parse_balance_field(
    field: &paymsg_mt::MtField,
    balance_type_code: &str,
    sub_type: Option<&str>,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Balance, PaymsgError> {
    let value = field.value.trim();

    if value.len() < 11 {
        return Err(PaymsgError::ParseError(format!(
            "Balance field {} too short: expected at least 11 chars, got {}",
            field.tag, value.len()
        )));
    }

    // Extract D/C mark
    let dc_mark = &value[0..1];
    let credit_debit_indicator = match dc_mark {
        "C" => "CRDT",
        "D" => "DBIT",
        _ => return Err(PaymsgError::ParseError(format!(
            "Invalid D/C mark in balance field: {}", dc_mark
        ))),
    }.to_string();

    // Extract date (YYMMDD)
    let date_str = &value[1..7];
    let date_mx = DateConverter::mt_to_mx(date_str)?;

    // Extract currency (3 letters)
    let currency = value[7..10].to_string();

    // Extract amount (rest of string)
    let amount_str = &value[10..];
    let amount_mx_str = AmountConverter::mt_to_mx(amount_str);
    let amount_decimal = Decimal::from_str(&amount_mx_str).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount format: {} ({})", amount_mx_str, e))
    })?;

    let balance_type = BalanceType {
        code_or_proprietary: CodeOrProprietary {
            code: Some(balance_type_code.to_string()),
            proprietary: None,
        },
        sub_type: sub_type.map(|st| BalanceSubType {
            code: None,
            proprietary: Some(st.to_string()),
        }),
    };

    Ok(Balance {
        balance_type,
        amount: ActiveOrHistoricCurrencyAndAmount {
            currency,
            value: amount_decimal,
        },
        credit_debit_indicator,
        date: DateOrDateTime {
            date: Some(date_mx),
            date_time: None,
        },
        availability: None,
    })
}

/// Extract entries from field 61
fn extract_entries(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    statement_currency: &str,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<Entry>, PaymsgError> {
    let mut entries = Vec::new();

    // Get all field 61 entries
    let fields_61 = field_map.get("61");
    if fields_61.is_none() {
        return Ok(entries); // No entries, return empty list
    }

    // Get all field 86 entries (supplementary info for each field 61)
    let fields_86 = field_map.get("86").cloned().unwrap_or_default();

    // Process each field 61
    for (idx, field_61) in fields_61.unwrap().iter().enumerate() {
        // Parse field 61
        let entry = parse_field_61(field_61, statement_currency, warnings)?;

        // Check if there's a corresponding field 86
        let field_86 = fields_86.get(idx);
        let entry_with_details = if let Some(f86) = field_86 {
            add_field_86_to_entry(entry, f86, warnings)?
        } else {
            entry
        };

        entries.push(entry_with_details);
    }

    Ok(entries)
}

/// Parse field 61: Statement Line
fn parse_field_61(
    field: &paymsg_mt::MtField,
    statement_currency: &str,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Entry, PaymsgError> {
    // Field 61 parsing is complex - we need to extract subfields
    // The field should already be parsed by parse_field_61() in the MT parser
    let value = field.value.trim();

    // For simplicity, try to extract basic components manually
    // Format: YYMMDD[MMDD][D|C|RD|RC][FundsCode]AMOUNT[Type][CustomerRef][//BankRef][\nSupplementary]

    let mut pos = 0;

    // Check if there's a continuation line (supplementary details)
    let (main_line, supplementary_details) = if let Some(newline_pos) = value.find('\n') {
        let supp = value[newline_pos + 1..].trim().to_string();
        (&value[..newline_pos], if supp.is_empty() { None } else { Some(supp) })
    } else {
        (value, None)
    };

    // Extract value date (YYMMDD) - 6 digits
    if main_line.len() < 6 {
        return Err(PaymsgError::ParseError(
            "Field 61: insufficient length for value date".to_string(),
        ));
    }
    let value_date_str = &main_line[pos..pos + 6];
    let value_date = DateConverter::mt_to_mx(value_date_str)?;
    pos += 6;

    // Check for optional entry date (MMDD) - 4 digits
    let booking_date = if pos + 4 <= main_line.len()
        && main_line[pos..pos + 4].chars().all(|c| c.is_ascii_digit())
        && pos + 4 < main_line.len()
    {
        let char_after = main_line.chars().nth(pos + 4).unwrap();
        if char_after == 'D' || char_after == 'C' || char_after == 'R' {
            let entry_date_mmdd = &main_line[pos..pos + 4];
            pos += 4;

            // Combine with year from value date
            let year = &value_date[0..4];
            let entry_date = format!("{}-{}-{}", year, &entry_date_mmdd[0..2], &entry_date_mmdd[2..4]);
            Some(entry_date)
        } else {
            None
        }
    } else {
        None
    };

    // Extract D/C mark - can be D, C, RD, or RC
    if pos >= main_line.len() {
        return Err(PaymsgError::ParseError(
            "Field 61: missing debit/credit mark".to_string(),
        ));
    }

    let (dc_mark, is_reversal) = if pos + 2 <= main_line.len()
        && (main_line[pos..pos + 2] == *"RD" || main_line[pos..pos + 2] == *"RC")
    {
        let mark = &main_line[pos..pos + 2];
        pos += 2;
        (if mark == "RC" { "C" } else { "D" }, true)
    } else {
        let mark = &main_line[pos..pos + 1];
        pos += 1;
        (mark, false)
    };

    let credit_debit_indicator = match dc_mark {
        "C" => "CRDT",
        "D" => "DBIT",
        _ => return Err(PaymsgError::ParseError(format!(
            "Invalid D/C mark in field 61: {}", dc_mark
        ))),
    }.to_string();

    // Check for optional funds code (single letter after D/C mark, before amount)
    let _funds_code = if pos < main_line.len()
        && main_line.chars().nth(pos).map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
    {
        let code = &main_line[pos..pos + 1];
        pos += 1;
        Some(code)
    } else {
        None
    };

    // Extract amount - continues until we hit a letter (transaction type)
    let amount_start = pos;
    while pos < main_line.len() {
        let ch = main_line.chars().nth(pos).unwrap();
        if ch.is_ascii_digit() || ch == ',' {
            pos += 1;
        } else {
            break;
        }
    }

    if pos == amount_start {
        return Err(PaymsgError::ParseError(
            "Field 61: missing amount".to_string(),
        ));
    }

    let amount_str = &main_line[amount_start..pos];
    let amount_mx_str = AmountConverter::mt_to_mx(amount_str);
    let amount_decimal = Decimal::from_str(&amount_mx_str).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount format: {} ({})", amount_mx_str, e))
    })?;

    // Extract transaction type (1 letter + 3 alphanumeric)
    let transaction_type = if pos + 4 <= main_line.len() {
        let trans_type = &main_line[pos..pos + 4];
        pos += 4;
        Some(trans_type.to_string())
    } else {
        None
    };

    // Extract customer reference - everything until "//" or end of line
    let remaining = &main_line[pos..];
    let (customer_reference, bank_reference) = if let Some(slash_pos) = remaining.find("//") {
        let cust_ref = remaining[..slash_pos].trim();
        let bank_ref = remaining[slash_pos + 2..].trim();
        (
            if cust_ref.is_empty() { None } else { Some(cust_ref.to_string()) },
            if bank_ref.is_empty() { None } else { Some(bank_ref.to_string()) },
        )
    } else if !remaining.is_empty() {
        (Some(remaining.trim().to_string()), None)
    } else {
        (None, None)
    };

    // Build Entry
    let entry = Entry {
        entry_reference: bank_reference.clone(),
        amount: ActiveOrHistoricCurrencyAndAmount {
            currency: statement_currency.to_string(),
            value: amount_decimal,
        },
        credit_debit_indicator,
        reversal_indicator: if is_reversal { Some(true) } else { None },
        status: EntryStatus {
            code: Some("BOOK".to_string()),
            proprietary: None,
        },
        booking_date: booking_date.map(|d| DateOrDateTime {
            date: Some(d),
            date_time: None,
        }),
        value_date: Some(DateOrDateTime {
            date: Some(value_date),
            date_time: None,
        }),
        account_servicer_reference: None,
        availability: None,
        bank_transaction_code: BankTransactionCode {
            domain: None,
            proprietary: transaction_type.map(|tt| ProprietaryBankTransactionCode {
                code: tt,
                issuer: None,
            }),
        },
        commission_waiver_indicator: None,
        additional_info_indicator: None,
        amount_details: None,
        charges: None,
        technical_input_channel: None,
        interest: None,
        entry_details: if customer_reference.is_some() || bank_reference.is_some() || supplementary_details.is_some() {
            Some(vec![EntryDetails {
                batch: None,
                transaction_details: Some(vec![TransactionDetails {
                    references: Some(TransactionReferences {
                        message_id: None,
                        account_servicer_reference: customer_reference,
                        payment_information_id: None,
                        instruction_id: None,
                        end_to_end_id: None,
                        transaction_id: None,
                        uetr: None,
                        mandate_id: None,
                        cheque_number: None,
                        clearing_system_reference: None,
                        account_owner_transaction_id: None,
                        proprietary: None,
                    }),
                    amount_details: None,
                    availability: None,
                    bank_transaction_code: None,
                    charges: None,
                    interest: None,
                    related_parties: None,
                    related_agents: None,
                    purpose: None,
                    related_remittance_information: None,
                    remittance_information: None,
                    related_dates: None,
                    related_price: None,
                    related_quantities: None,
                    financial_instrument_id: None,
                    tax: None,
                    return_information: None,
                    corporate_action: None,
                    safekeeping_account: None,
                    cash_deposit: None,
                    card_transaction: None,
                    additional_transaction_info: None,
                }]),
            }])
        } else {
            None
        },
        additional_entry_info: supplementary_details,
    };

    Ok(entry)
}

/// Add field 86 information to an entry
fn add_field_86_to_entry(
    mut entry: Entry,
    field_86: &paymsg_mt::MtField,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Entry, PaymsgError> {
    let text = field_86.value.trim();

    // For now, treat field 86 as unstructured remittance info
    // Advanced parsing would extract ?NN codes for structured info

    // If entry_details doesn't exist, create it
    if entry.entry_details.is_none() {
        entry.entry_details = Some(vec![EntryDetails {
            batch: None,
            transaction_details: Some(vec![TransactionDetails {
                references: None,
                amount_details: None,
                availability: None,
                bank_transaction_code: None,
                charges: None,
                interest: None,
                related_parties: None,
                related_agents: None,
                purpose: None,
                related_remittance_information: None,
                remittance_information: Some(RemittanceInformation {
                    unstructured: Some(vec![text.to_string()]),
                    structured: None,
                }),
                related_dates: None,
                related_price: None,
                related_quantities: None,
                financial_instrument_id: None,
                tax: None,
                return_information: None,
                corporate_action: None,
                safekeeping_account: None,
                cash_deposit: None,
                card_transaction: None,
                additional_transaction_info: None,
            }]),
        }]);
    } else {
        // Add to existing entry_details
        if let Some(ref mut entry_details) = entry.entry_details {
            if let Some(ref mut tx_details) = entry_details[0].transaction_details {
                if let Some(ref mut first_tx) = tx_details.first_mut() {
                    first_tx.remittance_information = Some(RemittanceInformation {
                        unstructured: Some(vec![text.to_string()]),
                        structured: None,
                    });
                }
            }
        }
    }

    Ok(entry)
}
