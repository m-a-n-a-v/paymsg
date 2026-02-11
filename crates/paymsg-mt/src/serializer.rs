//! MT message serialization (struct → MT text).
//!
//! This module provides serialization logic for SWIFT MT messages:
//! - Generate proper block structure with curly braces
//! - Format amounts with comma decimal separator
//! - Format dates in YYMMDD format
//! - Handle CRLF line endings in Block 4
//! - Proper padding for session/sequence numbers

use crate::blocks::*;
use crate::fields::MtField;
use crate::parser::MtMessage;
use paymsg_core::{Date, PaymsgError};
use rust_decimal::Decimal;
use std::fmt::Write as FmtWrite;

impl MtMessage {
    /// Serialize the MT message to SWIFT MT text format.
    ///
    /// Returns a string in the standard 5-block format with proper formatting,
    /// including CRLF line endings in Block 4 and proper field delimiters.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_mt::{MtMessage, BasicHeader, ApplicationHeader, Direction, TextBlock};
    ///
    /// let msg = MtMessage {
    ///     block1: BasicHeader {
    ///         application_id: "F".to_string(),
    ///         service_id: "01".to_string(),
    ///         logical_terminal_address: "DEUTDEFFAXXX".to_string(),
    ///         session_number: "0".to_string(),
    ///         sequence_number: "0".to_string(),
    ///     },
    ///     block2: ApplicationHeader {
    ///         direction: Direction::Input,
    ///         message_type: "103".to_string(),
    ///         bic: "BNPAFRPPXXXX".to_string(),
    ///         priority: "N".to_string(),
    ///         delivery_monitoring: None,
    ///         obsolescence_period: None,
    ///     },
    ///     block3: None,
    ///     block4: TextBlock {
    ///         content: ":20:REF123\r\n:32A:260210EUR1234,56".to_string(),
    ///     },
    ///     block5: None,
    /// };
    ///
    /// let output = msg.serialize().unwrap();
    /// assert!(output.contains("{1:F01DEUTDEFFAXXX0000000000}"));
    /// assert!(output.contains("{2:I103BNPAFRPPXXXXN}"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SerializationError` if any field has invalid length or format.
    pub fn serialize(&self) -> Result<String, PaymsgError> {
        let mut output = String::new();

        // Block 1: Basic Header
        output.push_str(&serialize_block1(&self.block1)?);

        // Block 2: Application Header
        output.push_str(&serialize_block2(&self.block2)?);

        // Block 3: User Header (optional)
        if let Some(ref block3) = self.block3 {
            output.push_str(&serialize_block3(block3)?);
        }

        // Block 4: Text Block
        output.push_str(&serialize_block4(&self.block4)?);

        // Block 5: Trailer (optional)
        if let Some(ref block5) = self.block5 {
            output.push_str(&serialize_block5(block5)?);
        }

        Ok(output)
    }
}

/// Serialize Block 1: Basic Header
///
/// Format: `{1:F01BANKBICAXXX0000000000}`
/// - Application ID: 1 char
/// - Service ID: 2 chars
/// - Logical Terminal Address: 12 chars
/// - Session number: 4 digits (zero-padded)
/// - Sequence number: 6 digits (zero-padded)
fn serialize_block1(block: &BasicHeader) -> Result<String, PaymsgError> {
    // Validate field lengths
    if block.application_id.len() != 1 {
        return Err(PaymsgError::SerializationError(
            "Application ID must be 1 character".to_string(),
        ));
    }
    if block.service_id.len() != 2 {
        return Err(PaymsgError::SerializationError(
            "Service ID must be 2 characters".to_string(),
        ));
    }
    if block.logical_terminal_address.len() != 12 {
        return Err(PaymsgError::SerializationError(format!(
            "Logical Terminal Address must be 12 characters, got {}",
            block.logical_terminal_address.len()
        )));
    }

    // Parse and pad session/sequence numbers
    let session = parse_and_pad(&block.session_number, 4)?;
    let sequence = parse_and_pad(&block.sequence_number, 6)?;

    Ok(format!(
        "{{1:{}{}{}{}{}}}",
        block.application_id,
        block.service_id,
        block.logical_terminal_address,
        session,
        sequence
    ))
}

/// Serialize Block 2: Application Header
///
/// Input format: `{2:I103BNPAFRPPXXXXN}`
/// Output format: `{2:O1030800010101DEUTDEFFAXXXN}`
fn serialize_block2(block: &ApplicationHeader) -> Result<String, PaymsgError> {
    let direction_char = match block.direction {
        Direction::Input => 'I',
        Direction::Output => 'O',
    };

    // Validate message type (3 chars)
    if block.message_type.len() != 3 {
        return Err(PaymsgError::SerializationError(format!(
            "Message type must be 3 characters, got {}",
            block.message_type.len()
        )));
    }

    // Validate priority (1 char)
    if block.priority.len() != 1 {
        return Err(PaymsgError::SerializationError(
            "Priority must be 1 character".to_string(),
        ));
    }

    let mut output = format!("{{2:{}{}", direction_char, block.message_type);

    match block.direction {
        Direction::Input => {
            // Input format: I + message_type(3) + destination_bic(12) + priority(1)
            if block.bic.len() != 12 {
                return Err(PaymsgError::SerializationError(format!(
                    "BIC must be 12 characters for Input messages, got {}",
                    block.bic.len()
                )));
            }
            output.push_str(&block.bic);
            output.push_str(&block.priority);
        }
        Direction::Output => {
            // Output format: O + message_type(3) + input_time(4) + mir(28) + output_date(6) + output_time(4) + priority(1)
            // For simplicity, we'll use defaults if not provided in the struct
            // A full implementation would need additional fields in ApplicationHeader

            // Default values for output format (these should come from additional struct fields)
            let input_time = "0800"; // HHMM

            // MIR (Message Input Reference): date(6) + LT address(12) + session(4) + sequence(6)
            let mir = if block.bic.len() >= 12 {
                // Extract date from current timestamp or use default
                let date = "010101"; // YYMMDD - should be actual date
                let lt_addr = &block.bic[..12];
                let session = "0001";
                let sequence = "000001";
                format!("{}{}{}{}", date, lt_addr, session, sequence)
            } else {
                return Err(PaymsgError::SerializationError(
                    "BIC too short for Output message".to_string(),
                ));
            };

            let output_date = "010101"; // YYMMDD
            let output_time = "0800"; // HHMM

            output.push_str(input_time);
            output.push_str(&mir);
            output.push_str(output_date);
            output.push_str(output_time);
            output.push_str(&block.priority);
        }
    }

    output.push('}');
    Ok(output)
}

/// Serialize Block 3: User Header
///
/// Format: `{3:{108:TESTMUR}{121:UUID}}`
fn serialize_block3(block: &UserHeader) -> Result<String, PaymsgError> {
    if block.tags.is_empty() {
        // Don't serialize empty block 3
        return Ok(String::new());
    }

    let mut output = String::from("{3:");

    // Sort tags for consistent output
    let mut tags: Vec<_> = block.tags.iter().collect();
    tags.sort_by_key(|(k, _)| *k);

    for (tag, value) in tags {
        write!(output, "{{{tag}:{value}}}").map_err(|e| {
            PaymsgError::SerializationError(format!("Failed to write tag {tag}: {e}"))
        })?;
    }

    output.push('}');
    Ok(output)
}

/// Serialize Block 4: Text Block
///
/// Format: `{4:\n:20:REF\n:32A:...\n-}`
fn serialize_block4(block: &TextBlock) -> Result<String, PaymsgError> {
    let mut output = String::from("{4:\r\n");

    // Content should already be properly formatted
    // We need to ensure it uses CRLF line endings and ends with -
    let content = block.content.trim();

    // Convert any \n to \r\n for SWIFT format
    let content = content.replace('\n', "\r\n");

    output.push_str(&content);

    // Ensure it ends with CRLF and dash
    if !content.ends_with("\r\n-") {
        if !content.ends_with('\n') && !content.is_empty() {
            output.push_str("\r\n");
        }
        output.push('-');
    }

    output.push('}');
    Ok(output)
}

/// Serialize Block 5: Trailer
///
/// Format: `{5:{CHK:ABC}{TNG:}}`
fn serialize_block5(block: &Trailer) -> Result<String, PaymsgError> {
    if block.tags.is_empty() {
        // Don't serialize empty block 5
        return Ok(String::new());
    }

    let mut output = String::from("{5:");

    // Sort tags for consistent output
    let mut tags: Vec<_> = block.tags.iter().collect();
    tags.sort_by_key(|(k, _)| *k);

    for (tag, value) in tags {
        write!(output, "{{{tag}:{value}}}").map_err(|e| {
            PaymsgError::SerializationError(format!("Failed to write tag {tag}: {e}"))
        })?;
    }

    output.push('}');
    Ok(output)
}

/// Helper to parse and zero-pad a number string.
fn parse_and_pad(s: &str, width: usize) -> Result<String, PaymsgError> {
    let num: u32 = s.parse().map_err(|e| {
        PaymsgError::SerializationError(format!("Invalid number '{}': {}", s, e))
    })?;
    Ok(format!("{:0width$}", num))
}

/// Format an amount with comma as decimal separator (SWIFT MT format).
///
/// SWIFT MT messages use comma (,) as the decimal separator instead of period (.).
///
/// # Examples
///
/// ```
/// use paymsg_mt::format_mt_amount;
/// use rust_decimal::Decimal;
///
/// let amount = Decimal::new(123456, 2); // 1234.56
/// assert_eq!(format_mt_amount(amount), "1234,56");
/// ```
pub fn format_mt_amount(amount: Decimal) -> String {
    amount.to_string().replace('.', ",")
}

/// Format a date as YYMMDD (SWIFT MT format).
///
/// Extracts the last two digits of the year to create the YY format.
///
/// # Examples
///
/// ```
/// use paymsg_mt::format_mt_date_yymmdd;
/// use paymsg_core::Date;
///
/// let date = Date::from_iso8601("2026-02-10").unwrap();
/// assert_eq!(format_mt_date_yymmdd(&date), "260210");
/// ```
pub fn format_mt_date_yymmdd(date: &Date) -> String {
    let s = date.to_string();
    // Date is in format YYYY-MM-DD, extract YY-MM-DD
    if s.len() >= 10 {
        let year = &s[2..4]; // YY
        let month = &s[5..7]; // MM
        let day = &s[8..10]; // DD
        format!("{}{}{}", year, month, day)
    } else {
        s
    }
}

/// Serialize multiple fields into Block 4 content format.
///
/// This helper takes a vector of fields and formats them as Block 4 content
/// with proper `:TAG:VALUE` format and CRLF line breaks.
///
/// # Examples
///
/// ```
/// use paymsg_mt::{serialize_fields, MtField};
///
/// let fields = vec![
///     MtField::new("20", "REF123"),
///     MtField::new("32A", "260210EUR1234,56"),
/// ];
///
/// let content = serialize_fields(&fields);
/// assert!(content.contains(":20:REF123"));
/// assert!(content.contains(":32A:260210EUR1234,56"));
/// ```
pub fn serialize_fields(fields: &[MtField]) -> String {
    let mut output = String::new();

    for field in fields {
        output.push_str(&format!(":{}:", field.tag));
        output.push_str(&field.value);
        output.push_str("\r\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_serialize_block1() {
        let block = BasicHeader {
            application_id: "F".to_string(),
            service_id: "01".to_string(),
            logical_terminal_address: "DEUTDEFFAXXX".to_string(),
            session_number: "0".to_string(),
            sequence_number: "0".to_string(),
        };

        let result = serialize_block1(&block).unwrap();
        assert_eq!(result, "{1:F01DEUTDEFFAXXX0000000000}");
    }

    #[test]
    fn test_serialize_block1_with_padding() {
        let block = BasicHeader {
            application_id: "F".to_string(),
            service_id: "01".to_string(),
            logical_terminal_address: "DEUTDEFFAXXX".to_string(),
            session_number: "123".to_string(),
            sequence_number: "456".to_string(),
        };

        let result = serialize_block1(&block).unwrap();
        assert_eq!(result, "{1:F01DEUTDEFFAXXX0123000456}");
    }

    #[test]
    fn test_serialize_block2_input() {
        let block = ApplicationHeader {
            direction: Direction::Input,
            message_type: "103".to_string(),
            bic: "BNPAFRPPXXXX".to_string(),
            priority: "N".to_string(),
            delivery_monitoring: None,
            obsolescence_period: None,
        };

        let result = serialize_block2(&block).unwrap();
        assert_eq!(result, "{2:I103BNPAFRPPXXXXN}");
    }

    #[test]
    fn test_serialize_block3() {
        let mut tags = HashMap::new();
        tags.insert("108".to_string(), "TESTMUR".to_string());
        tags.insert("121".to_string(), "uuid-1234".to_string());

        let block = UserHeader { tags };

        let result = serialize_block3(&block).unwrap();
        // Tags are sorted alphabetically
        assert!(result.contains("{108:TESTMUR}"));
        assert!(result.contains("{121:uuid-1234}"));
        assert!(result.starts_with("{3:"));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_serialize_block3_empty() {
        let block = UserHeader {
            tags: HashMap::new(),
        };

        let result = serialize_block3(&block).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_serialize_block4() {
        let block = TextBlock {
            content: ":20:REF123\r\n:32A:260210EUR1234,56".to_string(),
        };

        let result = serialize_block4(&block).unwrap();
        assert!(result.starts_with("{4:\r\n"));
        assert!(result.contains(":20:REF123"));
        assert!(result.ends_with("-}"));
    }

    #[test]
    fn test_serialize_block5() {
        let mut tags = HashMap::new();
        tags.insert("CHK".to_string(), "123456789ABC".to_string());

        let block = Trailer { tags };

        let result = serialize_block5(&block).unwrap();
        assert_eq!(result, "{5:{CHK:123456789ABC}}");
    }

    #[test]
    fn test_format_mt_amount() {
        let amount = Decimal::new(123456, 2); // 1234.56
        assert_eq!(format_mt_amount(amount), "1234,56");

        let amount2 = Decimal::new(100000, 2); // 1000.00
        assert_eq!(format_mt_amount(amount2), "1000,00");
    }

    #[test]
    fn test_format_mt_date_yymmdd() {
        let date = Date::from_iso8601("2026-02-10").unwrap();
        assert_eq!(format_mt_date_yymmdd(&date), "260210");

        let date2 = Date::from_iso8601("2025-12-31").unwrap();
        assert_eq!(format_mt_date_yymmdd(&date2), "251231");
    }

    #[test]
    fn test_serialize_fields() {
        let fields = vec![
            MtField::new("20", "REF123"),
            MtField::new("32A", "260210EUR1234,56"),
        ];

        let result = serialize_fields(&fields);
        assert_eq!(result, ":20:REF123\r\n:32A:260210EUR1234,56\r\n");
    }

    #[test]
    fn test_parse_and_pad() {
        assert_eq!(parse_and_pad("123", 4).unwrap(), "0123");
        assert_eq!(parse_and_pad("0", 6).unwrap(), "000000");
        assert_eq!(parse_and_pad("999", 4).unwrap(), "0999");
    }
}
