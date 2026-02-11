//! Translate command implementation.

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info, warn};
use paymsg_core::MessageType;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
pub struct TranslateArgs {
    /// Input file path (reads from stdin if not provided)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Source format (mt103, pacs008, mt202, pacs009, mt940, camt053, mt942, camt052).
    /// Auto-detect if not specified.
    #[arg(long, value_name = "FORMAT")]
    from: Option<String>,

    /// Target format (mt103, pacs008, mt202, pacs009, mt940, camt053, mt942, camt052)
    #[arg(long, value_name = "FORMAT", required = true)]
    to: String,

    /// Output file path (writes to stdout if not provided)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Validate the output message after translation
    #[arg(long)]
    validate: bool,

    /// Output structured JSON instead of native format
    #[arg(long)]
    json: bool,

    /// Path to specs directory (overrides PAYMSG_SPECS_DIR)
    #[arg(long, value_name = "DIR")]
    specs_dir: Option<PathBuf>,
}

/// Auto-detect message type from content
fn detect_message_type(content: &str) -> Result<MessageType> {
    let trimmed = content.trim();

    // Check for MT format
    if trimmed.starts_with("{1:") {
        // Parse to detect specific MT type
        let msg = paymsg_mt::MtMessage::parse(content)
            .context("Failed to parse MT message for type detection")?;

        match msg.block2.message_type.as_str() {
            "103" => return Ok(MessageType::Mt103),
            "202" => return Ok(MessageType::Mt202),
            "940" => return Ok(MessageType::Mt940),
            "942" => return Ok(MessageType::Mt942),
            other => anyhow::bail!("Unsupported MT message type: MT{}", other),
        }
    }

    // Check for MX format
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<Document") {
        if content.contains("pacs.008.001") {
            return Ok(MessageType::Pacs008);
        } else if content.contains("pacs.009.001") {
            return Ok(MessageType::Pacs009);
        } else if content.contains("camt.052.001") {
            return Ok(MessageType::Camt052);
        } else if content.contains("camt.053.001") {
            return Ok(MessageType::Camt053);
        } else {
            anyhow::bail!("Could not detect MX message type from XML content");
        }
    }

    anyhow::bail!("Could not auto-detect message format. Use --from to specify explicitly.")
}

/// Parse message type from string
fn parse_message_type(s: &str) -> Result<MessageType> {
    match s.to_lowercase().as_str() {
        "mt103" => Ok(MessageType::Mt103),
        "mt202" => Ok(MessageType::Mt202),
        "mt940" => Ok(MessageType::Mt940),
        "mt942" => Ok(MessageType::Mt942),
        "pacs008" | "pacs.008" => Ok(MessageType::Pacs008),
        "pacs009" | "pacs.009" => Ok(MessageType::Pacs009),
        "camt052" | "camt.052" => Ok(MessageType::Camt052),
        "camt053" | "camt.053" => Ok(MessageType::Camt053),
        _ => anyhow::bail!("Invalid message type: '{}'. Must be one of: mt103, pacs008, mt202, pacs009, mt940, camt053, mt942, camt052", s),
    }
}

/// Wrapper for translation output
#[derive(Debug, serde::Serialize)]
struct TranslationOutput {
    source_type: MessageType,
    target_type: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<paymsg_translate::DataLossWarning>>,
    message: serde_json::Value,
}

/// Execute the translate command
pub fn execute(args: TranslateArgs) -> Result<()> {
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

    // Determine source type
    let source_type = if let Some(ref from_str) = args.from {
        parse_message_type(from_str)?
    } else {
        detect_message_type(&content)?
    };

    // Parse target type
    let target_type = parse_message_type(&args.to)?;

    debug!("Translating from {:?} to {:?}", source_type, target_type);

    // Perform translation
    let (output_content, warnings) = translate_message(&content, source_type, target_type, args.json)?;

    // Report warnings to stderr
    if !warnings.is_empty() {
        for warning in &warnings {
            warn!(
                "Data loss - {}: {} ({:?})",
                warning.field_path,
                warning.description,
                warning.category
            );
            if let Some(ref original) = warning.original_value {
                warn!("  Original value: {}", original);
            }
        }
        eprintln!("\n{} data loss warning(s) encountered during translation", warnings.len());
    }

    // Validate if requested
    if args.validate {
        validate_output(&output_content, target_type, args.specs_dir.as_deref())?;
    }

    // Write output
    if let Some(ref path) = args.output {
        debug!("Writing to file: {}", path.display());
        fs::write(path, &output_content)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
        info!("Output written to: {}", path.display());
    } else {
        print!("{}", output_content);
    }

    Ok(())
}

/// Perform message translation
fn translate_message(
    content: &str,
    source_type: MessageType,
    target_type: MessageType,
    json_output: bool,
) -> Result<(String, Vec<paymsg_translate::DataLossWarning>)> {
    use paymsg_translate::*;

    match (source_type, target_type) {
        // MT103 ↔ pacs.008
        (MessageType::Mt103, MessageType::Pacs008) => {
            anyhow::bail!("MT103 → pacs.008 translation not yet supported (PAYMSG-014 incomplete)")
        }
        (MessageType::Pacs008, MessageType::Mt103) => {
            let pacs008 = paymsg_iso20022::parse_pacs008(content)
                .context("Failed to parse pacs.008 message")?;

            let result = translate_pacs008_to_mt103(&pacs008)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                result.message.serialize()?
            };

            Ok((output, result.warnings))
        }

        // MT202 ↔ pacs.009
        (MessageType::Mt202, MessageType::Pacs009) => {
            let mt202 = paymsg_mt::MtMessage::parse(content)
                .context("Failed to parse MT202 message")?;

            let result = translate_mt202_to_pacs009(&mt202)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                paymsg_iso20022::serialize_pacs009(&result.message)?
            };

            Ok((output, result.warnings))
        }
        (MessageType::Pacs009, MessageType::Mt202) => {
            let pacs009 = paymsg_iso20022::parse_pacs009(content)
                .context("Failed to parse pacs.009 message")?;

            let result = translate_pacs009_to_mt202(&pacs009)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                result.message.serialize()?
            };

            Ok((output, result.warnings))
        }

        // MT940 ↔ camt.053
        (MessageType::Mt940, MessageType::Camt053) => {
            let mt940 = paymsg_mt::MtMessage::parse(content)
                .context("Failed to parse MT940 message")?;

            let result = translate_mt940_to_camt053(&mt940)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                paymsg_iso20022::serialize_camt053(&result.message)?
            };

            Ok((output, result.warnings))
        }
        (MessageType::Camt053, MessageType::Mt940) => {
            let camt053 = paymsg_iso20022::parse_camt053(content)
                .context("Failed to parse camt.053 message")?;

            let result = translate_camt053_to_mt940(&camt053)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                result.message.serialize()?
            };

            Ok((output, result.warnings))
        }

        // MT942 ↔ camt.052
        (MessageType::Mt942, MessageType::Camt052) => {
            let mt942 = paymsg_mt::MtMessage::parse(content)
                .context("Failed to parse MT942 message")?;

            let result = translate_mt942_to_camt052(&mt942)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                paymsg_iso20022::serialize_camt052(&result.message)?
            };

            Ok((output, result.warnings))
        }
        (MessageType::Camt052, MessageType::Mt942) => {
            let camt052 = paymsg_iso20022::parse_camt052(content)
                .context("Failed to parse camt.052 message")?;

            let result = translate_camt052_to_mt942(&camt052)
                .context("Translation failed")?;

            let output = if json_output {
                serde_json::to_string_pretty(&TranslationOutput {
                    source_type,
                    target_type,
                    warnings: if result.warnings.is_empty() { None } else { Some(result.warnings.clone()) },
                    message: serde_json::to_value(&result.message)?,
                })?
            } else {
                result.message.serialize()?
            };

            Ok((output, result.warnings))
        }

        // Invalid translation path
        _ => anyhow::bail!(
            "Invalid translation: {:?} → {:?}. Supported translations: mt103↔pacs008, mt202↔pacs009, mt940↔camt053, mt942↔camt052",
            source_type,
            target_type
        ),
    }
}

/// Validate translated output
fn validate_output(
    content: &str,
    message_type: MessageType,
    specs_dir: Option<&std::path::Path>,
) -> Result<()> {
    use paymsg_validate::Severity;

    info!("Validating translated {:?} message", message_type);

    // Load specs
    let specs_path = if let Some(path) = specs_dir {
        path.to_path_buf()
    } else if let Ok(env_path) = std::env::var("PAYMSG_SPECS_DIR") {
        PathBuf::from(env_path)
    } else {
        PathBuf::from("../paymsg-specs")
    };

    // Validate based on message type
    let validation_result = match message_type {
        MessageType::Pacs008 => {
            let doc = paymsg_iso20022::parse_pacs008(content)?;
            let validator = paymsg_validate::MxSchemaValidator::new();
            paymsg_validate::Validator::validate(&validator, &doc)
        }
        MessageType::Pacs009 => {
            let doc = paymsg_iso20022::parse_pacs009(content)?;
            let validator = paymsg_validate::MxSchemaValidator::new();
            paymsg_validate::Validator::validate(&validator, &doc)
        }
        MessageType::Camt052 => {
            let doc = paymsg_iso20022::parse_camt052(content)?;
            let validator = paymsg_validate::MxSchemaValidator::new();
            paymsg_validate::Validator::validate(&validator, &doc)
        }
        MessageType::Camt053 => {
            let doc = paymsg_iso20022::parse_camt053(content)?;
            let validator = paymsg_validate::MxSchemaValidator::new();
            paymsg_validate::Validator::validate(&validator, &doc)
        }
        MessageType::Mt103 | MessageType::Mt202 | MessageType::Mt940 | MessageType::Mt942 => {
            let msg = paymsg_mt::MtMessage::parse(content)?;

            // Load SWIFT character sets
            let charsets = paymsg_validate::SwiftCharsets::load(&specs_path)
                .context("Failed to load SWIFT character sets")?;
            let charset_validator = paymsg_validate::SwiftCharsetValidator::new(charsets);

            let validator = paymsg_validate::MtSchemaValidator::load(&specs_path, charset_validator)
                .context("Failed to load MT schema validator")?;
            paymsg_validate::Validator::validate(&validator, &msg)
        }
    };

    // Report validation issues
    let errors: Vec<_> = validation_result
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();

    let warnings: Vec<_> = validation_result
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .collect();

    if !errors.is_empty() {
        eprintln!("\nValidation errors:");
        for issue in &errors {
            let field_path_str = issue.field_path.as_deref().unwrap_or("unknown");
            eprintln!("  [ERROR] {}: {}", field_path_str, issue.message);
            if let Some(ref suggestion) = issue.suggestion {
                eprintln!("    Suggestion: {}", suggestion);
            }
        }
        anyhow::bail!("{} validation error(s) found in translated message", errors.len());
    }

    if !warnings.is_empty() {
        eprintln!("\nValidation warnings:");
        for issue in &warnings {
            let field_path_str = issue.field_path.as_deref().unwrap_or("unknown");
            eprintln!("  [WARN] {}: {}", field_path_str, issue.message);
        }
    }

    info!("Validation passed ({} warnings)", warnings.len());
    Ok(())
}
