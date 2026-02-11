//! Integration tests for the translate command.

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

/// Get the path to the test data directory
fn get_test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("paymsg-specs/testdata/translation_pairs")
}

/// Test MT202 → pacs.009 translation
#[test]
fn test_translate_mt202_to_pacs009() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt202")
        .arg("--to")
        .arg("pacs009")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid XML
    assert!(result.contains("<?xml"));
    assert!(result.contains("pacs.009.001"));
}

/// Test pacs.009 → MT202 translation
#[test]
fn test_translate_pacs009_to_mt202() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.xml");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("pacs009")
        .arg("--to")
        .arg("mt202")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid MT format
    assert!(result.starts_with("{1:"));
    assert!(result.contains("{2:"));
    assert!(result.contains("{4:"));
}

/// Test MT940 → camt.053 translation
#[test]
fn test_translate_mt940_to_camt053() {
    let test_dir = get_test_data_dir().join("mt940_camt053");
    let source_file = test_dir.join("pair1_simple.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt940")
        .arg("--to")
        .arg("camt053")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid XML
    assert!(result.contains("<?xml"));
    assert!(result.contains("camt.053.001"));
}

/// Test camt.053 → MT940 translation
#[test]
fn test_translate_camt053_to_mt940() {
    let test_dir = get_test_data_dir().join("mt940_camt053");
    let source_file = test_dir.join("pair1_simple.source.xml");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("camt053")
        .arg("--to")
        .arg("mt940")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid MT format
    assert!(result.starts_with("{1:"));
    assert!(result.contains("{2:"));
    assert!(result.contains("{4:"));
}

/// Test MT942 → camt.052 translation
#[test]
fn test_translate_mt942_to_camt052() {
    let test_dir = get_test_data_dir().join("mt942_camt052");
    let source_file = test_dir.join("pair1_interim.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt942")
        .arg("--to")
        .arg("camt052")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid XML
    assert!(result.contains("<?xml"));
    assert!(result.contains("camt.052.001"));
}

/// Test camt.052 → MT942 translation
#[test]
fn test_translate_camt052_to_mt942() {
    let test_dir = get_test_data_dir().join("mt942_camt052");
    let source_file = test_dir.join("pair1_interim.source.xml");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("camt052")
        .arg("--to")
        .arg("mt942")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Translation failed");
    }

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid MT format
    assert!(result.starts_with("{1:"));
    assert!(result.contains("{2:"));
    assert!(result.contains("{4:"));
}

/// Test auto-detection of source format
#[test]
fn test_translate_auto_detect_mt() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--to")
        .arg("pacs009")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed with auto-detection
    assert!(output.status.success(), "Auto-detection failed");
}

/// Test auto-detection of source format for MX
#[test]
fn test_translate_auto_detect_mx() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.xml");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--to")
        .arg("mt202")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should succeed with auto-detection
    assert!(output.status.success(), "Auto-detection failed");
}

/// Test JSON output format
#[test]
fn test_translate_json_output() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt202")
        .arg("--to")
        .arg("pacs009")
        .arg("--json")
        .arg(&source_file);

    let output = cmd.output().unwrap();

    assert!(output.status.success());

    let result = String::from_utf8(output.stdout).unwrap();

    // Output should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();

    // Check JSON structure
    assert!(json.get("source_type").is_some());
    assert!(json.get("target_type").is_some());
    assert!(json.get("message").is_some());
}

/// Test output to file
#[test]
fn test_translate_output_file() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let output_file = std::env::temp_dir().join("paymsg_test_output.xml");

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt202")
        .arg("--to")
        .arg("pacs009")
        .arg("--output")
        .arg(&output_file)
        .arg(&source_file);

    let output = cmd.output().unwrap();

    assert!(output.status.success());

    // Check that the file was created
    assert!(output_file.exists());

    // Clean up
    fs::remove_file(&output_file).ok();
}

/// Test invalid translation path
#[test]
fn test_translate_invalid_path() {
    let test_dir = get_test_data_dir().join("mt202_pacs009");
    let source_file = test_dir.join("pair1_interbank.source.mt");

    if !source_file.exists() {
        eprintln!("Skipping test: source file not found: {:?}", source_file);
        return;
    }

    let mut cmd = Command::cargo_bin("paymsg").unwrap();
    cmd.arg("translate")
        .arg("--from")
        .arg("mt202")
        .arg("--to")
        .arg("camt053") // Invalid: can't translate MT202 to camt.053
        .arg(&source_file);

    let output = cmd.output().unwrap();

    // Translation should fail
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Invalid translation"));
}
