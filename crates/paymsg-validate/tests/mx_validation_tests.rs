//! Integration tests for MX message validation.

use paymsg_validate::{MxSchemaValidator, Validator};
use std::path::Path;

fn get_specs_dir() -> std::path::PathBuf {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let crate_path = Path::new(&crate_dir);
    // Navigate from crates/paymsg-validate to workspace root, then to sibling paymsg-specs
    crate_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("paymsg-specs")
}

#[test]
fn test_validate_pacs008_minimal_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load pacs.008 minimal valid message
    let pacs008_path = specs_dir.join("testdata/mx/pacs.008/minimal_valid.xml");
    let xml_text = std::fs::read_to_string(&pacs008_path).unwrap();
    let document = paymsg_iso20022::parse_pacs008(&xml_text).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Minimal valid pacs.008 should pass validation");
}

#[test]
fn test_validate_pacs008_full_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load pacs.008 full valid message
    let pacs008_path = specs_dir.join("testdata/mx/pacs.008/full_valid.xml");
    let xml_text = std::fs::read_to_string(&pacs008_path).unwrap();
    let document = paymsg_iso20022::parse_pacs008(&xml_text).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Full valid pacs.008 should pass validation");
}

#[test]
fn test_validate_pacs009_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load pacs.009 minimal valid message
    let pacs009_path = specs_dir.join("testdata/mx/pacs.009/minimal_valid.xml");
    if !pacs009_path.exists() {
        eprintln!("Skipping test: pacs.009 test file not found");
        return;
    }

    let xml_text = std::fs::read_to_string(&pacs009_path).unwrap();
    let document = paymsg_iso20022::parse_pacs009(&xml_text).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Valid pacs.009 should pass validation");
}

#[test]
fn test_validate_camt053_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load camt.053 minimal valid message
    let camt053_path = specs_dir.join("testdata/mx/camt.053/minimal_valid.xml");
    if !camt053_path.exists() {
        eprintln!("Skipping test: camt.053 test file not found");
        return;
    }

    let xml_text = std::fs::read_to_string(&camt053_path).unwrap();
    let document = paymsg_iso20022::parse_camt053(&xml_text).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Valid camt.053 should pass validation");
}

#[test]
fn test_validate_camt052_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load camt.052 minimal valid message
    let camt052_path = specs_dir.join("testdata/mx/camt.052/minimal_valid.xml");
    if !camt052_path.exists() {
        eprintln!("Skipping test: camt.052 test file not found");
        return;
    }

    let xml_text = std::fs::read_to_string(&camt052_path).unwrap();
    let document = paymsg_iso20022::parse_camt052(&xml_text).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Valid camt.052 should pass validation");
}

#[test]
fn test_validate_pacs008_invalid_currency() {
    // Create a document with invalid currency code
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></InstgAgt>
      <InstdAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId><EndToEndId>E2E123</EndToEndId></PmtId>
      <IntrBkSttlmAmt Ccy="us">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr><Nm>Test Debtor</Nm></Dbtr>
      <DbtrAcct><Id><Othr><Id>ACC123</Id></Othr></Id></DbtrAcct>
      <DbtrAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></DbtrAgt>
      <CdtrAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></CdtrAgt>
      <Cdtr><Nm>Test Creditor</Nm></Cdtr>
      <CdtrAcct><Id><Othr><Id>ACC456</Id></Othr></Id></CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let document = paymsg_iso20022::parse_pacs008(xml).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should have error about invalid currency
    assert!(!result.is_valid(), "Should fail validation due to invalid currency");
    assert!(result.errors().iter().any(|e| e.id == "MX_INVALID_CURRENCY"),
        "Should have MX_INVALID_CURRENCY error");
}

#[test]
fn test_validate_pacs008_zero_amount() {
    // Create a document with zero amount
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></InstgAgt>
      <InstdAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId><EndToEndId>E2E123</EndToEndId></PmtId>
      <IntrBkSttlmAmt Ccy="USD">0.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr><Nm>Test Debtor</Nm></Dbtr>
      <DbtrAcct><Id><Othr><Id>ACC123</Id></Othr></Id></DbtrAcct>
      <DbtrAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></DbtrAgt>
      <CdtrAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></CdtrAgt>
      <Cdtr><Nm>Test Creditor</Nm></Cdtr>
      <CdtrAcct><Id><Othr><Id>ACC456</Id></Othr></Id></CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let document = paymsg_iso20022::parse_pacs008(xml).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should have error about zero/negative amount
    assert!(!result.is_valid(), "Should fail validation due to zero amount");
    assert!(result.errors().iter().any(|e| e.id == "MX_INVALID_AMOUNT"),
        "Should have MX_INVALID_AMOUNT error");
}

#[test]
fn test_validate_pacs008_transaction_count_mismatch() {
    // Create a document with mismatched transaction count
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>5</NbOfTxs>
      <InstgAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></InstgAgt>
      <InstdAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId><EndToEndId>E2E123</EndToEndId></PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr><Nm>Test Debtor</Nm></Dbtr>
      <DbtrAcct><Id><Othr><Id>ACC123</Id></Othr></Id></DbtrAcct>
      <DbtrAgt><FinInstnId><BICFI>TESTBIC1XXX</BICFI></FinInstnId></DbtrAgt>
      <CdtrAgt><FinInstnId><BICFI>TESTBIC2XXX</BICFI></FinInstnId></CdtrAgt>
      <Cdtr><Nm>Test Creditor</Nm></Cdtr>
      <CdtrAcct><Id><Othr><Id>ACC456</Id></Othr></Id></CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let document = paymsg_iso20022::parse_pacs008(xml).unwrap();

    // Create validator
    let validator = MxSchemaValidator::new();

    // Validate
    let result = validator.validate(&document);

    // Should have error about transaction count mismatch
    assert!(!result.is_valid(), "Should fail validation due to transaction count mismatch");
    assert!(result.errors().iter().any(|e| e.id == "MX_TRANSACTION_COUNT_MISMATCH"),
        "Should have MX_TRANSACTION_COUNT_MISMATCH error");
}
