//! Validate command implementation.

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use log::{debug, info};
use paymsg_core::MessageType;
use paymsg_validate::{
    MtSchemaValidator, MxSchemaValidator, BusinessRuleValidator, ReferenceDataValidator,
    ValidationResult, Severity, SwiftCharsetValidator,
};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser)]
pub struct ValidateArgs {
    /// Input file path (reads from stdin if not provided)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file path for validation results (writes to stdout if not provided)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output format: json or table
    #[arg(short = 'f', long, value_name = "FORMAT", default_value = "table")]
    format: String,

    /// Minimum severity level to display: error, warning, or info
    #[arg(short, long, value_name = "SEVERITY", default_value = "info")]
    severity: String,

    /// Rule categories to apply (comma-separated): amount, date, party, structure, all
    #[arg(short, long, value_name = "CATEGORIES", default_value = "all")]
    rules: String,

    /// Path to paymsg-specs directory
    #[arg(long, value_name = "DIR")]
    specs_dir: Option<PathBuf>,

    /// Force message format (mt or mx). Auto-detect if not specified.
    #[arg(long, value_name = "FORMAT")]
    message_format: Option<String>,
}

/// Auto-detect message format from content
fn detect_format(content: &str) -> Result<MessageFormat> {
    let trimmed = content.trim();

    // Check for MT format: starts with {1:
    if trimmed.starts_with("{1:") {
        debug!("Detected MT format (starts with '{{1:')");
        return Ok(MessageFormat::Mt);
    }

    // Check for MX format: starts with XML declaration or <Document
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<Document") {
        debug!("Detected MX format (XML content)");
        return Ok(MessageFormat::Mx);
    }

    anyhow::bail!("Could not auto-detect message format. Use --message-format to specify explicitly.");
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MessageFormat {
    Mt,
    Mx,
}

impl std::str::FromStr for MessageFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mt" => Ok(MessageFormat::Mt),
            "mx" => Ok(MessageFormat::Mx),
            _ => anyhow::bail!("Invalid format '{}'. Must be 'mt' or 'mx'", s),
        }
    }
}

/// Parse minimum severity from string
fn parse_severity(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "error" => Ok(Severity::Error),
        "warning" => Ok(Severity::Warning),
        "info" => Ok(Severity::Info),
        _ => anyhow::bail!("Invalid severity '{}'. Must be 'error', 'warning', or 'info'", s),
    }
}

/// Determine the specs directory to use
fn get_specs_dir(args: &ValidateArgs) -> Result<PathBuf> {
    if let Some(ref dir) = args.specs_dir {
        return Ok(dir.clone());
    }

    if let Ok(dir) = std::env::var("PAYMSG_SPECS_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Default to ../paymsg-specs
    Ok(PathBuf::from("../paymsg-specs"))
}

/// Validate an MT message
fn validate_mt(content: &str, specs_dir: &Path, _rule_categories: &[String]) -> Result<ValidationResult> {
    let message = paymsg_mt::MtMessage::parse(content)
        .context("Failed to parse MT message")?;

    let message_type = detect_mt_type(&message)?;
    info!("Validating {:?} message", message_type);

    let mut combined_result = ValidationResult::new();

    // Load SWIFT character sets
    let charsets = paymsg_validate::SwiftCharsets::load(specs_dir)
        .context("Failed to load SWIFT character sets")?;
    let charset_validator = SwiftCharsetValidator::new(charsets);

    // Schema validation
    let mt_validator = MtSchemaValidator::load(specs_dir, charset_validator)
        .context("Failed to load MT schema validator")?;

    let schema_result = paymsg_validate::Validator::validate(&mt_validator, &message);
    combined_result.merge(schema_result);

    // Note: Reference data validation for MT messages not yet implemented
    // (ReferenceDataValidator only implements Validator for pacs008::Document)
    debug!("Reference data validation for MT messages not yet fully implemented");

    Ok(combined_result)
}

/// Detect specific MT message type from parsed message
fn detect_mt_type(msg: &paymsg_mt::MtMessage) -> Result<MessageType> {
    let msg_type_str = &msg.block2.message_type;
    match msg_type_str.as_str() {
        "103" => Ok(MessageType::Mt103),
        "202" => Ok(MessageType::Mt202),
        "940" => Ok(MessageType::Mt940),
        "942" => Ok(MessageType::Mt942),
        _ => anyhow::bail!("Unsupported MT message type: MT{}", msg_type_str),
    }
}

/// Detect specific MX message type from XML content
fn detect_mx_type(content: &str) -> Result<MessageType> {
    if content.contains("pacs.008.001") {
        Ok(MessageType::Pacs008)
    } else if content.contains("pacs.009.001") {
        Ok(MessageType::Pacs009)
    } else if content.contains("camt.052.001") {
        Ok(MessageType::Camt052)
    } else if content.contains("camt.053.001") {
        Ok(MessageType::Camt053)
    } else {
        anyhow::bail!("Could not detect MX message type from XML content")
    }
}

/// Validate an MX message
fn validate_mx(content: &str, specs_dir: &Path, rule_categories: &[String]) -> Result<ValidationResult> {
    let message_type = detect_mx_type(content)?;
    info!("Validating {:?} message", message_type);

    let mut combined_result = ValidationResult::new();

    // Load specs
    let spec_loader = paymsg_core::specs::SpecLoader::new(Some(specs_dir.to_path_buf()));
    let spec_registries = spec_loader.load_all()
        .context("Failed to load spec registries")?;

    // Schema validation
    let mx_validator = MxSchemaValidator::new();

    match message_type {
        MessageType::Pacs008 => {
            let doc = paymsg_iso20022::parse_pacs008(content)
                .context("Failed to parse pacs.008 message")?;
            let schema_result = paymsg_validate::Validator::validate(&mx_validator, &doc);
            combined_result.merge(schema_result);

            // Reference data validation (only implemented for pacs.008)
            let ref_validator = ReferenceDataValidator::new(spec_registries);
            let ref_result = paymsg_validate::Validator::validate(&ref_validator, &doc);
            combined_result.merge(ref_result);

            // Business rules validation
            if should_apply_business_rules(rule_categories) {
                let rules_file = specs_dir.join("rules/pacs008_rules.json");
                if rules_file.exists() {
                    match BusinessRuleValidator::new(&rules_file, specs_dir) {
                        Ok(business_validator) => {
                            let business_result = paymsg_validate::Validator::validate(&business_validator, &doc);
                            combined_result.merge(business_result);
                        }
                        Err(e) => {
                            debug!("Could not load business rules for pacs.008: {}", e);
                        }
                    }
                }
            }
        }
        MessageType::Pacs009 => {
            let doc = paymsg_iso20022::parse_pacs009(content)
                .context("Failed to parse pacs.009 message")?;
            let schema_result = paymsg_validate::Validator::validate(&mx_validator, &doc);
            combined_result.merge(schema_result);

            // Reference data validation not yet implemented for pacs.009
            debug!("Reference data validation not yet implemented for pacs.009");

            // Business rules validation
            if should_apply_business_rules(rule_categories) {
                let rules_file = specs_dir.join("rules/pacs009_rules.json");
                if rules_file.exists() {
                    debug!("Business rules validation not yet fully implemented for pacs.009");
                }
            }
        }
        MessageType::Camt052 => {
            let doc = paymsg_iso20022::parse_camt052(content)
                .context("Failed to parse camt.052 message")?;
            let schema_result = paymsg_validate::Validator::validate(&mx_validator, &doc);
            combined_result.merge(schema_result);

            // Reference data validation not yet implemented for camt.052
            debug!("Reference data validation not yet implemented for camt.052");

            // Business rules validation
            if should_apply_business_rules(rule_categories) {
                let rules_file = specs_dir.join("rules/camt052_rules.json");
                if rules_file.exists() {
                    debug!("Business rules validation not yet fully implemented for camt.052");
                }
            }
        }
        MessageType::Camt053 => {
            let doc = paymsg_iso20022::parse_camt053(content)
                .context("Failed to parse camt.053 message")?;
            let schema_result = paymsg_validate::Validator::validate(&mx_validator, &doc);
            combined_result.merge(schema_result);

            // Reference data validation not yet implemented for camt.053
            debug!("Reference data validation not yet implemented for camt.053");

            // Business rules validation
            if should_apply_business_rules(rule_categories) {
                let rules_file = specs_dir.join("rules/camt053_rules.json");
                if rules_file.exists() {
                    debug!("Business rules validation not yet fully implemented for camt.053");
                }
            }
        }
        _ => anyhow::bail!("Invalid MX message type: {:?}", message_type),
    }

    Ok(combined_result)
}

/// Check if business rules should be applied based on rule categories
fn should_apply_business_rules(categories: &[String]) -> bool {
    categories.iter().any(|c| c == "all" || c == "business")
}

/// Filter validation result by minimum severity
fn filter_by_severity(result: &ValidationResult, min_severity: &Severity) -> ValidationResult {
    let mut filtered = ValidationResult::new();

    for issue in &result.issues {
        let should_include = match min_severity {
            Severity::Error => issue.severity == Severity::Error,
            Severity::Warning => matches!(issue.severity, Severity::Error | Severity::Warning),
            Severity::Info => true, // Include all
        };

        if should_include {
            filtered.add_issue(issue.clone());
        }
    }

    filtered
}

/// Format validation result as JSON
fn format_as_json(result: &ValidationResult) -> Result<String> {
    serde_json::to_string_pretty(result)
        .context("Failed to serialize validation result to JSON")
}

/// Format validation result as a table with colors
fn format_as_table(result: &ValidationResult) -> String {
    if result.issues.is_empty() {
        return format!("{}\n", "✓ Validation passed - no issues found".green().bold());
    }

    let mut output = String::new();

    // Summary line
    let error_count = result.errors().len();
    let warning_count = result.warnings().len();
    let info_count = result.infos().len();

    let summary = if error_count > 0 {
        format!("✗ Validation failed: {} error(s), {} warning(s), {} info",
                error_count, warning_count, info_count).red().bold()
    } else if warning_count > 0 {
        format!("⚠ Validation passed with warnings: {} warning(s), {} info",
                warning_count, info_count).yellow().bold()
    } else {
        format!("ℹ Validation passed: {} info message(s)", info_count).blue().bold()
    };

    output.push_str(&format!("{}\n\n", summary));

    // Issue details
    for issue in &result.issues {
        let severity_str = match issue.severity {
            Severity::Error => "ERROR".red().bold(),
            Severity::Warning => "WARN ".yellow().bold(),
            Severity::Info => "INFO ".blue().bold(),
        };

        output.push_str(&format!("{} [{}]", severity_str, issue.id.bright_black()));

        if let Some(ref path) = issue.field_path {
            output.push_str(&format!(" {}", path.cyan()));
        }

        output.push_str(&format!("\n      {}\n", issue.message));

        if let Some(ref suggestion) = issue.suggestion {
            output.push_str(&format!("      💡 {}\n", suggestion.bright_black()));
        }

        output.push('\n');
    }

    output
}

/// Execute the validate command
pub fn execute(args: ValidateArgs) -> Result<()> {
    // Read input
    let content = if let Some(ref path) = args.input {
        debug!("Reading from file: {}", path.display());
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?
    } else {
        debug!("Reading from stdin");
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    };

    // Determine format
    let format = if let Some(ref fmt_str) = args.message_format {
        fmt_str.parse::<MessageFormat>()?
    } else {
        detect_format(&content)?
    };

    debug!("Using format: {:?}", format);

    // Get specs directory
    let specs_dir = get_specs_dir(&args)?;
    debug!("Using specs directory: {}", specs_dir.display());

    // Parse rule categories
    let rule_categories: Vec<String> = args.rules
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
    debug!("Rule categories: {:?}", rule_categories);

    // Validate message based on format
    let result = match format {
        MessageFormat::Mt => validate_mt(&content, &specs_dir, &rule_categories)?,
        MessageFormat::Mx => validate_mx(&content, &specs_dir, &rule_categories)?,
    };

    // Parse minimum severity
    let min_severity = parse_severity(&args.severity)?;

    // Filter by severity
    let filtered_result = filter_by_severity(&result, &min_severity);

    // Format output
    let output_str = match args.format.to_lowercase().as_str() {
        "json" => format_as_json(&filtered_result)?,
        "table" => format_as_table(&filtered_result),
        _ => anyhow::bail!("Invalid output format '{}'. Must be 'json' or 'table'", args.format),
    };

    // Write output
    if let Some(ref path) = args.output {
        debug!("Writing to file: {}", path.display());
        fs::write(path, output_str)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
        info!("Output written to: {}", path.display());
    } else {
        print!("{}", output_str);
    }

    // Exit with appropriate code
    if result.has_errors() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
