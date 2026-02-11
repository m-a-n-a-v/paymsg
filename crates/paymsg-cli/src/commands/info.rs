//! Info command implementation - show message details without full parsing.

use anyhow::{Context, Result};
use clap::Parser;
use log::debug;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
pub struct InfoArgs {
    /// Input file path (reads from stdin if not provided)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
}

/// Message information extracted without full parsing
#[derive(Debug, serde::Serialize)]
pub struct MessageInfo {
    /// Message format (MT or MX)
    pub format: String,
    /// Specific message type (e.g., MT103, pacs.008)
    pub message_type: String,
    /// Sender BIC (if present)
    pub sender: Option<String>,
    /// Receiver BIC (if present)
    pub receiver: Option<String>,
    /// Message reference (field 20 for MT, MsgId for MX)
    pub reference: Option<String>,
    /// Amount (if single amount message)
    pub amount: Option<String>,
    /// Currency code (if present)
    pub currency: Option<String>,
    /// Value date or creation date
    pub date: Option<String>,
}

/// Detect message format from content
fn detect_format(content: &str) -> Result<&'static str> {
    let trimmed = content.trim();
    if trimmed.starts_with("{1:") {
        Ok("MT")
    } else if trimmed.starts_with("<?xml") || trimmed.starts_with("<Document") {
        Ok("MX")
    } else {
        anyhow::bail!("Could not detect message format")
    }
}

/// Extract info from MT message without full parsing
fn extract_mt_info(content: &str) -> Result<MessageInfo> {
    let mut info = MessageInfo {
        format: "MT".to_string(),
        message_type: String::new(),
        sender: None,
        receiver: None,
        reference: None,
        amount: None,
        currency: None,
        date: None,
    };

    // Extract message type from block 2
    if let Some(block2_start) = content.find("{2:") {
        let block2_content = &content[block2_start..];
        if let Some(block2_end) = block2_content.find("}") {
            let block2 = &block2_content[3..block2_end];
            // Format: I103BANKBICXXXXN or O103...
            if block2.len() >= 4 {
                let msg_type = &block2[1..4];
                info.message_type = format!("MT{}", msg_type);

                // Extract sender/receiver from block 2
                if block2.starts_with('I') && block2.len() >= 15 {
                    // Input message: destination is in block 2
                    info.receiver = Some(block2[4..15].to_string());
                } else if block2.starts_with('O') && block2.len() >= 16 {
                    // Output message: sender is in block 2
                    info.sender = Some(block2[4..16].to_string());
                }
            }
        }
    }

    // Extract sender from block 1
    if let Some(block1_start) = content.find("{1:") {
        let block1_content = &content[block1_start..];
        if let Some(block1_end) = block1_content.find("}") {
            let block1 = &block1_content[3..block1_end];
            // Format: F01BANKBICAXXX0000000000
            if block1.len() >= 20 {
                let sender_bic = &block1[3..15];
                if info.sender.is_none() {
                    info.sender = Some(sender_bic.to_string());
                }
            }
        }
    }

    // Extract fields from block 4
    if let Some(block4_start) = content.find("{4:") {
        let block4_content = &content[block4_start + 3..];
        if let Some(block4_end) = block4_content.find("-}") {
            let block4 = &block4_content[..block4_end];

            // Extract field 20 (reference)
            if let Some(ref_start) = block4.find(":20:") {
                let ref_line = &block4[ref_start + 4..];
                if let Some(ref_end) = ref_line.find('\n') {
                    info.reference = Some(ref_line[..ref_end].trim().to_string());
                }
            }

            // Extract field 32A (value date, currency, amount)
            if let Some(f32a_start) = block4.find(":32A:") {
                let f32a_line = &block4[f32a_start + 5..];
                if let Some(f32a_end) = f32a_line.find('\n') {
                    let f32a_value = f32a_line[..f32a_end].trim();
                    // Format: YYMMDDCCCAMOUNT
                    if f32a_value.len() >= 12 {
                        info.date = Some(f32a_value[..6].to_string());
                        info.currency = Some(f32a_value[6..9].to_string());
                        info.amount = Some(f32a_value[9..].replace(',', "."));
                    }
                }
            }

            // For MT940/MT942, extract opening balance from :60F:
            if info.amount.is_none() {
                if let Some(f60f_start) = block4.find(":60F:") {
                    let f60f_line = &block4[f60f_start + 5..];
                    if let Some(f60f_end) = f60f_line.find('\n') {
                        let f60f_value = f60f_line[..f60f_end].trim();
                        // Format: D/C YYMMDDCCCAMOUNT
                        if f60f_value.len() >= 10 {
                            let sign = if f60f_value.starts_with('D') { "-" } else { "" };
                            info.date = Some(f60f_value[1..7].to_string());
                            info.currency = Some(f60f_value[7..10].to_string());
                            info.amount = Some(format!("{}{}", sign, f60f_value[10..].replace(',', ".")));
                        }
                    }
                }
            }

            // Extract field 25 (account) for MT940/MT942
            if let Some(f25_start) = block4.find(":25:") {
                let f25_line = &block4[f25_start + 4..];
                if let Some(f25_end) = f25_line.find('\n') {
                    if info.reference.is_none() {
                        info.reference = Some(f25_line[..f25_end].trim().to_string());
                    }
                }
            }
        }
    }

    Ok(info)
}

/// Extract info from MX message without full parsing
fn extract_mx_info(content: &str) -> Result<MessageInfo> {
    let mut info = MessageInfo {
        format: "MX".to_string(),
        message_type: String::new(),
        sender: None,
        receiver: None,
        reference: None,
        amount: None,
        currency: None,
        date: None,
    };

    // Detect message type from namespace or root element
    if content.contains("pacs.008.001") {
        info.message_type = "pacs.008".to_string();
    } else if content.contains("pacs.009.001") {
        info.message_type = "pacs.009".to_string();
    } else if content.contains("camt.052.001") {
        info.message_type = "camt.052".to_string();
    } else if content.contains("camt.053.001") {
        info.message_type = "camt.053".to_string();
    }

    // Extract MsgId
    if let Some(msgid_start) = content.find("<MsgId>") {
        let msgid_content = &content[msgid_start + 7..];
        if let Some(msgid_end) = msgid_content.find("</MsgId>") {
            info.reference = Some(msgid_content[..msgid_end].to_string());
        }
    }

    // Extract CreDtTm (creation date/time)
    if let Some(credt_start) = content.find("<CreDtTm>") {
        let credt_content = &content[credt_start + 9..];
        if let Some(credt_end) = credt_content.find("</CreDtTm>") {
            info.date = Some(credt_content[..credt_end].to_string());
        }
    }

    // Extract InstgAgt BIC (instructing agent - sender)
    if let Some(instg_start) = content.find("<InstgAgt>") {
        let instg_content = &content[instg_start..];
        if let Some(bicfi_start) = instg_content.find("<BICFI>") {
            let bicfi_content = &instg_content[bicfi_start + 7..];
            if let Some(bicfi_end) = bicfi_content.find("</BICFI>") {
                info.sender = Some(bicfi_content[..bicfi_end].to_string());
            }
        }
    }

    // Extract InstdAgt BIC (instructed agent - receiver)
    if let Some(instd_start) = content.find("<InstdAgt>") {
        let instd_content = &content[instd_start..];
        if let Some(bicfi_start) = instd_content.find("<BICFI>") {
            let bicfi_content = &instd_content[bicfi_start + 7..];
            if let Some(bicfi_end) = bicfi_content.find("</BICFI>") {
                info.receiver = Some(bicfi_content[..bicfi_end].to_string());
            }
        }
    }

    // Extract IntrBkSttlmAmt (amount and currency)
    if let Some(amt_start) = content.find("<IntrBkSttlmAmt") {
        let amt_content = &content[amt_start..];
        // Extract currency from Ccy attribute
        if let Some(ccy_start) = amt_content.find("Ccy=\"") {
            let ccy_content = &amt_content[ccy_start + 5..];
            if let Some(ccy_end) = ccy_content.find("\"") {
                info.currency = Some(ccy_content[..ccy_end].to_string());
            }
        }
        // Extract amount value
        if let Some(amt_value_start) = amt_content.find(">") {
            let amt_value_content = &amt_content[amt_value_start + 1..];
            if let Some(amt_value_end) = amt_value_content.find("</IntrBkSttlmAmt>") {
                info.amount = Some(amt_value_content[..amt_value_end].to_string());
            }
        }
    }

    // For camt messages, extract account ID as reference if MsgId not found
    if info.reference.is_none() {
        if let Some(acct_start) = content.find("<Acct>") {
            let acct_content = &content[acct_start..];
            if let Some(id_start) = acct_content.find("<Id>") {
                let id_content = &acct_content[id_start..];
                if let Some(iban_start) = id_content.find("<IBAN>") {
                    let iban_content = &id_content[iban_start + 6..];
                    if let Some(iban_end) = iban_content.find("</IBAN>") {
                        info.reference = Some(iban_content[..iban_end].to_string());
                    }
                }
            }
        }
    }

    Ok(info)
}

/// Execute the info command
pub fn execute(args: InfoArgs) -> Result<()> {
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

    // Detect format
    let format = detect_format(&content)?;
    debug!("Detected format: {}", format);

    // Extract info based on format
    let info = match format {
        "MT" => extract_mt_info(&content)?,
        "MX" => extract_mx_info(&content)?,
        _ => anyhow::bail!("Unknown format: {}", format),
    };

    // Output as JSON
    let json = serde_json::to_string_pretty(&info)
        .context("Failed to serialize message info to JSON")?;
    println!("{}", json);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mt_format() {
        let mt_content = "{1:F01BANKBICAXXX0000000000}";
        assert_eq!(detect_format(mt_content).unwrap(), "MT");
    }

    #[test]
    fn test_detect_mx_format() {
        let mx_content = "<?xml version=\"1.0\"?>\n<Document>";
        assert_eq!(detect_format(mx_content).unwrap(), "MX");
    }

    #[test]
    fn test_extract_mt103_info() {
        let mt103 = r#"{1:F01BANKUS33XXX0000000000}{2:I103BANKGB2LXXXXN}{4:
:20:REF123456
:32A:260210EUR1000,50
-}"#;
        let info = extract_mt_info(mt103).unwrap();
        assert_eq!(info.message_type, "MT103");
        assert_eq!(info.reference, Some("REF123456".to_string()));
        assert_eq!(info.currency, Some("EUR".to_string()));
        assert_eq!(info.amount, Some("1000.50".to_string()));
        assert_eq!(info.date, Some("260210".to_string()));
    }

    #[test]
    fn test_extract_pacs008_info() {
        let pacs008 = r#"<?xml version="1.0"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSG123456</MsgId>
      <CreDtTm>2026-02-10T12:00:00</CreDtTm>
      <InstgAgt><FinInstnId><BICFI>BANKUS33XXX</BICFI></FinInstnId></InstgAgt>
      <InstdAgt><FinInstnId><BICFI>BANKGB2LXXX</BICFI></FinInstnId></InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <IntrBkSttlmAmt Ccy="EUR">1000.50</IntrBkSttlmAmt>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;
        let info = extract_mx_info(pacs008).unwrap();
        assert_eq!(info.message_type, "pacs.008");
        assert_eq!(info.reference, Some("MSG123456".to_string()));
        assert_eq!(info.currency, Some("EUR".to_string()));
        assert_eq!(info.amount, Some("1000.50".to_string()));
        assert_eq!(info.sender, Some("BANKUS33XXX".to_string()));
        assert_eq!(info.receiver, Some("BANKGB2LXXX".to_string()));
    }
}
