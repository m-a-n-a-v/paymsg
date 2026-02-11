//! Parse command implementation.

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use paymsg_core::MessageType;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
pub struct ParseArgs {
    /// Input file path (reads from stdin if not provided)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Force message format (mt or mx). Auto-detect if not specified.
    #[arg(short, long, value_name = "FORMAT")]
    format: Option<String>,

    /// Output file path (writes to stdout if not provided)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Pretty-print JSON output
    #[arg(short, long)]
    pretty: bool,
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

    anyhow::bail!("Could not auto-detect message format. Use --format to specify explicitly.");
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

/// Parsed message representation for JSON output
#[derive(Debug, serde::Serialize)]
#[serde(tag = "format")]
enum ParsedMessage {
    #[serde(rename = "mt")]
    Mt {
        message_type: MessageType,
        #[serde(flatten)]
        message: Box<paymsg_mt::MtMessage>,
    },
    #[serde(rename = "mx")]
    Mx {
        message_type: MessageType,
        #[serde(flatten)]
        message: Box<MxMessage>,
    },
}

/// Wrapper for MX message types
#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
enum MxMessage {
    Pacs008(paymsg_iso20022::pacs008::Document),
    Pacs009(paymsg_iso20022::pacs009::Document),
    Camt052(paymsg_iso20022::camt052::Document),
    Camt053(paymsg_iso20022::camt053::Document),
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

/// Parse MT message
fn parse_mt(content: &str) -> Result<ParsedMessage> {
    let message = paymsg_mt::MtMessage::parse(content)
        .context("Failed to parse MT message")?;

    let message_type = detect_mt_type(&message)?;
    info!("Parsed {:?} message", message_type);

    Ok(ParsedMessage::Mt {
        message_type,
        message: Box::new(message),
    })
}

/// Parse MX message
fn parse_mx(content: &str) -> Result<ParsedMessage> {
    let message_type = detect_mx_type(content)?;

    let message = match message_type {
        MessageType::Pacs008 => {
            let doc = paymsg_iso20022::parse_pacs008(content)
                .context("Failed to parse pacs.008 message")?;
            MxMessage::Pacs008(doc)
        }
        MessageType::Pacs009 => {
            let doc = paymsg_iso20022::parse_pacs009(content)
                .context("Failed to parse pacs.009 message")?;
            MxMessage::Pacs009(doc)
        }
        MessageType::Camt052 => {
            let doc = paymsg_iso20022::parse_camt052(content)
                .context("Failed to parse camt.052 message")?;
            MxMessage::Camt052(doc)
        }
        MessageType::Camt053 => {
            let doc = paymsg_iso20022::parse_camt053(content)
                .context("Failed to parse camt.053 message")?;
            MxMessage::Camt053(doc)
        }
        _ => anyhow::bail!("Invalid MX message type: {:?}", message_type),
    };

    info!("Parsed {:?} message", message_type);

    Ok(ParsedMessage::Mx {
        message_type,
        message: Box::new(message),
    })
}

/// Execute the parse command
pub fn execute(args: ParseArgs) -> Result<()> {
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
    let format = if let Some(ref fmt_str) = args.format {
        fmt_str.parse::<MessageFormat>()?
    } else {
        detect_format(&content)?
    };

    debug!("Using format: {:?}", format);

    // Parse message based on format
    let parsed = match format {
        MessageFormat::Mt => parse_mt(&content)?,
        MessageFormat::Mx => parse_mx(&content)?,
    };

    // Serialize to JSON
    let json_output = if args.pretty {
        serde_json::to_string_pretty(&parsed)
            .context("Failed to serialize to JSON")?
    } else {
        serde_json::to_string(&parsed)
            .context("Failed to serialize to JSON")?
    };

    // Write output
    if let Some(ref path) = args.output {
        debug!("Writing to file: {}", path.display());
        fs::write(path, json_output)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
        info!("Output written to: {}", path.display());
    } else {
        println!("{}", json_output);
    }

    Ok(())
}
