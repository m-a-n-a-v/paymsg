//! Integration tests for the info subcommand.

use assert_cmd::Command;
use std::fs;

const MT103_SAMPLE: &str = r#"{1:F01BANKUS33XXX0000000000}{2:I103BANKGB2LXXXXN}{4:
:20:REF123456789
:23B:CRED
:32A:260210EUR1000,50
:50K:/123456789
ACME Corporation
123 Main Street
New York NY 10001
:59:/987654321
XYZ Limited
456 High Street
London EC1A 1BB
:71A:SHA
-}"#;

const PACS008_SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG20260210123456</MsgId>
      <CreDtTm>2026-02-10T12:00:00</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <SttlmInf>
        <SttlmMtd>INDA</SttlmMtd>
      </SttlmInf>
      <InstgAgt>
        <FinInstnId>
          <BICFI>BANKUS33XXX</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BANKGB2LXXX</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <InstrId>INSTR123</InstrId>
        <EndToEndId>E2E456</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="EUR">1000.50</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2026-02-10</IntrBkSttlmDt>
      <ChrgBr>SHAR</ChrgBr>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

const MT940_SAMPLE: &str = r#"{1:F01BANKUS33XXX0000000000}{2:O940BANKGB2LXXXXN}{4:
:20:STMT123456
:25:GB33BUKB20201555555555
:28C:00001/001
:60F:C260209EUR10000,00
:61:260210D500,00NCHKNONREF//CHECK001
:86:Check payment
:62F:C260210EUR9500,00
-}"#;

#[test]
fn test_info_mt103_from_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("mt103.txt");
    fs::write(&input_path, MT103_SAMPLE).unwrap();

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    let output = cmd.arg("info").arg(&input_path).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON contains expected fields
    assert!(stdout.contains(r#""format": "MT""#));
    assert!(stdout.contains(r#""message_type": "MT103""#));
    assert!(stdout.contains(r#""reference": "REF123456789""#));
    assert!(stdout.contains(r#""currency": "EUR""#));
    assert!(stdout.contains(r#""amount": "1000.50""#));
    assert!(stdout.contains(r#""date": "260210""#));
}

#[test]
fn test_info_pacs008_from_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("pacs008.xml");
    fs::write(&input_path, PACS008_SAMPLE).unwrap();

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    let output = cmd.arg("info").arg(&input_path).assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON contains expected fields
    assert!(stdout.contains(r#""format": "MX""#));
    assert!(stdout.contains(r#""message_type": "pacs.008""#));
    assert!(stdout.contains(r#""reference": "MSG20260210123456""#));
    assert!(stdout.contains(r#""currency": "EUR""#));
    assert!(stdout.contains(r#""amount": "1000.50""#));
    assert!(stdout.contains(r#""sender": "BANKUS33XXX""#));
    assert!(stdout.contains(r#""receiver": "BANKGB2LXXX""#));
}

#[test]
fn test_info_mt940_from_stdin() {
    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    let output = cmd
        .arg("info")
        .write_stdin(MT940_SAMPLE)
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON contains expected fields
    assert!(stdout.contains(r#""format": "MT""#));
    assert!(stdout.contains(r#""message_type": "MT940""#));
    assert!(stdout.contains(r#""currency": "EUR""#));
    // Opening balance amount
    assert!(stdout.contains(r#""amount": "10000.00""#));
}

#[test]
fn test_info_invalid_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("invalid.txt");
    fs::write(&input_path, "This is not a valid message").unwrap();

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("info")
        .arg(&input_path)
        .assert()
        .failure();
}

#[test]
fn test_info_nonexistent_file() {
    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("info")
        .arg("/nonexistent/file.txt")
        .assert()
        .failure();
}

#[test]
fn test_info_help() {
    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    let output = cmd.arg("info").arg("--help").assert().success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Show message information"));
    assert!(stdout.contains("EXAMPLES"));
}
