//! Integration tests using real MT message files from paymsg-specs.

use paymsg_mt::{Direction, MtMessage};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the path to the paymsg-specs directory.
fn get_specs_dir() -> PathBuf {
    if let Ok(specs_dir) = env::var("PAYMSG_SPECS_DIR") {
        PathBuf::from(specs_dir)
    } else {
        // During tests, we're in crates/paymsg-mt, so go up 3 levels to workspace root,
        // then to sibling paymsg-specs
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("paymsg-specs")
    }
}

#[test]
fn test_parse_mt103_minimal_valid() {
    let specs_dir = get_specs_dir();
    let file_path = specs_dir.join("testdata/mt/mt103/minimal_valid.mt");

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let msg = MtMessage::parse(&content).expect("Failed to parse MT103 minimal valid");

    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block1.service_id, "01");
    assert_eq!(msg.block1.logical_terminal_address, "DEUTDEFFAXXX");

    assert_eq!(msg.block2.direction, Direction::Input);
    assert_eq!(msg.block2.message_type, "103");
    assert_eq!(msg.block2.bic, "BNPAFRPPXXXX");
    assert_eq!(msg.block2.priority, "N");

    assert!(msg.block3.is_none());
    assert!(msg.block4.content.contains(":20:TESTREF12345"));
    assert!(msg.block4.content.contains(":23B:CRED"));
    assert!(msg.block4.content.contains(":32A:260210EUR1234,56"));
    assert!(msg.block4.content.contains(":71A:SHA"));
    assert!(msg.block5.is_none());
}

#[test]
fn test_parse_mt103_full_valid() {
    let specs_dir = get_specs_dir();
    let file_path = specs_dir.join("testdata/mt/mt103/full_valid.mt");

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let msg = MtMessage::parse(&content).expect("Failed to parse MT103 full valid");

    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block2.message_type, "103");

    // Check block 3 (User Header)
    assert!(msg.block3.is_some());
    let block3 = msg.block3.as_ref().unwrap();
    assert_eq!(block3.tags.get("108").unwrap(), "TESTMUR123456");
    assert_eq!(
        block3.tags.get("121").unwrap(),
        "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    );

    // Check block 4 content
    assert!(msg.block4.content.contains(":20:FULLREF98765"));
    assert!(msg.block4.content.contains(":32A:260215EUR10000,00"));
    assert!(msg.block4.content.contains(":71A:SHA"));

    // Check block 5 (Trailer)
    assert!(msg.block5.is_some());
    let block5 = msg.block5.as_ref().unwrap();
    assert_eq!(block5.tags.get("CHK").unwrap(), "123456789ABC");
}

#[test]
fn test_parse_mt202_minimal_valid() {
    let specs_dir = get_specs_dir();
    let file_path = specs_dir.join("testdata/mt/mt202/minimal_valid.mt");

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let msg = MtMessage::parse(&content).expect("Failed to parse MT202 minimal valid");

    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block2.message_type, "202");
    assert_eq!(msg.block2.bic, "BNPAFRPPXXXX");

    assert!(msg.block4.content.contains(":20:FI20231115001"));
    assert!(msg.block4.content.contains(":21:MT103REF123456"));
    assert!(msg.block4.content.contains(":32A:231115EUR50000,00"));
    assert!(msg.block4.content.contains(":58A:BNPAFRPP"));
}

#[test]
fn test_parse_mt940_minimal_valid() {
    let specs_dir = get_specs_dir();
    let file_path = specs_dir.join("testdata/mt/mt940/minimal_valid.mt");

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let msg = MtMessage::parse(&content).expect("Failed to parse MT940 minimal valid");

    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block2.message_type, "940");

    assert!(msg.block4.content.contains(":20:STMT20231115001"));
    assert!(msg.block4.content.contains(":25:DE89370400440532013000"));
    assert!(msg.block4.content.contains(":28C:235/1"));
    assert!(msg.block4.content.contains(":60F:C231114EUR10000,00"));
    assert!(msg.block4.content.contains(":62F:C231115EUR10000,00"));
}

#[test]
fn test_parse_mt942_minimal_valid() {
    let specs_dir = get_specs_dir();
    let file_path = specs_dir.join("testdata/mt/mt942/minimal_valid.mt");

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

    let msg = MtMessage::parse(&content).expect("Failed to parse MT942 minimal valid");

    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block2.message_type, "942");

    assert!(msg.block4.content.contains(":20:"));
    assert!(msg.block4.content.contains(":25:"));
}

#[test]
fn test_parse_multiple_mt103_files() {
    let specs_dir = get_specs_dir();
    let mt103_dir = specs_dir.join("testdata/mt/mt103");

    let files = vec![
        "minimal_valid.mt",
        "full_valid.mt",
        "cross_border.mt",
        "sepa_credit.mt",
        "usd_wire.mt",
    ];

    for file_name in files {
        let file_path = mt103_dir.join(file_name);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

            let msg = MtMessage::parse(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", file_name, e));

            assert_eq!(msg.block2.message_type, "103", "File: {}", file_name);
            assert!(!msg.block4.content.is_empty(), "File: {}", file_name);
        }
    }
}

#[test]
fn test_parse_multiple_mt940_files() {
    let specs_dir = get_specs_dir();
    let mt940_dir = specs_dir.join("testdata/mt/mt940");

    let files = vec![
        "minimal_valid.mt",
        "multi_statement_line.mt",
        "month_end.mt",
    ];

    for file_name in files {
        let file_path = mt940_dir.join(file_name);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

            let msg = MtMessage::parse(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", file_name, e));

            assert_eq!(msg.block2.message_type, "940", "File: {}", file_name);
        }
    }
}

#[test]
fn test_handle_different_line_endings() {
    let input_lf = "{1:F01DEUTDEFFAXXX0000000000}{2:I103BNPAFRPPXXXXN}{4:\n:20:REF\n:32A:260210EUR1000,00\n-}";
    let input_crlf = "{1:F01DEUTDEFFAXXX0000000000}{2:I103BNPAFRPPXXXXN}{4:\r\n:20:REF\r\n:32A:260210EUR1000,00\r\n-}";

    let msg_lf = MtMessage::parse(input_lf).expect("Failed to parse with LF");
    let msg_crlf = MtMessage::parse(input_crlf).expect("Failed to parse with CRLF");

    assert_eq!(msg_lf.block2.message_type, msg_crlf.block2.message_type);
    assert!(msg_lf.block4.content.contains(":20:REF"));
    assert!(msg_crlf.block4.content.contains(":20:REF"));
}
