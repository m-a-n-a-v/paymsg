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

    /// Parse field 25P: Account Identification with BIC.
    ///
    /// Format: BIC/Account where BIC is 8 or 11 characters and Account is up to 35 characters.
    pub fn parse_field_25p(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        // Find the slash separator
        if let Some(slash_pos) = value.find('/') {
            let bic_str = &value[..slash_pos];
            let account_str = &value[slash_pos + 1..];

            self.subfields.insert("bic".to_string(), bic_str.to_string());
            self.subfields
                .insert("account".to_string(), account_str.to_string());
        } else {
            return Err(PaymsgError::ParseError(
                "Field 25P must contain BIC/Account format with slash separator".to_string(),
            ));
        }

        Ok(())
    }

    /// Parse field 28C: Statement Number/Sequence Number.
    ///
    /// Format: StatementNumber[/SequenceNumber] where both are up to 5 digits.
    pub fn parse_field_28c(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if let Some(slash_pos) = value.find('/') {
            let stmt_num = &value[..slash_pos];
            let seq_num = &value[slash_pos + 1..];

            self.subfields
                .insert("statement_number".to_string(), stmt_num.to_string());
            self.subfields
                .insert("sequence_number".to_string(), seq_num.to_string());
        } else {
            // No sequence number, just statement number
            self.subfields
                .insert("statement_number".to_string(), value.to_string());
        }

        Ok(())
    }

    /// Parse balance fields: 60F, 60M, 62F, 62M, 64, 65.
    ///
    /// Format: [D|C]YYMMDDCCCAMOUNT where:
    /// - D/C: Debit or Credit indicator (1 char)
    /// - YYMMDD: Date (6 digits)
    /// - CCC: Currency code (3 letters)
    /// - AMOUNT: Amount with comma decimal separator
    pub fn parse_balance_field(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if value.len() < 11 {
            return Err(PaymsgError::ParseError(format!(
                "Balance field {} too short: expected at least 11 chars, got {}",
                self.tag,
                value.len()
            )));
        }

        // Extract debit/credit indicator
        let dc_mark = &value[0..1];
        self.subfields.insert("dc_mark".to_string(), dc_mark.to_string());

        // Extract date (YYMMDD)
        let date_str = &value[1..7];
        self.subfields.insert("date".to_string(), date_str.to_string());

        // Extract currency (3 letters)
        let currency_str = &value[7..10];
        self.subfields
            .insert("currency".to_string(), currency_str.to_string());

        // Extract amount (rest of the string)
        let amount_str = &value[10..];
        self.subfields
            .insert("amount".to_string(), amount_str.to_string());

        Ok(())
    }

    /// Parse field 61: Statement Line.
    ///
    /// Complex format: YYMMDD[MMDD][D|C|RD|RC][FundsCode]AMOUNT[Type][CustomerRef][//BankRef][\nSupplementary]
    ///
    /// This is the most complex MT field with many optional components.
    pub fn parse_field_61(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if value.len() < 10 {
            return Err(PaymsgError::ParseError(format!(
                "Field 61 too short: expected at least 10 chars, got {}",
                value.len()
            )));
        }

        let mut pos = 0;

        // Check if there's a continuation line (supplementary details)
        let (main_line, _supplementary) = if let Some(newline_pos) = value.find('\n') {
            let supp = value[newline_pos + 1..].trim().to_string();
            if !supp.is_empty() {
                self.subfields
                    .insert("supplementary_details".to_string(), supp.clone());
            }
            (&value[..newline_pos], Some(supp))
        } else {
            (value, None)
        };

        // Extract value date (YYMMDD) - 6 digits
        if main_line.len() < 6 {
            return Err(PaymsgError::ParseError(
                "Field 61: insufficient length for value date".to_string(),
            ));
        }
        let value_date = &main_line[pos..pos + 6];
        self.subfields
            .insert("value_date".to_string(), value_date.to_string());
        pos += 6;

        // Check for optional entry date (MMDD) - 4 digits
        // Entry date is present if:
        // 1. Next 4 chars are all digits AND
        // 2. The char after those 4 digits is a D/C mark (D, C, or R for reversal)
        // This prevents misinterpreting amounts starting with 0 as entry dates
        if pos + 4 <= main_line.len()
            && main_line[pos..pos + 4].chars().all(|c| c.is_ascii_digit())
            && pos + 4 < main_line.len()
        {
            let char_after = main_line.chars().nth(pos + 4).unwrap();
            if char_after == 'D' || char_after == 'C' || char_after == 'R' {
                let entry_date = &main_line[pos..pos + 4];
                self.subfields
                    .insert("entry_date".to_string(), entry_date.to_string());
                pos += 4;
            }
        }

        // Extract D/C mark - can be D, C, RD, or RC
        if pos >= main_line.len() {
            return Err(PaymsgError::ParseError(
                "Field 61: missing debit/credit mark".to_string(),
            ));
        }

        let dc_mark = if pos + 2 <= main_line.len()
            && (main_line[pos..pos + 2] == *"RD" || main_line[pos..pos + 2] == *"RC")
        {
            let mark = &main_line[pos..pos + 2];
            pos += 2;
            mark
        } else {
            let mark = &main_line[pos..pos + 1];
            pos += 1;
            mark
        };
        self.subfields
            .insert("dc_mark".to_string(), dc_mark.to_string());

        // Check for optional funds code (single letter after D/C mark, before amount)
        // Funds code is present if next char is a letter (not a digit)
        if pos < main_line.len()
            && main_line
                .chars()
                .nth(pos)
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
        {
            let funds_code = &main_line[pos..pos + 1];
            self.subfields
                .insert("funds_code".to_string(), funds_code.to_string());
            pos += 1;
        }

        // Extract amount - continues until we hit a letter (transaction type)
        // Amount can contain digits and comma
        let amount_start = pos;
        while pos < main_line.len() {
            let ch = main_line.chars().nth(pos).unwrap();
            if ch.is_ascii_digit() || ch == ',' {
                pos += 1;
            } else {
                break;
            }
        }

        if pos == amount_start {
            return Err(PaymsgError::ParseError(
                "Field 61: missing amount".to_string(),
            ));
        }

        let amount = &main_line[amount_start..pos];
        self.subfields
            .insert("amount".to_string(), amount.to_string());

        // Extract transaction type (1 letter + 3 alphanumeric)
        if pos + 4 <= main_line.len() {
            let trans_type = &main_line[pos..pos + 4];
            self.subfields
                .insert("transaction_type".to_string(), trans_type.to_string());
            pos += 4;
        }

        // Extract customer reference - everything until "//" or end of line
        let remaining = &main_line[pos..];
        if let Some(slash_pos) = remaining.find("//") {
            let customer_ref = &remaining[..slash_pos];
            if !customer_ref.is_empty() {
                self.subfields
                    .insert("customer_reference".to_string(), customer_ref.to_string());
            }

            // Extract bank reference (after //)
            let bank_ref = &remaining[slash_pos + 2..];
            if !bank_ref.is_empty() {
                self.subfields
                    .insert("bank_reference".to_string(), bank_ref.to_string());
            }
        } else if !remaining.is_empty() {
            // No bank reference, just customer reference
            self.subfields
                .insert("customer_reference".to_string(), remaining.to_string());
        }

        Ok(())
    }

    /// Parse field 86: Information to Account Owner.
    ///
    /// Free-form text, often with structured codes like ?20, ?21, ?32, ?60.
    /// Can be up to 6 lines of 65 characters (390 chars total).
    pub fn parse_field_86(&mut self) -> Result<(), PaymsgError> {
        // Store as text, optionally parse structured codes
        self.subfields
            .insert("text".to_string(), self.value.clone());

        // If the field contains ?NN codes, parse them
        if self.value.contains('?') {
            let mut structured = HashMap::new();
            let lines = self.value.lines();
            let mut current_code: Option<String> = None;
            let mut current_value = String::new();

            for line in lines {
                let trimmed = line.trim();

                // Check if line starts with ?NN code
                if trimmed.starts_with('?') && trimmed.len() >= 3 {
                    // Save previous code if any
                    if let Some(code) = current_code.take() {
                        structured.insert(code, current_value.trim().to_string());
                        current_value.clear();
                    }

                    // Extract new code (2 digits after ?)
                    let code = trimmed[1..3].to_string();
                    let value = if trimmed.len() > 3 {
                        &trimmed[3..]
                    } else {
                        ""
                    };

                    current_code = Some(code);
                    current_value = value.to_string();
                } else if current_code.is_some() {
                    // Continuation line for current code
                    if !current_value.is_empty() {
                        current_value.push(' ');
                    }
                    current_value.push_str(trimmed);
                } else {
                    // Text before first code or unstructured text
                    if !current_value.is_empty() {
                        current_value.push('\n');
                    }
                    current_value.push_str(trimmed);
                }
            }

            // Save last code if any
            if let Some(code) = current_code {
                structured.insert(code, current_value.trim().to_string());
            }

            // Store structured codes as subfield if we found any
            if !structured.is_empty() {
                for (code, value) in structured {
                    self.subfields.insert(format!("code_{}", code), value);
                }
            }
        }

        Ok(())
    }

    /// Parse field 34F: Floor Limit Indicator (MT942).
    ///
    /// Format: CCCAMOUNT where:
    /// - CCC: Currency code (3 letters)
    /// - AMOUNT: Amount with comma decimal separator
    pub fn parse_field_34f(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if value.len() < 4 {
            return Err(PaymsgError::ParseError(format!(
                "Field 34F too short: expected at least 4 chars, got {}",
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

    /// Parse field 13D: Date/Time Indication (MT942).
    ///
    /// Format: YYMMDD+HHMM
    pub fn parse_field_13d(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        // Expected format: 6 digits + plus sign + 4 digits
        if value.len() < 11 {
            return Err(PaymsgError::ParseError(format!(
                "Field 13D too short: expected at least 11 chars (YYMMDD+HHMM), got {}",
                value.len()
            )));
        }

        // Find the + separator
        if let Some(plus_pos) = value.find('+') {
            let date_str = &value[..plus_pos];
            let time_str = &value[plus_pos + 1..];

            self.subfields.insert("date".to_string(), date_str.to_string());
            self.subfields.insert("time".to_string(), time_str.to_string());
        } else {
            return Err(PaymsgError::ParseError(
                "Field 13D must contain date+time format with + separator".to_string(),
            ));
        }

        Ok(())
    }

    /// Parse field 90D/90C: Number and Sum of Entries (MT942).
    ///
    /// Format: NCCCAMOUNT where:
    /// - N: Number of entries (1-5 digits)
    /// - CCC: Currency code (3 letters)
    /// - AMOUNT: Total amount with comma decimal separator
    pub fn parse_field_90(&mut self) -> Result<(), PaymsgError> {
        let value = self.value.trim();

        if value.len() < 4 {
            return Err(PaymsgError::ParseError(format!(
                "Field {} too short: expected at least 4 chars, got {}",
                self.tag,
                value.len()
            )));
        }

        // Find where the currency code starts (first letter after digits)
        let mut currency_start = 0;
        for (i, ch) in value.chars().enumerate() {
            if ch.is_ascii_alphabetic() {
                currency_start = i;
                break;
            }
        }

        if currency_start == 0 {
            return Err(PaymsgError::ParseError(format!(
                "Field {}: missing currency code",
                self.tag
            )));
        }

        // Extract number of entries
        let num_entries = &value[..currency_start];
        self.subfields
            .insert("number_of_entries".to_string(), num_entries.to_string());

        // Extract currency (3 letters)
        if currency_start + 3 > value.len() {
            return Err(PaymsgError::ParseError(format!(
                "Field {}: insufficient length for currency code",
                self.tag
            )));
        }
        let currency_str = &value[currency_start..currency_start + 3];
        self.subfields
            .insert("currency".to_string(), currency_str.to_string());

        // Extract amount (rest of the string)
        let amount_str = &value[currency_start + 3..];
        self.subfields
            .insert("amount".to_string(), amount_str.to_string());

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
            // MT940/MT942 specific fields
            "25P" => self.parse_field_25p(),
            "28C" => self.parse_field_28c(),
            "60F" | "60M" | "62F" | "62M" | "64" | "65" => self.parse_balance_field(),
            "61" => self.parse_field_61(),
            "86" => self.parse_field_86(),
            "34F" => self.parse_field_34f(),
            "13D" => self.parse_field_13d(),
            "90D" | "90C" => self.parse_field_90(),
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

    // MT940/MT942 specific field tests

    #[test]
    fn test_parse_field_25p() {
        let mut field = MtField::new("25P", "BNPAFRPP/FR7630006000011234567890189");
        field.parse_field_25p().unwrap();

        assert_eq!(field.subfields.get("bic").unwrap(), "BNPAFRPP");
        assert_eq!(
            field.subfields.get("account").unwrap(),
            "FR7630006000011234567890189"
        );
    }

    #[test]
    fn test_parse_field_28c() {
        let mut field = MtField::new("28C", "235/1");
        field.parse_field_28c().unwrap();

        assert_eq!(field.subfields.get("statement_number").unwrap(), "235");
        assert_eq!(field.subfields.get("sequence_number").unwrap(), "1");

        // Test without sequence number
        let mut field2 = MtField::new("28C", "127");
        field2.parse_field_28c().unwrap();

        assert_eq!(field2.subfields.get("statement_number").unwrap(), "127");
        assert!(!field2.subfields.contains_key("sequence_number"));
    }

    #[test]
    fn test_parse_balance_field_60f() {
        let mut field = MtField::new("60F", "C231114EUR50000,00");
        field.parse_balance_field().unwrap();

        assert_eq!(field.subfields.get("dc_mark").unwrap(), "C");
        assert_eq!(field.subfields.get("date").unwrap(), "231114");
        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "50000,00");
    }

    #[test]
    fn test_parse_balance_field_62f_debit() {
        let mut field = MtField::new("62F", "D231115EUR97000,00");
        field.parse_balance_field().unwrap();

        assert_eq!(field.subfields.get("dc_mark").unwrap(), "D");
        assert_eq!(field.subfields.get("date").unwrap(), "231115");
        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "97000,00");
    }

    #[test]
    fn test_parse_field_61_simple() {
        // No entry date in this format: value date immediately followed by D/C mark
        let mut field = MtField::new("61", "231115C15000,00NTRF020231115001//BANK REF 001");
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert!(!field.subfields.contains_key("entry_date")); // No entry date
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "C");
        assert_eq!(field.subfields.get("amount").unwrap(), "15000,00");
    }

    #[test]
    fn test_parse_field_61_with_entry_date() {
        // Value date: 231115, Entry date: 1115, D/C: C, Amount: 15000,00
        let mut field = MtField::new("61", "2311151115C15000,00NTRF020231115001//BANK REF 001");
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert_eq!(field.subfields.get("entry_date").unwrap(), "1115");
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "C");
        assert_eq!(field.subfields.get("amount").unwrap(), "15000,00");
        assert_eq!(field.subfields.get("transaction_type").unwrap(), "NTRF");
        assert_eq!(
            field.subfields.get("customer_reference").unwrap(),
            "020231115001"
        );
        assert_eq!(field.subfields.get("bank_reference").unwrap(), "BANK REF 001");
    }

    #[test]
    fn test_parse_field_61_without_entry_date() {
        // No entry date (D/C immediately follows value date)
        let mut field = MtField::new("61", "231115C15000,00NTRF020231115001//BANK REF 001");
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert!(!field.subfields.contains_key("entry_date"));
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "C");
        assert_eq!(field.subfields.get("amount").unwrap(), "15000,00");
    }

    #[test]
    fn test_parse_field_61_debit() {
        let mut field = MtField::new("61", "231115D8000,00NDDT024115DEB001//BANK REF 005");
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "D");
        assert_eq!(field.subfields.get("amount").unwrap(), "8000,00");
        assert_eq!(field.subfields.get("transaction_type").unwrap(), "NDDT");
        assert_eq!(field.subfields.get("customer_reference").unwrap(), "024115DEB001");
        assert_eq!(field.subfields.get("bank_reference").unwrap(), "BANK REF 005");
    }

    #[test]
    fn test_parse_field_61_with_supplementary() {
        // Test without entry date (D/C immediately follows value date)
        let value = "231115D2500,00NCHK022115CHK123//BANK REF 003\nPresented cheque";
        let mut field = MtField::new("61", value);
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert!(!field.subfields.contains_key("entry_date"));
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "D");
        assert_eq!(field.subfields.get("amount").unwrap(), "2500,00");
        assert_eq!(
            field.subfields.get("supplementary_details").unwrap(),
            "Presented cheque"
        );
    }

    #[test]
    fn test_parse_field_61_reversal() {
        let mut field = MtField::new("61", "231115RC5000,00NSTD021115STDO001");
        field.parse_field_61().unwrap();

        assert_eq!(field.subfields.get("value_date").unwrap(), "231115");
        assert_eq!(field.subfields.get("dc_mark").unwrap(), "RC");
        assert_eq!(field.subfields.get("amount").unwrap(), "5000,00");
    }

    #[test]
    fn test_parse_field_86_unstructured() {
        let value = "Payment from Customer A for Invoice 12345";
        let mut field = MtField::new("86", value);
        field.parse_field_86().unwrap();

        assert_eq!(field.subfields.get("text").unwrap(), value);
    }

    #[test]
    fn test_parse_field_86_structured() {
        let value = "?20Payment from Customer A\n?32ACME CORPORATION\n?60DE89370400440532013000";
        let mut field = MtField::new("86", value);
        field.parse_field_86().unwrap();

        assert_eq!(field.subfields.get("text").unwrap(), value);
        assert_eq!(
            field.subfields.get("code_20").unwrap(),
            "Payment from Customer A"
        );
        assert_eq!(field.subfields.get("code_32").unwrap(), "ACME CORPORATION");
        assert_eq!(
            field.subfields.get("code_60").unwrap(),
            "DE89370400440532013000"
        );
    }

    #[test]
    fn test_parse_field_86_structured_multiline() {
        let value = "?20Incoming Wire Transfer\n?32Business Partner Corp\n?60IT60X0542811101000000123456";
        let mut field = MtField::new("86", value);
        field.parse_field_86().unwrap();

        assert_eq!(
            field.subfields.get("code_20").unwrap(),
            "Incoming Wire Transfer"
        );
        assert_eq!(
            field.subfields.get("code_32").unwrap(),
            "Business Partner Corp"
        );
        assert_eq!(
            field.subfields.get("code_60").unwrap(),
            "IT60X0542811101000000123456"
        );
    }

    #[test]
    fn test_parse_field_34f() {
        let mut field = MtField::new("34F", "EUR10000,00");
        field.parse_field_34f().unwrap();

        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "10000,00");
    }

    #[test]
    fn test_parse_field_13d() {
        let mut field = MtField::new("13D", "231115+1430");
        field.parse_field_13d().unwrap();

        assert_eq!(field.subfields.get("date").unwrap(), "231115");
        assert_eq!(field.subfields.get("time").unwrap(), "1430");
    }

    #[test]
    fn test_parse_field_90d() {
        let mut field = MtField::new("90D", "8EUR45000,00");
        field.parse_field_90().unwrap();

        assert_eq!(field.subfields.get("number_of_entries").unwrap(), "8");
        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "45000,00");
    }

    #[test]
    fn test_parse_field_90c() {
        let mut field = MtField::new("90C", "12EUR78000,00");
        field.parse_field_90().unwrap();

        assert_eq!(field.subfields.get("number_of_entries").unwrap(), "12");
        assert_eq!(field.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field.subfields.get("amount").unwrap(), "78000,00");
    }

    #[test]
    fn test_parse_mt940_minimal() {
        let content = ":20:STMT20231115001
:25:DE89370400440532013000
:28C:235/1
:60F:C231114EUR10000,00
:62F:C231115EUR10000,00";

        let fields = parse_block4_fields(content).unwrap();

        assert_eq!(fields.len(), 5);

        let field_20 = fields.iter().find(|f| f.tag == "20").unwrap();
        assert_eq!(field_20.value, "STMT20231115001");

        let field_28c = fields.iter().find(|f| f.tag == "28C").unwrap();
        assert_eq!(field_28c.subfields.get("statement_number").unwrap(), "235");
        assert_eq!(field_28c.subfields.get("sequence_number").unwrap(), "1");

        let field_60f = fields.iter().find(|f| f.tag == "60F").unwrap();
        assert_eq!(field_60f.subfields.get("dc_mark").unwrap(), "C");
        assert_eq!(field_60f.subfields.get("currency").unwrap(), "EUR");
        assert_eq!(field_60f.subfields.get("amount").unwrap(), "10000,00");
    }

    #[test]
    fn test_parse_mt942_minimal() {
        let content = ":20:INTR20231115001
:25:DE89370400440532013000
:28C:1/1";

        let fields = parse_block4_fields(content).unwrap();

        assert_eq!(fields.len(), 3);

        let field_20 = fields.iter().find(|f| f.tag == "20").unwrap();
        assert_eq!(field_20.value, "INTR20231115001");

        let field_28c = fields.iter().find(|f| f.tag == "28C").unwrap();
        assert_eq!(field_28c.subfields.get("statement_number").unwrap(), "1");
        assert_eq!(field_28c.subfields.get("sequence_number").unwrap(), "1");
    }
}
