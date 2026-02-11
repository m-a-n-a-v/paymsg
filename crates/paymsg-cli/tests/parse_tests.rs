//! Integration tests for the parse command.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from paymsg-cli
    path.pop(); // Go up from crates
    path.push("target");
    path.push("debug");
    path.push("paymsg");
    path
}

/// Get the path to the testdata directory
fn get_testdata_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from paymsg-cli
    path.pop(); // Go up from crates
    path.pop(); // Go up from paymsg
    path.push("paymsg-specs");
    path.push("testdata");
    path
}

#[test]
fn test_parse_mt940_from_file() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mt");
    testdata.push("mt940");
    testdata.push("minimal_valid.mt");

    // Skip test if testdata doesn't exist
    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap(), "--pretty"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""format": "mt""#), "Expected MT format in output");
    assert!(stdout.contains("mt940"), "Expected mt940 message type");
}

#[test]
fn test_parse_mt103_from_file() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mt");
    testdata.push("mt103");
    testdata.push("minimal_valid.mt");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""format":"mt""#), "Expected MT format in output");
}

#[test]
fn test_parse_pacs008_from_file() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mx");
    testdata.push("pacs.008");
    testdata.push("minimal_valid.xml");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap(), "--pretty"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""format": "mx""#), "Expected MX format in output");
    assert!(stdout.contains("pacs008"), "Expected pacs008 message type");
}

#[test]
fn test_parse_camt053_from_file() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mx");
    testdata.push("camt.053");
    testdata.push("minimal_valid.xml");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""format":"mx""#), "Expected MX format in output");
}

#[test]
fn test_parse_with_format_flag() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mt");
    testdata.push("mt940");
    testdata.push("minimal_valid.mt");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap(), "--format", "mt"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""format":"mt""#), "Expected MT format in output");
}

#[test]
fn test_parse_invalid_format_flag() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mt");
    testdata.push("mt940");
    testdata.push("minimal_valid.mt");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output = Command::new(&binary)
        .args(["parse", testdata.to_str().unwrap(), "--format", "invalid"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should have failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid format"), "Expected format error");
}

#[test]
fn test_parse_nonexistent_file() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["parse", "/nonexistent/file.mt"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should have failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to read file"), "Expected file read error");
}

#[test]
fn test_parse_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["parse", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Parse a message"), "Expected help text");
    assert!(stdout.contains("--format"), "Expected --format option");
    assert!(stdout.contains("--pretty"), "Expected --pretty option");
}

#[test]
fn test_parse_output_to_file() {
    let binary = get_binary_path();
    let mut testdata = get_testdata_path();
    testdata.push("mt");
    testdata.push("mt940");
    testdata.push("minimal_valid.mt");

    if !testdata.exists() {
        eprintln!("Skipping test: testdata not found at {:?}", testdata);
        return;
    }

    let output_file = "/tmp/paymsg_test_output.json";

    let output = Command::new(&binary)
        .args([
            "parse",
            testdata.to_str().unwrap(),
            "--output",
            output_file,
            "--pretty",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command failed: {:?}", output);

    // Check that output file was created
    let content = std::fs::read_to_string(output_file).expect("Failed to read output file");
    assert!(content.contains(r#""format": "mt""#), "Expected MT format in output file");

    // Clean up
    std::fs::remove_file(output_file).ok();
}
