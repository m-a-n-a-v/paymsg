//! Integration tests for MT940 ↔ camt.053 translation

use paymsg_mt::MtMessage;
use paymsg_translate::{translate_mt940_to_camt053, translate_camt053_to_mt940};

#[test]
fn test_mt940_to_camt053_minimal() {
    // Sample MT940 message with minimal fields
    let mt940_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
:20:STMT20231115001
:25:DE89370400440532013000
:28C:235/1
:60F:C231114EUR10000,00
:62F:C231115EUR10000,00
-}"#;

    // Parse MT940
    let mt940 = MtMessage::parse(mt940_text).expect("Failed to parse MT940");

    // Translate to camt.053
    let result = translate_mt940_to_camt053(&mt940).expect("Failed to translate MT940 to camt.053");

    // Check that warnings were generated for missing fields
    assert!(result.has_warnings());

    // Basic assertions
    let doc = result.message;
    assert_eq!(doc.bank_to_customer_statement.statement.len(), 1);

    let stmt = &doc.bank_to_customer_statement.statement[0];
    assert_eq!(stmt.id, "STMT20231115001");
    assert_eq!(stmt.legal_sequence_number, Some(235));
    assert_eq!(stmt.electronic_sequence_number, Some("1".to_string()));

    // Check account
    assert_eq!(stmt.account.id.iban, Some("DE89370400440532013000".to_string()));
    assert_eq!(stmt.account.currency, Some("EUR".to_string()));

    // Check balances
    assert_eq!(stmt.balance.len(), 2); // Opening and closing

    let opening_balance = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("OPBD")
    }).expect("Opening balance not found");
    assert_eq!(opening_balance.credit_debit_indicator, "CRDT");
    assert_eq!(opening_balance.amount.currency, "EUR");
    assert_eq!(opening_balance.amount.value.to_string(), "10000.00");

    let closing_balance = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("CLBD")
    }).expect("Closing balance not found");
    assert_eq!(closing_balance.credit_debit_indicator, "CRDT");
    assert_eq!(closing_balance.amount.currency, "EUR");
    assert_eq!(closing_balance.amount.value.to_string(), "10000.00");
}

#[test]
fn test_camt053_to_mt940_roundtrip() {
    // Sample MT940 message
    let mt940_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
:20:STMT001
:25:DE89370400440532013000
:28C:100
:60F:C231114EUR5000,00
:62F:C231115EUR5500,00
-}"#;

    // Parse MT940
    let mt940_original = MtMessage::parse(mt940_text).expect("Failed to parse MT940");

    // Translate to camt.053
    let camt053_result = translate_mt940_to_camt053(&mt940_original)
        .expect("Failed to translate MT940 to camt.053");
    let camt053 = camt053_result.message;

    // Translate back to MT940
    let mt940_result = translate_camt053_to_mt940(&camt053)
        .expect("Failed to translate camt.053 to MT940");
    let mt940_roundtrip = mt940_result.message;

    // Parse fields from original and roundtrip
    let original_fields = mt940_original.block4.parse_fields().expect("Failed to parse original fields");
    let roundtrip_fields = mt940_roundtrip.block4.parse_fields().expect("Failed to parse roundtrip fields");

    // Build field maps (trim and remove trailing dash)
    let original_map: std::collections::HashMap<_, _> = original_fields
        .iter()
        .map(|f| (f.tag.as_str(), f.value.trim().trim_end_matches('-').trim()))
        .collect();
    let roundtrip_map: std::collections::HashMap<_, _> = roundtrip_fields
        .iter()
        .map(|f| (f.tag.as_str(), f.value.trim().trim_end_matches('-').trim()))
        .collect();

    // Check key fields are preserved
    assert_eq!(original_map.get("20"), roundtrip_map.get("20"));
    assert_eq!(original_map.get("25"), roundtrip_map.get("25"));
    assert_eq!(original_map.get("28C"), roundtrip_map.get("28C"));
    assert_eq!(original_map.get("60F"), roundtrip_map.get("60F"));
    assert_eq!(original_map.get("62F"), roundtrip_map.get("62F"));
}

#[test]
fn test_mt940_with_entries() {
    // MT940 message with statement entries (field 61)
    let mt940_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
:20:STMT20231115002
:25:DE89370400440532013000
:28C:235
:60F:C231114EUR10000,00
:61:2311140214C1500,00NTRFNONREF//BNK001
Payment received
:86:Invoice payment
:62F:C231115EUR11500,00
-}"#;

    // Parse MT940
    let mt940 = MtMessage::parse(mt940_text).expect("Failed to parse MT940");

    // Translate to camt.053
    let result = translate_mt940_to_camt053(&mt940).expect("Failed to translate MT940 to camt.053");

    let stmt = &result.message.bank_to_customer_statement.statement[0];

    // Check that entry was created
    assert!(stmt.entry.is_some());
    let entries = stmt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.credit_debit_indicator, "CRDT");
    assert_eq!(entry.amount.value.to_string(), "1500.00");
    assert_eq!(entry.amount.currency, "EUR");

    // Check value date
    assert!(entry.value_date.is_some());
    let value_date = entry.value_date.as_ref().unwrap();
    assert_eq!(value_date.date, Some("2023-11-14".to_string()));

    // Check booking date
    assert!(entry.booking_date.is_some());
    let booking_date = entry.booking_date.as_ref().unwrap();
    assert_eq!(booking_date.date, Some("2023-02-14".to_string()));

    // Check transaction type
    assert!(entry.bank_transaction_code.proprietary.is_some());
    let tx_code = entry.bank_transaction_code.proprietary.as_ref().unwrap();
    assert_eq!(tx_code.code, "NTRF");

    // Check entry reference
    assert_eq!(entry.entry_reference, Some("BNK001".to_string()));
}

#[test]
fn test_mt940_balance_types() {
    // MT940 with multiple balance types
    let mt940_text = r#"{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
:20:STMT20231115003
:25:DE89370400440532013000
:28C:235
:60F:C231114EUR10000,00
:62F:C231115EUR11000,00
:64:C231115EUR10500,00
:65:C231116EUR12000,00
-}"#;

    // Parse MT940
    let mt940 = MtMessage::parse(mt940_text).expect("Failed to parse MT940");

    // Translate to camt.053
    let result = translate_mt940_to_camt053(&mt940).expect("Failed to translate MT940 to camt.053");

    let stmt = &result.message.bank_to_customer_statement.statement[0];

    // Check all balance types
    assert_eq!(stmt.balance.len(), 4);

    // Opening balance (OPBD)
    let opening = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("OPBD")
    }).expect("Opening balance not found");
    assert_eq!(opening.amount.value.to_string(), "10000.00");

    // Closing balance (CLBD)
    let closing = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("CLBD")
    }).expect("Closing balance not found");
    assert_eq!(closing.amount.value.to_string(), "11000.00");

    // Closing available balance (CLAV)
    let closing_avail = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("CLAV")
    }).expect("Closing available balance not found");
    assert_eq!(closing_avail.amount.value.to_string(), "10500.00");

    // Forward available balance (FWAV)
    let forward_avail = stmt.balance.iter().find(|b| {
        b.balance_type.code_or_proprietary.code.as_ref().map(|c| c.as_str()) == Some("FWAV")
    }).expect("Forward available balance not found");
    assert_eq!(forward_avail.amount.value.to_string(), "12000.00");
}

#[test]
fn test_translation_infrastructure_exists() {
    // Just verify the translation functions exist and are callable
    use paymsg_translate::{translate_mt940_to_camt053, translate_camt053_to_mt940};

    // This test just checks that the functions are available
    let _ = translate_mt940_to_camt053;
    let _ = translate_camt053_to_mt940;
}
