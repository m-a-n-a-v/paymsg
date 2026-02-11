//! Integration tests for reference data validation.

use paymsg_core::specs::SpecLoader;
use paymsg_iso20022::parse_pacs008;
use paymsg_validate::{ReferenceDataValidator, Validator};
use std::path::PathBuf;

fn get_specs_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_dir = PathBuf::from(manifest_dir);
    // From crates/paymsg-validate go up 2 levels to workspace root, then to sibling ../paymsg-specs
    crate_dir.parent().unwrap().parent().unwrap().parent().unwrap().join("paymsg-specs")
}

fn load_validator() -> ReferenceDataValidator {
    let specs_path = get_specs_path();
    let loader = SpecLoader::new(Some(specs_path));
    let specs = loader.load_all().expect("Failed to load specs");
    ReferenceDataValidator::new(specs)
}

#[test]
fn test_validate_valid_pacs008() {
    let validator = load_validator();
    let specs_path = get_specs_path();
    let test_file = specs_path.join("testdata/mx/pacs.008/minimal_valid.xml");
    let xml = std::fs::read_to_string(test_file).expect("Failed to read test file");
    let doc = parse_pacs008(&xml).expect("Failed to parse XML");

    let result = validator.validate(&doc);

    // Minimal valid should have no errors (may have warnings/info)
    let errors = result.errors();
    assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
}

#[test]
fn test_validate_invalid_currency() {
    let validator = load_validator();
    let specs_path = get_specs_path();
    let test_file = specs_path.join("testdata/mx/pacs.008/invalid_currency.xml");

    // Check if the test file exists
    if !test_file.exists() {
        // Create a minimal test with invalid currency inline
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="XXX">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <Othr>
            <Id>12345678</Id>
          </Othr>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

        let doc = parse_pacs008(xml).expect("Failed to parse XML");
        let result = validator.validate(&doc);

        // Should have error for invalid currency code XXX
        let errors = result.errors();
        assert!(!errors.is_empty(), "Expected currency validation error");
        assert!(errors.iter().any(|e| e.id == "REF-002"), "Expected REF-002 error for invalid currency");
    }
}

#[test]
fn test_validate_invalid_decimal_places() {
    let validator = load_validator();

    // Create a test with too many decimal places for USD
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.123</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <Othr>
            <Id>12345678</Id>
          </Othr>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let doc = parse_pacs008(xml).expect("Failed to parse XML");
    let result = validator.validate(&doc);

    // Should have error for too many decimal places (USD requires 2, got 3)
    let errors = result.errors();
    assert!(!errors.is_empty(), "Expected decimal places validation error");
    assert!(errors.iter().any(|e| e.id == "REF-003"), "Expected REF-003 error for decimal places");
}

#[test]
fn test_validate_missing_exchange_rate() {
    let validator = load_validator();

    // Create a test with different currencies but no exchange rate
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <InstdAmt Ccy="EUR">850.00</InstdAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <Othr>
            <Id>12345678</Id>
          </Othr>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let doc = parse_pacs008(xml).expect("Failed to parse XML");
    let result = validator.validate(&doc);

    // Should have error for missing exchange rate when currencies differ
    let errors = result.errors();
    assert!(!errors.is_empty(), "Expected missing exchange rate error");
    assert!(errors.iter().any(|e| e.id == "REF-013"), "Expected REF-013 error for missing exchange rate");
}

#[test]
fn test_validate_invalid_bic() {
    let validator = load_validator();

    // Create a test with invalid BIC
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <Othr>
            <Id>12345678</Id>
          </Othr>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>INVALID</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let doc = parse_pacs008(xml).expect("Failed to parse XML");
    let result = validator.validate(&doc);

    // Should have error for invalid BIC format
    let errors = result.errors();
    assert!(!errors.is_empty(), "Expected invalid BIC error");
    assert!(errors.iter().any(|e| e.id == "REF-005"), "Expected REF-005 error for invalid BIC");
}

#[test]
fn test_validate_invalid_iban() {
    let validator = load_validator();

    // Create a test with invalid IBAN
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <IBAN>DE99370400440532013000</IBAN>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let doc = parse_pacs008(xml).expect("Failed to parse XML");
    let result = validator.validate(&doc);

    // Should have error for invalid IBAN check digits
    let errors = result.errors();
    assert!(!errors.is_empty(), "Expected invalid IBAN error");
    assert!(errors.iter().any(|e| e.id == "REF-007"), "Expected REF-007 error for invalid IBAN");
}

#[test]
fn test_validate_past_settlement_date() {
    let validator = load_validator();

    // Create a test with past settlement date
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG001</MsgId>
      <CreDtTm>2026-02-10T12:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">1000.00</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2020-01-01</IntrBkSttlmDt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>Test Debtor</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <Othr>
            <Id>12345678</Id>
          </Othr>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>Test Creditor</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <Othr>
            <Id>87654321</Id>
          </Othr>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

    let doc = parse_pacs008(xml).expect("Failed to parse XML");
    let result = validator.validate(&doc);

    // Should have warning for past settlement date
    let warnings = result.warnings();
    assert!(!warnings.is_empty(), "Expected past settlement date warning");
    assert!(warnings.iter().any(|w| w.id == "REF-011"), "Expected REF-011 warning for past date");
}

#[test]
fn test_validate_full_valid_pacs008() {
    let validator = load_validator();
    let specs_path = get_specs_path();
    let test_file = specs_path.join("testdata/mx/pacs.008/full_valid.xml");

    if test_file.exists() {
        let xml = std::fs::read_to_string(test_file).expect("Failed to read test file");
        let doc = parse_pacs008(&xml).expect("Failed to parse XML");

        let result = validator.validate(&doc);

        // Full valid should have no errors
        let errors = result.errors();
        assert!(errors.is_empty(), "Expected no errors in full_valid.xml, got: {:?}", errors);
    }
}
