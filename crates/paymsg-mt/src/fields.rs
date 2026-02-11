//! MT message field structures and parsing.
//!
//! This module provides field-level parsing for SWIFT MT messages (Block 4).
//! It handles:
//! - Tag:value field parsing with multi-line support
//! - Repeating fields (e.g., multiple :23E: entries)
//! - Compound field subfield extraction (e.g., field 32A: date + currency + amount)
//! - Field specification loading from JSON

use paymsg_core::{Date, PaymsgError};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

/// A single MT field with tag and value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MtField {
    /// Field tag (e.g., "20", "32A", "50K")
    pub tag: String,
    /// Raw field value (may be multi-line)
    pub value: String,
    /// Parsed subfields for compound fields (e.g., 32A has date, currency, amount)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub subfields: HashMap<String, String>,
}

impl MtField {
    /// Create a new field with tag and value.
    pub fn new(tag: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            value: value.into(),
            subfields: HashMap::new(),
        }
    }

    /// Create a new field with parsed subfields.
    pub fn with_subfields(
        tag: impl Into<String>,
        value: impl Into<String>,
        subfields: HashMap<String, String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            value: value.into(),
            subfields,
        }
    }

    /// Get the base tag without option letter (e.g., "50" from "50K").
    pub fn base_tag(&self) -> &str {
        // Find first non-digit character
        let base_len = self
            .tag
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .count();
        &self.tag[..base_len]
    }

    /// Get the option letter if present (e.g., "K" from "50K").
    pub fn option(&self) -> Option<char> {
        self.tag.chars().find(|c| !c.is_ascii_digit())
    }

    /// Parse compound field 32A: Value Date, Currency Code, Amount.
    ///
    /// Format: YYMMDDCCCAMOUNT where:
    /// - YYMMDD: Date (6 digits)
    /// - CCC: Currency code (3 letters)
    /// - AMOUNT: Amount with comma decimal separator
    pub fn parse_field_32a(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        // Expected format: 6 digits + 3 letters + amount
        if value.len() < 10 {
            return Err(PaymsgError::ParseError(format!(
                "Field 32A too short: expected at least 10 chars, got {}",
                value.len()
            )));
        }

        // Extract date (YYMMDD)
        let date_str = &value[0..6];
        self.subfields
            .insert("date".to_string(), date_str.to_string());

        // Extract currency (3 letters)
        let currency_str = &value[6..9];
        self.subfields
            .insert("currency".to_string(), currency_str.to_string());

        // Extract amount (rest of the string)
        let amount_str = &value[9..];
        self.subfields
            .insert("amount".to_string(), amount_str.to_string());

        Ok(())
    }

    /// Parse compound field 33B: Currency/Instructed Amount.
    ///
    /// Format: CCCAMOUNT where:
    /// - CCC: Currency code (3 letters)
    /// - AMOUNT: Amount with comma decimal separator
    pub fn parse_field_33b(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if value.len() < 4 {
            return Err(PaymsgError::ParseError(format!(
                "Field 33B too short: expected at least 4 chars, got {}",
                value.len()
            )));
        }

        // Extract currency (3 letters)
        let currency_str = &value[0..3];
        self.subfields
            .insert("currency".to_string(), currency_str.to_string());

        // Extract amount (rest of the string)
        let amount_str = &value[3..];
        self.subfields
            .insert("amount".to_string(), amount_str.to_string());

        Ok(())
    }

    /// Parse field 50K: Ordering Customer - Name & Address.
    ///
    /// Format: Optional account line (starting with /), then up to 4 lines of name/address.
    pub fn parse_field_50k(&mut self) -> Result<(), PaymsgError> {
        let lines: Vec<&str> = self.value.lines().collect();

        if lines.is_empty() {
            return Ok(());
        }

        let mut account = None;
        let mut name_address_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if this is an account line (starts with /)
            if trimmed.starts_with('/') && account.is_none() {
                account = Some(trimmed.to_string());
            } else {
                name_address_lines.push(trimmed);
            }
        }

        if let Some(acc) = account {
            self.subfields.insert("account".to_string(), acc);
        }

        if !name_address_lines.is_empty() {
            self.subfields
                .insert("name_address".to_string(), name_address_lines.join("\n"));
        }

        Ok(())
    }

    /// Parse field 59: Beneficiary Customer.
    ///
    /// Format: Optional account line (starting with /), then up to 4 lines of name/address.
    /// Same structure as 50K.
    pub fn parse_field_59(&mut self) -> Result<(), PaymsgError> {
        // Reuse the 50K parser logic
        self.parse_field_50k()
    }

    /// Parse field 52A, 56A, 57A, 58A: Institution with optional account and BIC.
    ///
    /// Format: Optional account line (starting with /), then BIC on next line.
    pub fn parse_field_institution_a(&mut self) -> Result<(), PaymsgError> {
        let lines: Vec<&str> = self.value.lines().map(|l| l.trim()).collect();

        if lines.is_empty() {
            return Ok(());
        }

        let mut account = None;
        let mut bic = None;

        for line in lines {
            if line.is_empty() {
                continue;
            }

            // Check if this is an account line (starts with /)
            if line.starts_with('/') && account.is_none() {
                account = Some(line.to_string());
            } else if bic.is_none() {
                // First non-account line is the BIC
                bic = Some(line.to_string());
            }
        }

        if let Some(acc) = account {
            self.subfields.insert("account".to_string(), acc);
        }

        if let Some(bic_val) = bic {
            self.subfields.insert("bic".to_string(), bic_val);
        }

        Ok(())
    }

    /// Parse field 70: Remittance Information.
    ///
    /// Free-form text, up to 4 lines of 35 characters.
    /// No special parsing needed, but we track it as a single text block.
    pub fn parse_field_70(&mut self) -> Result<(), PaymsgError> {
        // Just store as-is, no subfield extraction needed
        self.subfields
            .insert("text".to_string(), self.value.clone());
        Ok(())
    }

    /// Parse field 72: Sender to Receiver Information.
    ///
    /// Free-form text with optional structured codes like /INS/, /ACC/, /BNF/.
    pub fn parse_field_72(&mut self) -> Result<(), PaymsgError> {
        // Just store as-is for now; advanced parsing can extract /CODE/ patterns later
        self.subfields
            .insert("text".to_string(), self.value.clone());
        Ok(())
    }

    /// Parse subfields based on the field tag.
    ///
    /// This automatically detects compound fields and extracts subfields.
    pub fn parse_subfields(&mut self) -> Result<(), PaymsgError> {
        match self.tag.as_str() {
            "32A" => self.parse_field_32a(),
            "33B" => self.parse_field_33b(),
            "50K" => self.parse_field_50k(),
            "59" | "59A" => self.parse_field_59(),
            "52A" | "56A" | "57A" | "58A" => self.parse_field_institution_a(),
            "70" => self.parse_field_70(),
            "72" => self.parse_field_72(),
            _ => Ok(()), // No special parsing for other fields
        }
    }
}

/// Parse fields from Block 4 text content.
///
/// Handles:
/// - `:TAG:VALUE` format where TAG is 2 digits optionally followed by a letter
/// - Multi-line values (lines without tag prefix belong to previous field)
/// - Repeating fields
///
/// Returns a vector of parsed fields in order of appearance.
pub fn parse_block4_fields(content: &str) -> Result<Vec<MtField>, PaymsgError> {
    let mut fields = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut current_value = String::new();

    // Regex to match field tags: :NN[a]:
    let tag_regex = Regex::new(r"^:(\d{2,3}[A-Z]?):").map_err(|e| {
        PaymsgError::ParseError(format!("Failed to compile field tag regex: {}", e))
    })?;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this line starts a new field
        if let Some(captures) = tag_regex.captures(trimmed) {
            // Save the previous field if any
            if let Some(tag) = current_tag.take() {
                let mut field = MtField::new(tag, current_value.trim());
                field.parse_subfields()?;
                fields.push(field);
                current_value.clear();
            }

            // Extract the new tag and value
            let tag = captures[1].to_string();
            let value_start = captures[0].len();
            let value = if value_start < trimmed.len() {
                &trimmed[value_start..]
            } else {
                ""
            };

            current_tag = Some(tag);
            current_value = value.to_string();
        } else if current_tag.is_some() {
            // This is a continuation line for the current field
            if !current_value.is_empty() {
                current_value.push('\n');
            }
            current_value.push_str(trimmed);
        }
        // Else: line before first field or empty line, ignore
    }

    // Save the last field
    if let Some(tag) = current_tag {
        let mut field = MtField::new(tag, current_value.trim());
        field.parse_subfields()?;
        fields.push(field);
    }

    Ok(fields)
}

/// MT field specification from JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtFieldSpec {
    /// Field tag (e.g., "20", "32A")
    pub tag: String,
    /// Field name
    pub name: String,
    /// Status: M (mandatory), O (optional)
    pub status: String,
    /// Options (e.g., ["A", "K"] for fields with variants)
    #[serde(default)]
    pub options: Vec<String>,
    /// Maximum length
    #[serde(default)]
    pub max_length: usize,
    /// Format pattern regex
    #[serde(default)]
    pub format_pattern: Option<String>,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Subfield specifications
    #[serde(default)]
    pub subfields: Option<Vec<MtSubfieldSpec>>,
}

/// MT subfield specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtSubfieldSpec {
    /// Subfield name
    pub name: String,
    /// Position (e.g., "1-6" or "7-9")
    #[serde(default)]
    pub position: String,
    /// Format (e.g., "YYMMDD", "3!a")
    #[serde(default)]
    pub format: String,
    /// Description
    #[serde(default)]
    pub description: String,
}

/// MT message type specification from JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtMessageSpec {
    /// Message type (e.g., "MT103")
    pub message_type: String,
    /// Message name
    pub name: String,
    /// Description
    pub description: String,
    /// Field specifications
    pub fields: Vec<MtFieldSpec>,
}

/// Load MT message specification from JSON file.
pub fn load_mt_spec(spec_path: &Path) -> Result<MtMessageSpec, PaymsgError> {
    let content = std::fs::read_to_string(spec_path).map_err(|e| {
        PaymsgError::SpecLoadError {
            file: spec_path.display().to_string(),
            reason: e.to_string(),
        }
    })?;

    let spec: MtMessageSpec = serde_json::from_str(&content).map_err(|e| {
        PaymsgError::SpecLoadError {
            file: spec_path.display().to_string(),
            reason: format!("JSON parse error: {}", e),
        }
    })?;

    Ok(spec)
}

/// Load all MT specs from a directory.
///
/// Loads MT103, MT202, MT940, MT942 specs.
pub fn load_all_mt_specs(specs_dir: &Path) -> Result<HashMap<String, MtMessageSpec>, PaymsgError> {
    let mut specs = HashMap::new();

    for msg_type in &["mt103", "mt202", "mt940", "mt942"] {
        let spec_path = specs_dir.join("mt-specs").join(format!("{}.json", msg_type));
        if spec_path.exists() {
            let spec = load_mt_spec(&spec_path)?;
            specs.insert(spec.message_type.clone(), spec);
        }
    }

    Ok(specs)
}

/// Helper: Parse MT amount (with comma decimal separator) to Decimal.
pub fn parse_mt_amount(amount_str: &str) -> Result<Decimal, PaymsgError> {
    // Replace comma with period for Decimal parsing
    let normalized = amount_str.replace(',', ".");
    Decimal::from_str(&normalized).map_err(|e| {
        PaymsgError::ParseError(format!("Invalid amount '{}': {}", amount_str, e))
    })
}

/// Helper: Parse MT date in YYMMDD format to Date.
pub fn parse_mt_date_yymmdd(date_str: &str) -> Result<Date, PaymsgError> {
    if date_str.len() != 6 {
        return Err(PaymsgError::ParseError(format!(
            "Invalid YYMMDD date: expected 6 chars, got {}",
            date_str.len()
        )));
    }

    // Assume 20XX century for YY
    let year = format!("20{}", &date_str[0..2]);
    let month = &date_str[2..4];
    let day = &date_str[4..6];

    let full_date = format!("{}-{}-{}", year, month, day);
    Date::from_iso8601(&full_date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_field() {
        let content = ":20:TESTREF12345\n:23B:CRED";
        let fields = parse_block4_fields(content).unwrap();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].tag, "20");
        assert_eq!(fields[0].value, "TESTREF12345");
        assert_eq!(fields[1].tag, "23B");
        assert_eq!(fields[1].value, "CRED");
    }

    #[test]
    fn test_parse_multiline_field() {
        let content = ":50K:/DE89370400440532013000\nHANS MUELLER\nHAUPTSTRASSE 1\n60313 FRANKFURT";
        let fields = parse_block4_fields(content).unwrap();

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag, "50K");
        assert!(fields[0].value.contains("HANS MUELLER"));
        assert!(fields[0].value.contains("HAUPTSTRASSE 1"));
    }

    #[test]
    fn test_parse_field_32a() {
        let mut field = MtField::new("32A", "260210EUR1234,56");
        field.parse_field_32a().unwrap();

        assert_eq!(field.subfields.get("date").unwrap(), "260210");
        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "1234,56");
    }

    #[test]
    fn test_parse_field_33b() {
        let mut field = MtField::new("33B", "USD11500,00");
        field.parse_field_33b().unwrap();

        assert_eq!(field.subfields.get("currency").unwrap(), "USD");
        assert_eq!(field.subfields.get("amount").unwrap(), "11500,00");
    }

    #[test]
    fn test_parse_field_50k() {
        let value = "/DE89370400440532013000\nMUELLER INTERNATIONAL GMBH\nHAUPTSTRASSE 123\n60313 FRANKFURT";
        let mut field = MtField::new("50K", value);
        field.parse_field_50k().unwrap();

        assert_eq!(
            field.subfields.get("account").unwrap(),
            "/DE89370400440532013000"
        );
        assert!(field
            .subfields
            .get("name_address")
            .unwrap()
            .contains("MUELLER INTERNATIONAL GMBH"));
    }

    #[test]
    fn test_parse_field_59() {
        let value = "/FR1420041010050500013M02606\nJEAN DUPONT\n1 RUE DE LA PAIX\n75001 PARIS";
        let mut field = MtField::new("59", value);
        field.parse_field_59().unwrap();

        assert_eq!(
            field.subfields.get("account").unwrap(),
            "/FR1420041010050500013M02606"
        );
        assert!(field
            .subfields
            .get("name_address")
            .unwrap()
            .contains("JEAN DUPONT"));
    }

    #[test]
    fn test_parse_field_institution_a() {
        let value = "/987654321\nBNPAFRPPXXX";
        let mut field = MtField::new("57A", value);
        field.parse_field_institution_a().unwrap();

        assert_eq!(field.subfields.get("account").unwrap(), "/987654321");
        assert_eq!(field.subfields.get("bic").unwrap(), "BNPAFRPPXXX");
    }

    #[test]
    fn test_parse_mt_amount() {
        assert_eq!(parse_mt_amount("1234,56").unwrap().to_string(), "1234.56");
        assert_eq!(parse_mt_amount("10000,00").unwrap().to_string(), "10000.00");
        assert_eq!(parse_mt_amount("50000").unwrap().to_string(), "50000");
    }

    #[test]
    fn test_parse_mt_date_yymmdd() {
        let date = parse_mt_date_yymmdd("260210").unwrap();
        assert_eq!(date.to_iso8601(), "2026-02-10");

        let date2 = parse_mt_date_yymmdd("231115").unwrap();
        assert_eq!(date2.to_iso8601(), "2023-11-15");
    }

    #[test]
    fn test_field_base_tag_and_option() {
        let field = MtField::new("50K", "test");
        assert_eq!(field.base_tag(), "50");
        assert_eq!(field.option(), Some('K'));

        let field2 = MtField::new("20", "test");
        assert_eq!(field2.base_tag(), "20");
        assert_eq!(field2.option(), None);
    }

    #[test]
    fn test_parse_full_mt103_block4() {
        let content = ":20:FULLREF98765
:13C:/SNDTIME/1615+0100
:23B:CRED
:32A:260215EUR10000,00
:50K:/DE89370400440532013000
MUELLER INTERNATIONAL GMBH
HAUPTSTRASSE 123
:59:/FR1420041010050500013M02606
DUPONT TRADING SARL
1 RUE DE LA PAIX
:71A:SHA";

        let fields = parse_block4_fields(content).unwrap();

        assert!(fields.len() >= 6);

        // Find field 32A and check subfields
        let field_32a = fields.iter().find(|f| f.tag == "32A").unwrap();
        assert_eq!(field_32a.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field_32a.subfields.get("date").unwrap(), "260215");
        assert_eq!(field_32a.subfields.get("amount").unwrap(), "10000,00");

        // Find field 50K
        let field_50k = fields.iter().find(|f| f.tag == "50K").unwrap();
        assert!(field_50k.subfields.contains_key("account"));
        assert!(field_50k.subfields.contains_key("name_address"));
    }

    #[test]
    fn test_parse_repeating_fields() {
        let content = ":23E:CHQB\n:23E:HOLD\n:23E:INTC";
        let fields = parse_block4_fields(content).unwrap();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].tag, "23E");
        assert_eq!(fields[0].value, "CHQB");
        assert_eq!(fields[1].tag, "23E");
        assert_eq!(fields[1].value, "HOLD");
        assert_eq!(fields[2].tag, "23E");
        assert_eq!(fields[2].value, "INTC");
    }
}
