//! Integration tests for MT942 ↔ camt.052 translation

use paymsg_mt::MtMessage;
use paymsg_translate::{translate_mt942_to_camt052, translate_camt052_to_mt942};

#[test]
fn test_mt942_to_camt052_minimal() {
    // Sample MT942 message with minimal fields
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210001
:25:DE89370400440532013000
:28C:3
:13D:260210+1430
:34F:EUR10000,00
:90D:2EUR5000,00
:90C:3EUR7500,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    // Check that warnings were generated for missing fields
    assert!(result.has_warnings());

    // Basic assertions
    let doc = result.message;
    assert_eq!(doc.bank_to_customer_account_report.report.len(), 1);

    let rpt = &doc.bank_to_customer_account_report.report[0];
    assert_eq!(rpt.id, "RPT20260210001");
    assert_eq!(rpt.legal_sequence_number, Some(3));
    assert_eq!(rpt.electronic_sequence_number, None);

    // Check creation date/time from field 13D
    assert_eq!(rpt.creation_date_time, "2026-02-10T14:30:00");

    // Check account
    assert_eq!(
        rpt.account.id.iban,
        Some("DE89370400440532013000".to_string())
    );
    assert_eq!(rpt.account.currency, Some("EUR".to_string()));

    // MT942 has NO balances (unlike MT940)
    assert!(rpt.balance.is_none());

    // Check floor limit in reporting source
    assert!(rpt.reporting_source.is_some());

    // Check transactions summary
    assert!(rpt.transactions_summary.is_some());
    let summary = rpt.transactions_summary.as_ref().unwrap();

    // Check debit totals (field 90D)
    assert!(summary.total_debit_entries.is_some());
    let debit = summary.total_debit_entries.as_ref().unwrap();
    assert_eq!(debit.number_of_entries, Some(2));
    assert_eq!(debit.sum.unwrap().to_string(), "5000.00");

    // Check credit totals (field 90C)
    assert!(summary.total_credit_entries.is_some());
    let credit = summary.total_credit_entries.as_ref().unwrap();
    assert_eq!(credit.number_of_entries, Some(3));
    assert_eq!(credit.sum.unwrap().to_string(), "7500.00");
}

#[test]
fn test_camt052_to_mt942_roundtrip() {
    // Sample MT942 message
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT001
:25:DE89370400440532013000
:28C:5
:13D:260210+1000
:90D:1EUR2000,00
:90C:2EUR5000,00
-}"#;

    // Parse MT942
    let mt942_original = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let camt052_result =
        translate_mt942_to_camt052(&mt942_original).expect("Failed to translate MT942 to camt.052");
    let camt052 = camt052_result.message;

    // Translate back to MT942
    let mt942_result =
        translate_camt052_to_mt942(&camt052).expect("Failed to translate camt.052 to MT942");
    let mt942_roundtrip = mt942_result.message;

    // Parse fields from original and roundtrip
    let original_fields = mt942_original
        .block4
        .parse_fields()
        .expect("Failed to parse original fields");
    let roundtrip_fields = mt942_roundtrip
        .block4
        .parse_fields()
        .expect("Failed to parse roundtrip fields");

    // Build field maps (trim and remove trailing dash)
    let original_map: std::collections::HashMap<_, _> = original_fields
        .iter()
        .map(|f| {
            (
                f.tag.as_str(),
                f.value.trim().trim_end_matches('-').trim(),
            )
        })
        .collect();
    let roundtrip_map: std::collections::HashMap<_, _> = roundtrip_fields
        .iter()
        .map(|f| {
            (
                f.tag.as_str(),
                f.value.trim().trim_end_matches('-').trim(),
            )
        })
        .collect();

    // Check key fields are preserved
    assert_eq!(original_map.get("20"), roundtrip_map.get("20"));
    assert_eq!(original_map.get("25"), roundtrip_map.get("25"));
    assert_eq!(original_map.get("28C"), roundtrip_map.get("28C"));
    assert_eq!(original_map.get("13D"), roundtrip_map.get("13D"));
    assert_eq!(original_map.get("90D"), roundtrip_map.get("90D"));
    assert_eq!(original_map.get("90C"), roundtrip_map.get("90C"));
}

#[test]
fn test_mt942_with_entries() {
    // MT942 message with interim entries (field 61)
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210002
:25:DE89370400440532013000
:28C:4
:13D:260210+1530
:34F:EUR5000,00
:61:2602100211C1500,00NTRFNONREF//BNK001
:86:Wire transfer received
:90D:1EUR2000,00
:90C:2EUR3500,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check that entry was created
    assert!(rpt.entry.is_some());
    let entries = rpt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.credit_debit_indicator.as_str(), "CRDT");
    assert_eq!(entry.amount.value.to_string(), "1500.00");
    assert_eq!(entry.amount.currency, "EUR");

    // Check value date
    assert!(entry.value_date.is_some());
    let value_date = entry.value_date.as_ref().unwrap();
    assert_eq!(value_date.date, Some("2026-02-10".to_string()));

    // Check booking date
    assert!(entry.booking_date.is_some());
    let booking_date = entry.booking_date.as_ref().unwrap();
    assert_eq!(booking_date.date, Some("2026-02-11".to_string()));

    // Check transaction type
    let tx_code = &entry.bank_transaction_code;
    assert!(tx_code.proprietary.is_some());
    assert_eq!(
        tx_code.proprietary.as_ref().unwrap().code,
        "NTRF".to_string()
    );

    // Check entry details with remittance info (from field 86)
    assert!(entry.entry_details.is_some());
}

#[test]
fn test_mt942_floor_limit() {
    // MT942 message with floor limit indicator
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210003
:25:DE89370400440532013000
:28C:1
:13D:260210+0900
:34F:EUR10000,00
:90D:5EUR50000,00
:90C:3EUR75000,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check floor limit in reporting source
    assert!(rpt.reporting_source.is_some());
    let reporting_source = rpt.reporting_source.as_ref().unwrap();
    assert!(reporting_source.proprietary.is_some());

    let prtry = reporting_source.proprietary.as_ref().unwrap();
    assert!(prtry.contains("Floor limit"));
    assert!(prtry.contains("EUR"));
    assert!(prtry.contains("10000"));

    // Check additional report info also contains floor limit context
    assert!(rpt.additional_report_info.is_some());
    let addtl_info = rpt.additional_report_info.as_ref().unwrap();
    assert!(addtl_info.contains("Floor limit"));
}

#[test]
fn test_mt942_field_25p_with_bic() {
    // MT942 message with field 25P (BIC/Account format)
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210004
:25P:DEUTDEFF/DE89370400440532013000
:28C:2
:13D:260210+1200
:90D:0EUR0,00
:90C:1EUR1000,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check account has IBAN
    assert_eq!(
        rpt.account.id.iban,
        Some("DE89370400440532013000".to_string())
    );

    // Check account servicer has BIC
    assert!(rpt.account.servicer.is_some());
    let servicer = rpt.account.servicer.as_ref().unwrap();
    assert_eq!(
        servicer.financial_institution_identification.bic,
        Some("DEUTDEFFXXX".to_string())
    ); // Normalized to BIC11
}

#[test]
fn test_mt942_debit_credit_entries() {
    // MT942 message with both debit and credit entries
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210005
:25:DE89370400440532013000
:28C:1
:13D:260210+1030
:61:260210C2000,00NCHKCUST001
:86:Check deposit
:61:260210D500,00NDDTVENDOR123
:86:Direct debit payment
:90D:1EUR500,00
:90C:1EUR2000,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check that both entries were created
    assert!(rpt.entry.is_some());
    let entries = rpt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 2);

    // Check credit entry
    let credit_entry = entries.iter().find(|e| e.credit_debit_indicator.as_str() == "CRDT").expect("Credit entry not found");
    assert_eq!(credit_entry.amount.value.to_string(), "2000.00");
    assert_eq!(
        credit_entry
            .bank_transaction_code
            .proprietary
            .as_ref()
            .unwrap()
            .code,
        "NCHK".to_string()
    );

    // Check debit entry
    let debit_entry = entries.iter().find(|e| e.credit_debit_indicator.as_str() == "DBIT").expect("Debit entry not found");
    assert_eq!(debit_entry.amount.value.to_string(), "500.00");
    assert_eq!(
        debit_entry
            .bank_transaction_code
            .proprietary
            .as_ref()
            .unwrap()
            .code,
        "NDDT".to_string()
    );
}

#[test]
fn test_mt942_reversal_entries() {
    // MT942 message with reversal entries (RC and RD marks)
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210006
:25:DE89370400440532013000
:28C:1
:13D:260210+1400
:61:260210RC1000,00NTRFREF001
:86:Reversal of credit transfer
:61:260210RD500,00NCHKREF002
:86:Reversal of check payment
:90D:1EUR500,00
:90C:1EUR1000,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check entries
    assert!(rpt.entry.is_some());
    let entries = rpt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 2);

    // Check reversal credit entry
    let rc_entry = entries
        .iter()
        .find(|e| {
            e.credit_debit_indicator.as_str() == "CRDT"
                && e.reversal_indicator.unwrap_or(false)
        })
        .expect("Reversal credit entry not found");
    assert_eq!(rc_entry.amount.value.to_string(), "1000.00");

    // Check reversal debit entry
    let rd_entry = entries
        .iter()
        .find(|e| {
            e.credit_debit_indicator.as_str() == "DBIT"
                && e.reversal_indicator.unwrap_or(false)
        })
        .expect("Reversal debit entry not found");
    assert_eq!(rd_entry.amount.value.to_string(), "500.00");
}

#[test]
fn test_mt942_pending_entries() {
    // MT942 message with pending entry (D funds code)
    let mt942_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I942CORPORATEXXXN}{4:
:20:RPT20260210007
:25:DE89370400440532013000
:28C:1
:13D:260210+1600
:61:260210CD3000,00NTRFPENDING001
:86:Pending wire transfer
:90D:0EUR0,00
:90C:1EUR3000,00
-}"#;

    // Parse MT942
    let mt942 = MtMessage::parse(mt942_text).expect("Failed to parse MT942");

    // Translate to camt.052
    let result = translate_mt942_to_camt052(&mt942).expect("Failed to translate MT942 to camt.052");

    let rpt = &result.message.bank_to_customer_account_report.report[0];

    // Check entry
    assert!(rpt.entry.is_some());
    let entries = rpt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];

    // Check status is PDNG (pending)
    // Status is not optional
    let status = &entry.status;
    assert_eq!(status.code, Some("PDNG".to_string()));
}

#[test]
fn test_translation_infrastructure_exists() {
    // Just verify the translation functions exist and are callable
    use paymsg_translate::{translate_camt052_to_mt942, translate_mt942_to_camt052};

    // This test just checks that the functions are available
    let _ = translate_mt942_to_camt052;
    let _ = translate_camt052_to_mt942;
}
