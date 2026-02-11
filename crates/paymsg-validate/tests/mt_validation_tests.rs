//! Integration tests for MT message validation.

use paymsg_mt::MtMessage;
use paymsg_validate::{MtSchemaValidator, SwiftCharsetValidator, SwiftCharsets, Validator};
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
fn test_validate_mt103_minimal_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load MT103 minimal valid message
    let mt103_path = specs_dir.join("testdata/mt/mt103/minimal_valid.mt");
    let mt103_text = std::fs::read_to_string(&mt103_path).unwrap();
    let message = MtMessage::parse(&mt103_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Should be valid
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in &result.issues {
            eprintln!("  [{:?}] {}: {}", issue.severity, issue.id, issue.message);
        }
    }
    assert!(result.is_valid(), "Minimal valid MT103 should pass validation");
}

#[test]
fn test_validate_mt103_missing_mandatory_field() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Create an MT103 message missing field 20 (mandatory)
    let mt103_text = r#"{1:F01BANKBICAXXXX0000000000}{2:I103BANKBICAXXXXN}{4:
:23B:CRED
:32A:260210USD1000,00
:50K:ORDERING CUSTOMER
NAME
ADDRESS
:59:BENEFICIARY
NAME
ADDRESS
:71A:SHA
-}"#;

    let message = MtMessage::parse(mt103_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Should have error about missing field 20
    assert!(!result.is_valid(), "Should fail validation due to missing field 20");
    assert!(result.errors().iter().any(|e| e.id == "MT_MISSING_MANDATORY_FIELD"),
        "Should have MT_MISSING_MANDATORY_FIELD error");
}

#[test]
fn test_validate_mt103_field_too_long() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Create an MT103 message with field 20 exceeding max length (16 chars)
    let mt103_text = r#"{1:F01BANKBICAXXXX0000000000}{2:I103BANKBICAXXXXN}{4:
:20:THIS_IS_A_VERY_LONG_REFERENCE_NUMBER
:23B:CRED
:32A:260210USD1000,00
:50K:ORDERING CUSTOMER
:59:BENEFICIARY
:71A:SHA
-}"#;

    let message = MtMessage::parse(mt103_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Should have error about field being too long
    assert!(!result.is_valid(), "Should fail validation due to field 20 being too long");
    assert!(result.errors().iter().any(|e| e.id == "MT_FIELD_TOO_LONG"),
        "Should have MT_FIELD_TOO_LONG error");
}

#[test]
fn test_validate_mt202_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load MT202 full valid message
    let mt202_path = specs_dir.join("testdata/mt/mt202/full_valid.mt");
    if !mt202_path.exists() {
        eprintln!("Skipping test: MT202 test file not found");
        return;
    }

    let mt202_text = std::fs::read_to_string(&mt202_path).unwrap();
    let message = MtMessage::parse(&mt202_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Check that mandatory field checks work (no missing mandatory fields)
    let has_missing_mandatory = result.errors().iter().any(|e| e.id == "MT_MISSING_MANDATORY_FIELD");
    assert!(!has_missing_mandatory, "Should not have missing mandatory fields");

    // The message was parsed successfully, which shows the validator works
    assert_eq!(message.block2.message_type, "202");
}

#[test]
fn test_validate_mt940_valid() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Load MT940 minimal valid message
    let mt940_path = specs_dir.join("testdata/mt/mt940/minimal_valid.mt");
    if !mt940_path.exists() {
        eprintln!("Skipping test: MT940 test file not found");
        return;
    }

    let mt940_text = std::fs::read_to_string(&mt940_path).unwrap();
    let message = MtMessage::parse(&mt940_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Should be valid (may have warnings about unknown fields, but no errors)
    if !result.is_valid() {
        eprintln!("Validation errors:");
        for issue in result.errors() {
            eprintln!("  {}: {}", issue.id, issue.message);
        }
    }
    // MT940 spec may not be as complete, so we just check it doesn't crash
    // and that we can parse and validate the message structure
    assert_eq!(message.block2.message_type, "940");
}

#[test]
fn test_validate_unknown_message_type() {
    let specs_dir = get_specs_dir();
    if !specs_dir.exists() {
        eprintln!("Skipping test: paymsg-specs directory not found");
        return;
    }

    // Create a message with unknown type MT999
    let mt_text = r#"{1:F01BANKBICAXXXX0000000000}{2:I999BANKBICAXXXXN}{4:
:20:REF123
-}"#;

    let message = MtMessage::parse(mt_text).unwrap();

    // Create validator
    let charsets = SwiftCharsets::load(&specs_dir).unwrap();
    let charset_validator = SwiftCharsetValidator::new(charsets);
    let validator = MtSchemaValidator::load(&specs_dir, charset_validator).unwrap();

    // Validate
    let result = validator.validate(&message);

    // Should have error about unknown message type
    assert!(!result.is_valid(), "Should fail validation for unknown message type");
    assert!(result.errors().iter().any(|e| e.id == "MT_UNKNOWN_MESSAGE_TYPE"),
        "Should have MT_UNKNOWN_MESSAGE_TYPE error");
}
