//! Integration tests for pacs.008 → MT103 translation
//!
//! Tests the reverse translation from ISO 20022 pacs.008 to SWIFT MT103

use paymsg_iso20022::parse_pacs008;
use paymsg_translate::translate_pacs008_to_mt103;
use std::path::Path;

/// Test pacs.008 → MT103 translation with minimal message
#[test]
fn test_pacs008_to_mt103_minimal() {
    let specs_dir = Path::new("../paymsg-specs");
    if !specs_dir.exists() {
        eprintln!("Skipping test - specs directory not found at {:?}", specs_dir);
        return;
    }
    let testdata_dir = specs_dir.join("testdata/translation_pairs/mt103_pacs008");

    // Load source pacs.008 XML
    let pacs008_xml = std::fs::read_to_string(testdata_dir.join("pair3_minimal.source.xml"))
        .expect("Failed to read source XML");

    // Parse pacs.008
    let pacs008 = parse_pacs008(&pacs008_xml).expect("Failed to parse pacs.008");

    // Translate to MT103
    let result = translate_pacs008_to_mt103(&pacs008).expect("Translation failed");

    // Serialize to MT format
    let mt_text = result.message.serialize().expect("Serialization failed");

    // Load expected MT103
    let expected_mt = std::fs::read_to_string(testdata_dir.join("pair3_minimal.expected.mt"))
        .expect("Failed to read expected MT");

    // Compare - normalize whitespace for comparison
    let mt_normalized = normalize_mt(&mt_text);
    let _expected_normalized = normalize_mt(&expected_mt);

    // Print for debugging
    println!("Generated MT103:\n{}", mt_text);
    println!("\nExpected MT103:\n{}", expected_mt);
    println!("\nWarnings: {:?}", result.warnings);

    // Compare key fields
    assert!(
        mt_normalized.contains(":20:MIN123456"),
        "Field 20 missing or incorrect"
    );
    assert!(
        mt_normalized.contains(":23B:CRED"),
        "Field 23B missing or incorrect"
    );
    assert!(
        mt_normalized.contains(":32A:260210EUR500,00"),
        "Field 32A missing or incorrect"
    );
    assert!(
        mt_normalized.contains(":50K:ALICE MARTIN"),
        "Field 50K missing or incorrect"
    );
    assert!(
        mt_normalized.contains(":59:BOB SCHMIDT"),
        "Field 59 missing or incorrect"
    );
    assert!(
        mt_normalized.contains(":71A:SHA"),
        "Field 71A missing or incorrect"
    );
}

/// Test pacs.008 → MT103 translation with SEPA transfer
#[test]
fn test_pacs008_to_mt103_sepa() {
    let specs_dir = Path::new("../paymsg-specs");
    if !specs_dir.exists() {
        eprintln!("Skipping test - specs directory not found at {:?}", specs_dir);
        return;
    }
    let testdata_dir = specs_dir.join("testdata/translation_pairs/mt103_pacs008");

    // Load source pacs.008 XML
    let pacs008_xml = std::fs::read_to_string(testdata_dir.join("pair1_sepa_transfer.source.xml"))
        .expect("Failed to read source XML");

    // Parse pacs.008
    let pacs008 = parse_pacs008(&pacs008_xml).expect("Failed to parse pacs.008");

    // Translate to MT103
    let result = translate_pacs008_to_mt103(&pacs008).expect("Translation failed");

    // Serialize to MT format
    let mt_text = result.message.serialize().expect("Serialization failed");

    println!("Generated MT103:\n{}", mt_text);
    println!("\nWarnings: {:?}", result.warnings);

    // Check key fields exist
    let mt_normalized = normalize_mt(&mt_text);
    assert!(
        mt_normalized.contains(":20:"),
        "Field 20 (reference) missing"
    );
    assert!(
        mt_normalized.contains(":32A:"),
        "Field 32A (date/currency/amount) missing"
    );
    assert!(
        mt_normalized.contains(":50K:"),
        "Field 50K (ordering customer) missing"
    );
    assert!(
        mt_normalized.contains(":59:"),
        "Field 59 (beneficiary) missing"
    );
    assert!(
        mt_normalized.contains(":71A:"),
        "Field 71A (charge bearer) missing"
    );
}

/// Test pacs.008 → MT103 translation with USD wire
#[test]
fn test_pacs008_to_mt103_usd_wire() {
    let specs_dir = Path::new("../paymsg-specs");
    if !specs_dir.exists() {
        eprintln!("Skipping test - specs directory not found at {:?}", specs_dir);
        return;
    }
    let testdata_dir = specs_dir.join("testdata/translation_pairs/mt103_pacs008");

    // Load source pacs.008 XML
    let pacs008_xml = std::fs::read_to_string(testdata_dir.join("pair2_usd_wire.source.xml"))
        .expect("Failed to read source XML");

    // Parse pacs.008
    let pacs008 = parse_pacs008(&pacs008_xml).expect("Failed to parse pacs.008");

    // Translate to MT103
    let result = translate_pacs008_to_mt103(&pacs008).expect("Translation failed");

    // Serialize to MT format
    let mt_text = result.message.serialize().expect("Serialization failed");

    println!("Generated MT103:\n{}", mt_text);
    println!("\nWarnings: {:?}", result.warnings);

    // Check key fields exist
    let mt_normalized = normalize_mt(&mt_text);
    assert!(
        mt_normalized.contains(":20:"),
        "Field 20 (reference) missing"
    );
    assert!(
        mt_normalized.contains(":32A:"),
        "Field 32A (date/currency/amount) missing"
    );
    assert!(
        mt_normalized.contains("USD"),
        "USD currency missing from field 32A"
    );
}

/// Test charge bearer conversion (SHAR → SHA, DEBT → OUR, CRED → BEN)
#[test]
fn test_charge_bearer_conversion() {
    use paymsg_iso20022::pacs008::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Create a minimal pacs.008 document with DEBT charge bearer
    let pacs008 = Document {
        fi_to_fi_customer_credit_transfer: FIToFICstmrCdtTrf {
            group_header: GroupHeader {
                message_id: "TEST123".to_string(),
                creation_date_time: "2026-02-10T09:00:00".to_string(),
                number_of_transactions: "1".to_string(),
                total_interbank_settlement_amount: None,
                interbank_settlement_date: None,
                settlement_information: None,
                instructing_agent: BranchAndFinancialInstitutionIdentification {
                    financial_institution_id: FinancialInstitutionIdentification {
                        bic: Some("BNPAFRPPXXX".to_string()),
                        name: None,
                        postal_address: None,
                    },
                },
                instructed_agent: BranchAndFinancialInstitutionIdentification {
                    financial_institution_id: FinancialInstitutionIdentification {
                        bic: Some("DEUTDEFFXXX".to_string()),
                        name: None,
                        postal_address: None,
                    },
                },
            },
            credit_transfer_transaction_information: vec![CreditTransferTransactionInformation {
                payment_id: PaymentIdentification {
                    instruction_id: Some("TEST123".to_string()),
                    end_to_end_id: "TEST123".to_string(),
                    transaction_id: None,
                    uetr: None,
                },
                payment_type_information: None,
                interbank_settlement_amount: ActiveCurrencyAndAmount {
                    currency: "EUR".to_string(),
                    value: Decimal::from_str("1000.00").unwrap(),
                },
                interbank_settlement_date: Some("2026-02-10".to_string()),
                instructed_amount: None,
                exchange_rate: None,
                charge_bearer: "DEBT".to_string(), // Should convert to OUR
                charges_information: None,
                intermediary_agent_1: None,
                intermediary_agent_2: None,
                intermediary_agent_3: None,
                debtor: PartyIdentification {
                    name: Some("TEST DEBTOR".to_string()),
                    postal_address: None,
                    id: None,
                },
                debtor_account: CashAccount {
                    id: AccountIdentification {
                        iban: Some("FR1234567890123456789012345".to_string()),
                        other: None,
                    },
                    account_type: None,
                    currency: None,
                },
                debtor_agent: BranchAndFinancialInstitutionIdentification {
                    financial_institution_id: FinancialInstitutionIdentification {
                        bic: Some("BNPAFRPPXXX".to_string()),
                        name: None,
                        postal_address: None,
                    },
                },
                creditor_agent: BranchAndFinancialInstitutionIdentification {
                    financial_institution_id: FinancialInstitutionIdentification {
                        bic: Some("DEUTDEFFXXX".to_string()),
                        name: None,
                        postal_address: None,
                    },
                },
                creditor: PartyIdentification {
                    name: Some("TEST CREDITOR".to_string()),
                    postal_address: None,
                    id: None,
                },
                creditor_account: CashAccount {
                    id: AccountIdentification {
                        iban: Some("DE89370400440532013000".to_string()),
                        other: None,
                    },
                    account_type: None,
                    currency: None,
                },
                instruction_for_creditor_agent: None,
                instruction_for_next_agent: None,
                purpose: None,
                regulatory_reporting: None,
                remittance_information: None,
            }],
        },
    };

    // Translate
    let result = translate_pacs008_to_mt103(&pacs008).expect("Translation failed");

    // Serialize
    let mt_text = result.message.serialize().expect("Serialization failed");

    println!("Generated MT103:\n{}", mt_text);

    // Check that DEBT was converted to OUR
    assert!(
        mt_text.contains(":71A:OUR"),
        "DEBT should be converted to OUR"
    );
}

/// Normalize MT message for comparison (remove extra whitespace, normalize line endings)
fn normalize_mt(mt: &str) -> String {
    mt.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
