//! Translate MT942 (SWIFT) to camt.052 (ISO 20022)
//!
//! This module implements translation from SWIFT MT942 (Interim Transaction Report)
//! to ISO 20022 camt.052.001.10 (BankToCustomerAccountReport).
//!
//! Key differences from MT940/camt.053:
//! - MT942 has NO opening/closing balances (60F/60M/62F/62M) - interim focus
//! - MT942 has floor limit indicator (34F) - only transactions above threshold are detailed
//! - MT942 has summary totals (90D/90C) for all debits/credits including below floor limit
//! - MT942 has date/time indication (13D) for precise intraday reporting
//! - Field 61 and 86 structures identical to MT940
//!
//! Key transformations:
//! - Split: field 28C → LglSeqNb + ElctrncSeqNb
//! - Split: field 13D → CreDtTm (date/time to ISO 8601)
//! - Split: field 34F → RptgSrc/Prtry or AddtlRptInf
//! - Split: field 61 → Entry elements
//! - Split: field 90D/90C → TxsSummary totals
//! - Field mapping: field 20 → Rpt/Id
//! - Field mapping: field 25/25P → Rpt/Acct

use crate::types::{
    BicNormalizer, DateConverter, DataLossCategory, DataLossWarning, TranslationResult,
};
use chrono::Utc;
use paymsg_core::{Iban, PaymsgError};
use paymsg_iso20022::camt052::{
    AccountReport, BankToCustomerAccountReport, Document, GroupHeader,
};
use paymsg_iso20022::camt053::{
    AccountIdentification, ActiveOrHistoricCurrencyAndAmount, BankTransactionCode,
    BranchAndFinancialInstitutionIdentification, CashAccount, DateOrDateTime, Entry, EntryDetails,
    EntryStatus, FinancialInstitutionIdentification, GenericAccountIdentification,
    NumberAndSumOfTransactions, ProprietaryBankTransactionCode, RemittanceInformation,
    ReportingSource, TransactionDetails, TransactionReferences, TransactionsSummary,
};
use paymsg_mt::MtMessage;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

/// Translate an MT942 message to camt.052
pub fn translate(mt942: &MtMessage) -> Result<TranslationResult<Document>, PaymsgError> {
    let mut warnings = Vec::new();

    // Build Group Header from message envelope
    let grp_hdr = build_group_header(mt942, &mut warnings)?;

    // Build Account Report from Block 4 fields
    let rpt = build_account_report(mt942, &mut warnings)?;

    let bank_to_customer_account_report = BankToCustomerAccountReport {
        group_header: grp_hdr,
        report: vec![rpt],
        additional_report_info: None,
    };

    let document = Document {
        bank_to_customer_account_report,
    };

    Ok(TranslationResult::with_warnings(document, warnings))
}

/// Build Group Header from MT942 message
fn build_group_header(
    mt942: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<GroupHeader, PaymsgError> {
    // Parse fields from Block 4
    let parsed_fields = mt942.block4.parse_fields()?;
    let mut field_map: HashMap<String, Vec<&paymsg_mt::MtField>> = HashMap::new();
    for field in &parsed_fields {
        field_map.entry(field.tag.clone()).or_default().push(field);
    }

    // Generate message ID (not present in MT942)
    let message_id = format!("MT942-{}", Uuid::new_v4());
    warnings.push(DataLossWarning::new(
        "GrpHdr/MsgId",
        DataLossCategory::NoEquivalent,
        "Generated message ID (not present in MT942)",
    ));

    // Creation date/time - use field 13D if present, otherwise current time
    let creation_date_time = if let Some(field_13d) = field_map.get("13D").and_then(|f| f.first())
    {
        parse_field_13d(&field_13d.value)?
    } else {
        warnings.push(DataLossWarning::new(
            "GrpHdr/CreDtTm",
            DataLossCategory::NoEquivalent,
            "Set to current timestamp (field 13D not present)",
        ));
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    };

    Ok(GroupHeader {
        message_id,
        creation_date_time,
        message_recipient: None,
        message_pagination: None,
        additional_information: None,
    })
}

/// Build Account Report from Block 4 fields
fn build_account_report(
    mt942: &MtMessage,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<AccountReport, PaymsgError> {
    // Parse fields from Block 4
    let parsed_fields = mt942.block4.parse_fields()?;

    // Build field map for easy access
    let mut field_map: HashMap<String, Vec<&paymsg_mt::MtField>> = HashMap::new();
    for field in &parsed_fields {
        field_map.entry(field.tag.clone()).or_default().push(field);
    }

    // Field 20: Report Reference → Rpt/Id
    let report_id = field_map
        .get("20")
        .and_then(|fields| fields.first())
        .map(|f| f.value.trim().to_string())
        .ok_or_else(|| PaymsgError::TranslationError("MT942 field 20 is mandatory".to_string()))?;

    // Truncate to 16 chars if needed (per mapping spec)
    let report_id = if report_id.len() > 16 {
        warnings.push(
            DataLossWarning::new(
                "Rpt/Id",
                DataLossCategory::Truncation,
                format!("Report ID truncated from {} to 16 chars", report_id.len()),
            )
            .with_original_value(&report_id),
        );
        report_id[..16].to_string()
    } else {
        report_id
    };

    // Field 28C: Report Number/Sequence Number
    let (legal_seq_nb, electronic_seq_nb) = extract_field_28c(&field_map, warnings)?;

    // Creation date/time - use field 13D if present
    let creation_date_time = if let Some(field_13d) = field_map.get("13D").and_then(|f| f.first())
    {
        parse_field_13d(&field_13d.value)?
    } else {
        warnings.push(DataLossWarning::new(
            "Rpt/CreDtTm",
            DataLossCategory::NoEquivalent,
            "Set to current timestamp (field 13D not present)",
        ));
        Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    };

    // Field 25/25P: Account Identification
    let account = extract_account(&field_map, warnings)?;

    // Field 34F: Floor Limit Indicator (optional)
    let (reporting_source, floor_limit_info) = extract_floor_limit(&field_map, warnings)?;

    // Get currency from field 34F or 90D/90C
    let currency = extract_currency(&field_map)?;

    // Set account currency
    let mut account = account;
    account.currency = Some(currency.clone());

    // Field 61: Statement Lines → Entries
    let entries = extract_entries(&field_map, &currency, warnings)?;

    // Fields 90D/90C: Summary totals
    let transactions_summary = extract_transactions_summary(&field_map, warnings)?;

    // Combine floor limit info with additional report info
    let additional_report_info = floor_limit_info;

    Ok(AccountReport {
        id: report_id,
        report_pagination: None,
        electronic_sequence_number: electronic_seq_nb,
        legal_sequence_number: legal_seq_nb,
        creation_date_time,
        from_to_date: None,
        copy_duplicate_indicator: None,
        reporting_source,
        account,
        related_account: None,
        interest: None,
        balance: None, // MT942 has no balances
        transactions_summary,
        entry: if entries.is_empty() {
            None
        } else {
            Some(entries)
        },
        additional_report_info,
    })
}

/// Parse field 13D: Date/Time Indication (YYMMDD+HHMM)
fn parse_field_13d(value: &str) -> Result<String, PaymsgError> {
    let value = value.trim();

    // Expected format: YYMMDD+HHMM (13 chars)
    if value.len() < 11 {
        return Err(PaymsgError::ParseError(format!(
            "Invalid field 13D format: {}",
            value
        )));
    }

    // Extract date part (YYMMDD)
    let date_str = &value[..6];
    let date = DateConverter::mt_to_mx(date_str)?;

    // Check for + separator
    if !value.contains('+') {
        return Err(PaymsgError::ParseError(format!(
            "Field 13D missing + separator: {}",
            value
        )));
    }

    // Extract time part (HHMM)
    let time_start = value.find('+').unwrap() + 1;
    let time_str = &value[time_start..];

    if time_str.len() < 4 {
        return Err(PaymsgError::ParseError(format!(
            "Invalid field 13D time format: {}",
            time_str
        )));
    }

    let hour = &time_str[..2]
        .parse::<u32>()
        .map_err(|_| PaymsgError::ParseError(format!("Invalid hour in field 13D: {}", time_str)))?;
    let minute = &time_str[2..4]
        .parse::<u32>()
        .map_err(|_| {
            PaymsgError::ParseError(format!("Invalid minute in field 13D: {}", time_str))
        })?;

    // Create ISO 8601 date-time string
    Ok(format!("{}T{:02}:{:02}:00", date, hour, minute))
}

/// Extract field 28C: Report Number/Sequence Number
fn extract_field_28c(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<(Option<u64>, Option<String>), PaymsgError> {
    let field_28c = field_map.get("28C").and_then(|fields| fields.first());

    if let Some(field) = field_28c {
        let value = field.value.trim();

        // Parse field 28C: format is ReportNumber[/SequenceNumber]
        if let Some(slash_pos) = value.find('/') {
            let report_num_str = &value[..slash_pos];
            let seq_num_str = &value[slash_pos + 1..];

            let legal_seq_nb = report_num_str.parse::<u64>().ok();
            let electronic_seq_nb = Some(seq_num_str.to_string());

            Ok((legal_seq_nb, electronic_seq_nb))
        } else {
            // No sequence number, just report number
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
        "MT942 must have field 25 or 25P for account identification".to_string(),
    ))
}

/// Extract floor limit from field 34F
fn extract_floor_limit(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<(Option<ReportingSource>, Option<String>), PaymsgError> {
    if let Some(field_34f) = field_map.get("34F").and_then(|fields| fields.first()) {
        let value = field_34f.value.trim();

        // Parse field 34F: Currency(3) + Amount(15d)
        if value.len() >= 3 {
            let currency = &value[..3];
            let amount_str = &value[3..];

            // Convert comma to period for decimal
            let amount_normalized = amount_str.replace(',', ".");

            // Format floor limit as free text
            let floor_limit_text = format!(
                "Floor limit: {} {} - transactions below this threshold are included in totals but not individually listed",
                currency, amount_normalized
            );

            warnings.push(DataLossWarning::new(
                "Rpt/RptgSrc/Prtry",
                DataLossCategory::NoEquivalent,
                "MT942 field 34F (floor limit) has no direct camt.052 equivalent - included in AddtlRptInf",
            ));

            // Create reporting source with proprietary info
            let reporting_source = Some(ReportingSource {
                code: None,
                proprietary: Some(format!("Floor limit: {} {}", currency, amount_normalized)),
            });

            Ok((reporting_source, Some(floor_limit_text)))
        } else {
            Err(PaymsgError::ParseError(format!(
                "Invalid field 34F format: {}",
                value
            )))
        }
    } else {
        Ok((None, None))
    }
}

/// Extract currency from field 34F or 90D/90C
fn extract_currency(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
) -> Result<String, PaymsgError> {
    // Try field 34F first (floor limit)
    if let Some(field_34f) = field_map.get("34F").and_then(|fields| fields.first()) {
        let value = field_34f.value.trim();
        if value.len() >= 3 {
            return Ok(value[..3].to_string());
        }
    }

    // Try field 90D or 90C (summary totals)
    if let Some(field_90d) = field_map.get("90D").and_then(|fields| fields.first()) {
        if let Some(currency) = extract_currency_from_90_field(&field_90d.value) {
            return Ok(currency);
        }
    }

    if let Some(field_90c) = field_map.get("90C").and_then(|fields| fields.first()) {
        if let Some(currency) = extract_currency_from_90_field(&field_90c.value) {
            return Ok(currency);
        }
    }

    Err(PaymsgError::TranslationError(
        "Cannot determine currency from MT942 message (no field 34F, 90D, or 90C)".to_string(),
    ))
}

/// Extract currency from field 90D or 90C
/// Format: NumberOfEntries(5n) + Currency(3a) + TotalAmount(15d)
fn extract_currency_from_90_field(value: &str) -> Option<String> {
    let value = value.trim();

    // Find the currency (3 letters) after the number of entries
    // Look for the first sequence of 3 consecutive letters
    let chars: Vec<char> = value.chars().collect();
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i].is_alphabetic() && chars[i + 1].is_alphabetic() && chars[i + 2].is_alphabetic()
        {
            let currency: String = chars[i..i + 3].iter().collect();
            return Some(currency);
        }
    }

    None
}

/// Extract entries from field 61
fn extract_entries(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    currency: &str,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Vec<Entry>, PaymsgError> {
    let mut entries = Vec::new();

    // Get all field 61 entries
    let empty_vec = Vec::new();
    let field_61_entries = field_map.get("61").unwrap_or(&empty_vec);

    // Get all field 86 entries (information to account owner)
    let empty_vec_86 = Vec::new();
    let field_86_entries = field_map.get("86").unwrap_or(&empty_vec_86);

    for (idx, field_61) in field_61_entries.iter().enumerate() {
        // Parse field 61
        let entry = parse_field_61(field_61, currency, warnings)?;

        // Add field 86 info if present for this entry
        let entry = if idx < field_86_entries.len() {
            add_field_86_info(entry, field_86_entries[idx], warnings)?
        } else {
            entry
        };

        entries.push(entry);
    }

    Ok(entries)
}

/// Parse field 61: Statement Line
/// Format: ValueDate[EntryDate]DCMark[FundsCode]Amount[TransactionType][CustomerRef][//BankRef][SupplementaryDetails]
fn parse_field_61(
    field: &paymsg_mt::MtField,
    currency: &str,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Entry, PaymsgError> {
    let value = field.value.trim();

    // Parse value date (YYMMDD, positions 0-5)
    if value.len() < 6 {
        return Err(PaymsgError::ParseError(format!(
            "Field 61 too short: {}",
            value
        )));
    }
    let value_date_str = &value[..6];
    let value_date = DateConverter::mt_to_mx(value_date_str)?;

    let mut pos = 6;

    // Parse optional entry date (MMDD)
    let booking_date = if pos + 4 <= value.len()
        && value[pos..pos + 2].chars().all(|c| c.is_ascii_digit())
        && value[pos + 2..pos + 4].chars().all(|c| c.is_ascii_digit())
    {
        let mmdd = &value[pos..pos + 4];
        let mm = &mmdd[..2];
        let dd = &mmdd[2..4];

        // Extract year from value date
        let value_date_year = value_date[..4].parse::<i32>().unwrap_or(2026);

        pos += 4;

        format!("{}-{}-{}", value_date_year, mm, dd)
    } else {
        value_date.clone()
    };

    // Parse D/C mark (C, D, RC, RD)
    if pos >= value.len() {
        return Err(PaymsgError::ParseError(format!(
            "Field 61 missing D/C mark: {}",
            value
        )));
    }

    let dc_mark = if pos + 1 < value.len()
        && (value[pos..pos + 2] == *"RC" || value[pos..pos + 2] == *"RD")
    {
        let mark = &value[pos..pos + 2];
        pos += 2;
        mark
    } else {
        let mark = &value[pos..pos + 1];
        pos += 1;
        mark
    };

    // Determine credit/debit indicator and reversal
    let (cdt_dbt_ind, is_reversal) = match dc_mark {
        "C" => ("CRDT", false),
        "D" => ("DBIT", false),
        "RC" => ("CRDT", true),
        "RD" => ("DBIT", true),
        _ => {
            return Err(PaymsgError::ParseError(format!(
                "Invalid D/C mark in field 61: {}",
                dc_mark
            )))
        }
    };

    // Parse optional funds code (single char: D for pending)
    let entry_status = if pos < value.len() && value.chars().nth(pos).unwrap() == 'D' {
        pos += 1;
        EntryStatus {
            code: Some("PDNG".to_string()),
            proprietary: None,
        }
    } else {
        EntryStatus {
            code: Some("BOOK".to_string()),
            proprietary: None,
        }
    };

    // Parse amount (up to comma or first alpha char)
    let amount_start = pos;
    let mut amount_end = pos;
    while amount_end < value.len() {
        let ch = value.chars().nth(amount_end).unwrap();
        if ch == ',' || (!ch.is_ascii_digit() && amount_end > amount_start) {
            break;
        }
        amount_end += 1;
    }

    // Check if we found a comma (decimal separator)
    if amount_end < value.len() && value.chars().nth(amount_end).unwrap() == ',' {
        amount_end += 1;
        // Include decimal digits
        while amount_end < value.len() && value.chars().nth(amount_end).unwrap().is_ascii_digit() {
            amount_end += 1;
        }
    }

    let amount_str = &value[amount_start..amount_end];
    let amount_normalized = amount_str.replace(',', ".");
    let amount = Decimal::from_str(&amount_normalized).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount in field 61: {} ({})", amount_str, e))
    })?;

    pos = amount_end;

    // Parse transaction type (4 chars, optional)
    let transaction_type = if pos + 4 <= value.len() {
        let tx_type = &value[pos..pos + 4];
        if tx_type.chars().all(|c| c.is_alphanumeric()) {
            pos += 4;
            Some(tx_type.to_string())
        } else {
            None
        }
    } else {
        None
    };

    // Parse customer reference (up to //, max 16 chars)
    let customer_ref = if pos < value.len() {
        if let Some(bank_ref_pos) = value[pos..].find("//") {
            let ref_str = &value[pos..pos + bank_ref_pos];
            pos += bank_ref_pos;
            Some(ref_str.to_string())
        } else {
            let ref_str = &value[pos..];
            pos = value.len();
            Some(ref_str.to_string())
        }
    } else {
        None
    };

    // Parse bank reference (after //)
    let bank_ref = if pos + 2 <= value.len() && &value[pos..pos + 2] == "//" {
        pos += 2;
        Some(value[pos..].to_string())
    } else {
        None
    };

    // Build entry
    let entry = Entry {
        entry_reference: None,
        amount: ActiveOrHistoricCurrencyAndAmount {
            value: amount,
            currency: currency.to_string(),
        },
        credit_debit_indicator: cdt_dbt_ind.to_string(),
        reversal_indicator: if is_reversal { Some(true) } else { None },
        status: entry_status,
        availability: None,
        booking_date: Some(DateOrDateTime {
            date: Some(booking_date),
            date_time: None,
        }),
        value_date: Some(DateOrDateTime {
            date: Some(value_date),
            date_time: None,
        }),
        account_servicer_reference: bank_ref.clone(),
        bank_transaction_code: transaction_type
            .as_ref()
            .map(|tt| BankTransactionCode {
                domain: None,
                proprietary: Some(ProprietaryBankTransactionCode {
                    code: tt.clone(),
                    issuer: None,
                }),
            })
            .unwrap_or(BankTransactionCode {
                domain: None,
                proprietary: None,
            }),
        commission_waiver_indicator: None,
        additional_info_indicator: None,
        amount_details: None,
        charges: None,
        technical_input_channel: None,
        interest: None,
        entry_details: if customer_ref.is_some() {
            Some(vec![EntryDetails {
                batch: None,
                transaction_details: Some(vec![TransactionDetails {
                    references: Some(TransactionReferences {
                        message_id: None,
                        account_servicer_reference: customer_ref,
                        payment_information_id: None,
                        instruction_id: None,
                        end_to_end_id: None,
                        transaction_id: None,
                        account_owner_transaction_id: None,
                        mandate_id: None,
                        cheque_number: None,
                        clearing_system_reference: None,
                        uetr: None,
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
        additional_entry_info: None,
    };

    Ok(entry)
}

/// Add field 86 information to entry
fn add_field_86_info(
    mut entry: Entry,
    field_86: &paymsg_mt::MtField,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<Entry, PaymsgError> {
    let info = field_86.value.trim();

    // Add to entry details remittance information
    if let Some(ref mut entry_details) = entry.entry_details {
        if let Some(ref mut tx_details) = entry_details[0].transaction_details {
            if let Some(ref mut tx) = tx_details.get_mut(0) {
                tx.remittance_information = Some(RemittanceInformation {
                    unstructured: Some(vec![info.to_string()]),
                    structured: None,
                });
            }
        }
    } else {
        // Create entry details with remittance info
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
                    unstructured: Some(vec![info.to_string()]),
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
    }

    Ok(entry)
}

/// Extract transactions summary from fields 90D and 90C
fn extract_transactions_summary(
    field_map: &HashMap<String, Vec<&paymsg_mt::MtField>>,
    warnings: &mut Vec<DataLossWarning>,
) -> Result<Option<TransactionsSummary>, PaymsgError> {
    let field_90d = field_map.get("90D").and_then(|fields| fields.first());
    let field_90c = field_map.get("90C").and_then(|fields| fields.first());

    if field_90d.is_none() && field_90c.is_none() {
        return Ok(None);
    }

    // Parse field 90D: Number and Sum of Debit Entries
    let total_debit_entries = if let Some(field) = field_90d {
        Some(parse_field_90(&field.value, warnings)?)
    } else {
        None
    };

    // Parse field 90C: Number and Sum of Credit Entries
    let total_credit_entries = if let Some(field) = field_90c {
        Some(parse_field_90(&field.value, warnings)?)
    } else {
        None
    };

    Ok(Some(TransactionsSummary {
        total_entries: None,
        total_credit_entries,
        total_debit_entries,
    }))
}

/// Parse field 90D or 90C
/// Format: NumberOfEntries(5n) + Currency(3a) + TotalAmount(15d)
fn parse_field_90(
    value: &str,
    _warnings: &mut Vec<DataLossWarning>,
) -> Result<NumberAndSumOfTransactions, PaymsgError> {
    let value = value.trim();

    // Extract number of entries (first 1-5 digits)
    let mut num_end = 0;
    while num_end < value.len() && value.chars().nth(num_end).unwrap().is_ascii_digit() {
        num_end += 1;
        if num_end >= 5 {
            break;
        }
    }

    let num_str = &value[..num_end];
    let number_of_entries = num_str.parse::<u64>().map_err(|e| {
        PaymsgError::ParseError(format!("Invalid number of entries in field 90: {} ({})", num_str, e))
    })?;

    // Extract currency (3 letters) - not used but part of format
    if value.len() < num_end + 3 {
        return Err(PaymsgError::ParseError(format!(
            "Field 90 too short for currency: {}",
            value
        )));
    }
    let _currency = &value[num_end..num_end + 3];

    // Extract amount (remaining chars)
    let amount_str = &value[num_end + 3..];
    let amount_normalized = amount_str.replace(',', ".");
    let amount = Decimal::from_str(&amount_normalized).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount in field 90: {} ({})", amount_str, e))
    })?;

    Ok(NumberAndSumOfTransactions {
        number_of_entries: Some(number_of_entries),
        sum: Some(amount),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_13d() {
        let result = parse_field_13d("260210+1430").unwrap();
        assert_eq!(result, "2026-02-10T14:30:00");

        let result = parse_field_13d("991231+2359").unwrap();
        assert_eq!(result, "1999-12-31T23:59:00");
    }

    #[test]
    fn test_extract_currency_from_90_field() {
        assert_eq!(
            extract_currency_from_90_field("5EUR12345,67"),
            Some("EUR".to_string())
        );
        assert_eq!(
            extract_currency_from_90_field("10USD25000,00"),
            Some("USD".to_string())
        );
    }

    #[test]
    fn test_parse_field_90() {
        let result = parse_field_90("5EUR12345,67", &mut Vec::new()).unwrap();
        assert_eq!(result.number_of_entries, Some(5));
        assert_eq!(result.sum.unwrap(), Decimal::from_str("12345.67").unwrap());
    }
}
