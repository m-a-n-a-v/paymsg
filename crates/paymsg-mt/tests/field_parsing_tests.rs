//! Integration tests for MT field parsing with real test data.

use paymsg_mt::{parse_block4_fields, MtMessage};
use std::path::PathBuf;

/// Get the path to the paymsg-specs directory.
fn get_specs_dir() -> PathBuf {
    // During tests, CARGO_MANIFEST_DIR points to crates/paymsg-mt
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let crate_dir = PathBuf::from(manifest_dir);

    // Navigate: crates/paymsg-mt -> crates -> workspace root -> ../paymsg-specs
    crate_dir
        .parent() // crates/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .parent() // parent of workspace
        .unwrap()
        .join("paymsg-specs")
}

#[test]
fn test_parse_mt103_minimal_valid() {
    let specs_dir = get_specs_dir();
    let mt_file = specs_dir.join("testdata/mt/mt103/minimal_valid.mt");

    let content = std::fs::read_to_string(&mt_file)
        .expect("Failed to read minimal_valid.mt");

    let msg = MtMessage::parse(&content).expect("Failed to parse MT103");

    // Parse fields from block 4
    let fields = msg.block4.parse_fields().expect("Failed to parse fields");

    // Should have at least the mandatory fields: 20, 23B, 32A, 50K, 59, 71A
    assert!(fields.len() >= 5, "Expected at least 5 fields, got {}", fields.len());

    // Check for field 20 (Transaction Reference)
    let field_20 = fields.iter().find(|f| f.tag == "20");
    assert!(field_20.is_some(), "Field 20 (Transaction Reference) is mandatory");

    // Check for field 32A (Value Date/Currency/Amount)
    let field_32a = fields.iter().find(|f| f.tag == "32A");
    assert!(field_32a.is_some(), "Field 32A is mandatory");

    if let Some(f) = field_32a {
        // Should have parsed subfields
        assert!(f.subfields.contains_key("date"), "Field 32A should have date subfield");
        assert!(f.subfields.contains_key("currency"), "Field 32A should have currency subfield");
        assert!(f.subfields.contains_key("amount"), "Field 32A should have amount subfield");

        // Validate currency is 3 letters
        let currency = f.subfields.get("currency").unwrap();
        assert_eq!(currency.len(), 3, "Currency should be 3 characters");
    }
}

#[test]
fn test_parse_mt103_full_valid() {
    let specs_dir = get_specs_dir();
    let mt_file = specs_dir.join("testdata/mt/mt103/full_valid.mt");

    let content = std::fs::read_to_string(&mt_file)
        .expect("Failed to read full_valid.mt");

    let msg = MtMessage::parse(&content).expect("Failed to parse MT103");

    // Parse fields
    let fields = msg.block4.parse_fields().expect("Failed to parse fields");

    println!("Parsed {} fields from full_valid.mt", fields.len());

    // Check field 20
    let field_20 = fields.iter().find(|f| f.tag == "20").unwrap();
    assert_eq!(field_20.value, "FULLREF98765");

    // Check field 32A with subfields
    let field_32a = fields.iter().find(|f| f.tag == "32A").unwrap();
    assert_eq!(field_32a.subfields.get("date").unwrap(), "260215");
    assert_eq!(field_32a.subfields.get("currency").unwrap(), "EUR");
    assert_eq!(field_32a.subfields.get("amount").unwrap(), "10000,00");

    // Check field 33B (instructed amount)
    let field_33b = fields.iter().find(|f| f.tag == "33B").unwrap();
    assert_eq!(field_33b.subfields.get("currency").unwrap(), "USD");
    assert_eq!(field_33b.subfields.get("amount").unwrap(), "11500,00");

    // Check field 36 (exchange rate)
    let field_36 = fields.iter().find(|f| f.tag == "36").unwrap();
    assert_eq!(field_36.value, "1,15");

    // Check field 50K (Ordering Customer)
    let field_50k = fields.iter().find(|f| f.tag == "50K").unwrap();
    assert_eq!(field_50k.subfields.get("account").unwrap(), "/DE89370400440532013000");
    assert!(field_50k.subfields.get("name_address").unwrap().contains("MUELLER INTERNATIONAL GMBH"));

    // Check field 59 (Beneficiary)
    let field_59 = fields.iter().find(|f| f.tag == "59").unwrap();
    assert_eq!(field_59.subfields.get("account").unwrap(), "/FR1420041010050500013M02606");
    assert!(field_59.subfields.get("name_address").unwrap().contains("DUPONT TRADING SARL"));

    // Check field 71A (charge bearer)
    let field_71a = fields.iter().find(|f| f.tag == "71A").unwrap();
    assert_eq!(field_71a.value, "SHA");

    // Check field 52A (Ordering Institution)
    let field_52a = fields.iter().find(|f| f.tag == "52A").unwrap();
    assert_eq!(field_52a.subfields.get("account").unwrap(), "/12345678");
    assert_eq!(field_52a.subfields.get("bic").unwrap(), "DEUTDEFFXXX");

    // Check field 57A (Account with Institution)
    let field_57a = fields.iter().find(|f| f.tag == "57A").unwrap();
    assert_eq!(field_57a.subfields.get("account").unwrap(), "/987654321");
    assert_eq!(field_57a.subfields.get("bic").unwrap(), "BNPAFRPPXXX");
}

#[test]
fn test_parse_mt202_full_valid() {
    let specs_dir = get_specs_dir();
    let mt_file = specs_dir.join("testdata/mt/mt202/full_valid.mt");

    let content = std::fs::read_to_string(&mt_file)
        .expect("Failed to read MT202 full_valid.mt");

    let msg = MtMessage::parse(&content).expect("Failed to parse MT202");

    assert_eq!(msg.block2.message_type, "202");

    // Parse fields
    let fields = msg.block4.parse_fields().expect("Failed to parse fields");

    println!("Parsed {} fields from MT202 full_valid.mt", fields.len());

    // Check field 20 (Transaction Reference)
    let field_20 = fields.iter().find(|f| f.tag == "20").unwrap();
    assert_eq!(field_20.value, "FI20231115002");

    // Check field 21 (Related Reference)
    let field_21 = fields.iter().find(|f| f.tag == "21").unwrap();
    assert_eq!(field_21.value, "MT103REF789012");

    // Check field 32A
    let field_32a = fields.iter().find(|f| f.tag == "32A").unwrap();
    assert_eq!(field_32a.subfields.get("date").unwrap(), "231115");
    assert_eq!(field_32a.subfields.get("currency").unwrap(), "USD");
    assert_eq!(field_32a.subfields.get("amount").unwrap(), "125000,00");

    // Check field 58A (Beneficiary Institution)
    let field_58a = fields.iter().find(|f| f.tag == "58A").unwrap();
    assert_eq!(field_58a.subfields.get("account").unwrap(), "/1234567890");
    assert_eq!(field_58a.subfields.get("bic").unwrap(), "BNPAFRPP");
}

#[test]
fn test_parse_mt103_with_repeating_fields() {
    // Test case with repeating field 23E
    let content = "{1:F01DEUTDEFFAXXX0000000000}{2:I103BNPAFRPPXXXXN}{4:
:20:REF123
:23B:CRED
:23E:CHQB/CHECK123
:23E:HOLD
:23E:INTC
:32A:260210EUR5000,00
:50K:/ACCT123
TEST CUSTOMER
:59:/BENEFACCT
BENEFICIARY NAME
:71A:SHA
-}";

    let msg = MtMessage::parse(content).expect("Failed to parse MT103");
    let fields = msg.block4.parse_fields().expect("Failed to parse fields");

    // Count field 23E occurrences
    let field_23e_count = fields.iter().filter(|f| f.tag == "23E").count();
    assert_eq!(field_23e_count, 3, "Should have 3 field 23E entries");

    // Check each 23E value
    let field_23e_values: Vec<&str> = fields
        .iter()
        .filter(|f| f.tag == "23E")
        .map(|f| f.value.as_str())
        .collect();

    assert_eq!(field_23e_values[0], "CHQB/CHECK123");
    assert_eq!(field_23e_values[1], "HOLD");
    assert_eq!(field_23e_values[2], "INTC");
}

#[test]
fn test_parse_field_70_remittance_info() {
    let content = ":20:REF123
:32A:260210EUR1000,00
:70:/INV/2026-02-001
INVOICE 2026-02-001 DATED 2026-02-01
CONSULTING SERVICES JANUARY 2026
:71A:SHA";

    let fields = parse_block4_fields(content).expect("Failed to parse fields");

    let field_70 = fields.iter().find(|f| f.tag == "70").unwrap();
    assert!(field_70.value.contains("INV/2026-02-001"));
    assert!(field_70.value.contains("CONSULTING SERVICES"));
    assert_eq!(field_70.subfields.get("text").unwrap(), &field_70.value);
}

#[test]
fn test_parse_field_72_sender_to_receiver_info() {
    let content = ":20:REF123
:32A:260210EUR1000,00
:72:/ACC/BENEFICIARY ACCOUNT INFO
/INS/URGENT PAYMENT
:71A:SHA";

    let fields = parse_block4_fields(content).expect("Failed to parse fields");

    let field_72 = fields.iter().find(|f| f.tag == "72").unwrap();
    assert!(field_72.value.contains("/ACC/BENEFICIARY ACCOUNT INFO"));
    assert!(field_72.value.contains("/INS/URGENT PAYMENT"));
}

#[test]
fn test_field_base_tag_and_option() {
    let content = ":50K:TEST\n:50A:TEST2\n:20:REF";

    let fields = parse_block4_fields(content).expect("Failed to parse fields");

    let field_50k = fields.iter().find(|f| f.tag == "50K").unwrap();
    assert_eq!(field_50k.base_tag(), "50");
    assert_eq!(field_50k.option(), Some('K'));

    let field_50a = fields.iter().find(|f| f.tag == "50A").unwrap();
    assert_eq!(field_50a.base_tag(), "50");
    assert_eq!(field_50a.option(), Some('A'));

    let field_20 = fields.iter().find(|f| f.tag == "20").unwrap();
    assert_eq!(field_20.base_tag(), "20");
    assert_eq!(field_20.option(), None);
}

#[test]
fn test_load_mt_spec() {
    let specs_dir = get_specs_dir();
    let mt103_spec = specs_dir.join("mt-specs/mt103.json");

    let spec = paymsg_mt::load_mt_spec(&mt103_spec).expect("Failed to load MT103 spec");

    assert_eq!(spec.message_type, "MT103");
    assert_eq!(spec.name, "Single Customer Credit Transfer");
    assert!(!spec.fields.is_empty(), "MT103 spec should have fields");

    // Check for mandatory field 20
    let field_20 = spec.fields.iter().find(|f| f.tag == "20");
    assert!(field_20.is_some(), "Field 20 should be in spec");
    assert_eq!(field_20.unwrap().status, "M", "Field 20 should be mandatory");

    // Check for field 32A with subfields
    let field_32a = spec.fields.iter().find(|f| f.tag == "32A");
    assert!(field_32a.is_some(), "Field 32A should be in spec");
    let f32a = field_32a.unwrap();
    assert_eq!(f32a.status, "M", "Field 32A should be mandatory");
    assert!(f32a.subfields.is_some(), "Field 32A should have subfields");

    let subfields = f32a.subfields.as_ref().unwrap();
    assert_eq!(subfields.len(), 3, "Field 32A should have 3 subfields");

    // Verify subfield names
    let subfield_names: Vec<&str> = subfields.iter().map(|s| s.name.as_str()).collect();
    assert!(subfield_names.contains(&"Value Date"));
    assert!(subfield_names.contains(&"Currency Code"));
    assert!(subfield_names.contains(&"Amount"));
}

#[test]
fn test_load_all_mt_specs() {
    let specs_dir = get_specs_dir();

    let specs = paymsg_mt::load_all_mt_specs(&specs_dir)
        .expect("Failed to load MT specs");

    assert!(specs.contains_key("MT103"), "Should load MT103 spec");
    assert!(specs.contains_key("MT202"), "Should load MT202 spec");
    assert!(specs.contains_key("MT940"), "Should load MT940 spec");
    assert!(specs.contains_key("MT942"), "Should load MT942 spec");

    assert_eq!(specs.len(), 4, "Should load exactly 4 MT specs");
}
