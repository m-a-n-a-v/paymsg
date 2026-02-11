//! Round-trip tests: parse → serialize → parse → compare
//!
//! These tests ensure that MT messages can be parsed, serialized back to text,
//! and the re-parsed result matches the original structure.

use paymsg_mt::{MtMessage, TextBlock};
use std::path::PathBuf;

fn get_specs_dir() -> PathBuf {
    // Navigate from test binary location to workspace root, then to sibling paymsg-specs
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR should be set during tests");
    let crate_dir = PathBuf::from(manifest_dir);
    // Go up from crates/paymsg-mt to workspace root
    let workspace_root = crate_dir.parent().unwrap().parent().unwrap();
    // Navigate to sibling paymsg-specs directory
    workspace_root.parent().unwrap().join("paymsg-specs")
}

fn normalize_whitespace(s: &str) -> String {
    // Normalize line endings and trim trailing whitespace on each line
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_roundtrip_mt103_minimal() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt103/minimal_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT103");

    // Verify key fields
    assert_eq!(msg.block1.application_id, "F");
    assert_eq!(msg.block1.service_id, "01");
    assert_eq!(msg.block2.message_type, "103");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT103");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT103");

    // Compare structures (should be identical)
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    // Block 4 content should match after normalization
    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_roundtrip_mt103_full() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt103/full_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT103");

    // Verify it has optional blocks
    assert!(msg.block3.is_some(), "Full message should have Block 3");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT103");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT103");

    // Compare structures
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_roundtrip_mt202_minimal() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt202/minimal_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT202");

    assert_eq!(msg.block2.message_type, "202");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT202");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT202");

    // Compare
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_roundtrip_mt202_full() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt202/full_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT202");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT202");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT202");

    // Compare
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_roundtrip_mt940_minimal() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt940/minimal_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT940");

    assert_eq!(msg.block2.message_type, "940");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT940");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT940");

    // Compare
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_roundtrip_mt942_minimal() {
    let specs_dir = get_specs_dir();
    let test_file = specs_dir.join("testdata/mt/mt942/minimal_valid.mt");

    let original_text = std::fs::read_to_string(&test_file).expect("Failed to read test file");

    // Parse
    let msg = MtMessage::parse(&original_text).expect("Failed to parse MT942");

    assert_eq!(msg.block2.message_type, "942");

    // Serialize
    let serialized = msg.serialize().expect("Failed to serialize MT942");

    // Parse again
    let msg2 = MtMessage::parse(&serialized).expect("Failed to re-parse MT942");

    // Compare
    assert_eq!(msg.block1, msg2.block1);
    assert_eq!(msg.block2, msg2.block2);
    assert_eq!(msg.block3, msg2.block3);
    assert_eq!(msg.block5, msg2.block5);

    let content1 = normalize_whitespace(&msg.block4.content);
    let content2 = normalize_whitespace(&msg2.block4.content);
    assert_eq!(content1, content2);
}

#[test]
fn test_serialize_preserves_field_order() {
    // Create a message manually and verify field order is preserved
    let msg = MtMessage {
        block1: paymsg_mt::BasicHeader {
            application_id: "F".to_string(),
            service_id: "01".to_string(),
            logical_terminal_address: "DEUTDEFFAXXX".to_string(),
            session_number: "0".to_string(),
            sequence_number: "0".to_string(),
        },
        block2: paymsg_mt::ApplicationHeader {
            direction: paymsg_mt::Direction::Input,
            message_type: "103".to_string(),
            bic: "BNPAFRPPXXXX".to_string(),
            priority: "N".to_string(),
            delivery_monitoring: None,
            obsolescence_period: None,
        },
        block3: None,
        block4: TextBlock {
            content: ":20:REF123\r\n:32A:260210EUR1234,56\r\n:50K:HANS MUELLER\r\n:59:JEAN DUPONT".to_string(),
        },
        block5: None,
    };

    let serialized = msg.serialize().expect("Failed to serialize");

    // Should contain all fields in order
    assert!(serialized.contains(":20:REF123"));
    assert!(serialized.contains(":32A:260210EUR1234,56"));
    assert!(serialized.contains(":50K:HANS MUELLER"));
    assert!(serialized.contains(":59:JEAN DUPONT"));

    // Verify order
    let pos_20 = serialized.find(":20:").unwrap();
    let pos_32a = serialized.find(":32A:").unwrap();
    let pos_50k = serialized.find(":50K:").unwrap();
    let pos_59 = serialized.find(":59:").unwrap();

    assert!(pos_20 < pos_32a);
    assert!(pos_32a < pos_50k);
    assert!(pos_50k < pos_59);
}

#[test]
fn test_serialize_empty_optional_blocks() {
    // Message without block 3 and block 5
    let msg = MtMessage {
        block1: paymsg_mt::BasicHeader {
            application_id: "F".to_string(),
            service_id: "01".to_string(),
            logical_terminal_address: "DEUTDEFFAXXX".to_string(),
            session_number: "0".to_string(),
            sequence_number: "0".to_string(),
        },
        block2: paymsg_mt::ApplicationHeader {
            direction: paymsg_mt::Direction::Input,
            message_type: "103".to_string(),
            bic: "BNPAFRPPXXXX".to_string(),
            priority: "N".to_string(),
            delivery_monitoring: None,
            obsolescence_period: None,
        },
        block3: None,
        block4: TextBlock {
            content: ":20:REF123".to_string(),
        },
        block5: None,
    };

    let serialized = msg.serialize().expect("Failed to serialize");

    // Should have blocks 1, 2, 4 but not 3 or 5
    assert!(serialized.contains("{1:"));
    assert!(serialized.contains("{2:"));
    assert!(serialized.contains("{4:"));
    assert!(!serialized.contains("{3:"));
    assert!(!serialized.contains("{5:"));
}
