//! MT message parser implementation.
//!
//! This module provides parsing logic for SWIFT MT messages, including:
//! - Block extraction (5 blocks)
//! - Block-specific parsing for each block type
//! - Handling of both \r\n and \n line endings

use crate::blocks::*;
use paymsg_core::PaymsgError;
use regex::Regex;
use std::collections::HashMap;

/// Main MT message structure containing all 5 blocks.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MtMessage {
    /// Block 1: Basic Header (mandatory)
    pub block1: BasicHeader,
    /// Block 2: Application Header (mandatory)
    pub block2: ApplicationHeader,
    /// Block 3: User Header (optional)
    pub block3: Option<UserHeader>,
    /// Block 4: Text Block (mandatory)
    pub block4: TextBlock,
    /// Block 5: Trailer (optional)
    pub block5: Option<Trailer>,
}

impl MtMessage {
    /// Parse an MT message from raw text.
    ///
    /// The message must be in the standard 5-block format:
    /// `{1:...}{2:...}{3:...}{4:...-}{5:...}`
    ///
    /// Blocks 3 and 5 are optional. The parser handles both `\r\n` and `\n` line endings.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_mt::MtMessage;
    ///
    /// let input = "{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
    /// :20:STMT20231115001
    /// :25:DE89370400440532013000
    /// :28C:235/1
    /// :60F:C231114EUR10000,00
    /// :62F:C231115EUR10000,00
    /// -}";
    ///
    /// let msg = MtMessage::parse(input).unwrap();
    /// assert_eq!(msg.block2.message_type, "940");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::ParseError` if:
    /// - Required blocks (1, 2, 4) are missing
    /// - Block format is invalid
    /// - Block content cannot be parsed
    pub fn parse(input: &str) -> Result<Self, PaymsgError> {
        // Normalize line endings to \n
        let input = input.replace("\r\n", "\n");

        // Extract blocks
        let blocks = extract_blocks(&input)?;

        // Find and parse each block by its number
        let mut block1 = None;
        let mut block2 = None;
        let mut block3 = None;
        let mut block4 = None;
        let mut block5 = None;

        for block_str in blocks {
            match block_str.chars().next() {
                Some('1') => block1 = Some(parse_block1(&block_str)?),
                Some('2') => block2 = Some(parse_block2(&block_str)?),
                Some('3') => block3 = Some(parse_block3(&block_str)?),
                Some('4') => block4 = Some(parse_block4(&block_str)?),
                Some('5') => block5 = Some(parse_block5(&block_str)?),
                _ => {}
            }
        }

        // Validate mandatory blocks are present
        let block1 = block1.ok_or_else(|| PaymsgError::ParseError("Block 1 is mandatory".to_string()))?;
        let block2 = block2.ok_or_else(|| PaymsgError::ParseError("Block 2 is mandatory".to_string()))?;
        let block4 = block4.ok_or_else(|| PaymsgError::ParseError("Block 4 is mandatory".to_string()))?;

        Ok(MtMessage {
            block1,
            block2,
            block3,
            block4,
            block5,
        })
    }
}

/// Extract block contents from the raw message.
///
/// Returns a vector of block contents (with the block number prefix, e.g., "1F01...")
fn extract_blocks(input: &str) -> Result<Vec<String>, PaymsgError> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        // Find the next block start pattern {N:
        let remaining = &input[pos..];

        // Try to find a block pattern
        if let Some(block_start) = remaining.find("{1:") {
            if blocks.is_empty() {
                // This is block 1
                pos += block_start + 3; // Move past {1:
                let block_end = find_block_end(input, pos, '1')?;
                blocks.push(format!("1{}", &input[pos..block_end]));
                pos = block_end + 1; // Skip closing }
                continue;
            }
        }

        if let Some(block_start) = remaining.find("{2:") {
            if blocks.len() == 1 {
                // This is block 2
                pos += block_start + 3;
                let block_end = find_block_end(input, pos, '2')?;
                blocks.push(format!("2{}", &input[pos..block_end]));
                pos = block_end + 1;
                continue;
            }
        }

        if let Some(block_start) = remaining.find("{3:") {
            pos += block_start + 3;
            let block_end = find_block_end(input, pos, '3')?;
            blocks.push(format!("3{}", &input[pos..block_end]));
            pos = block_end + 1;
            continue;
        }

        if let Some(block_start) = remaining.find("{4:") {
            pos += block_start + 3;
            // Block 4 ends with -}
            if let Some(end_offset) = input[pos..].find("-}") {
                let block_end = pos + end_offset;
                blocks.push(format!("4{}", &input[pos..block_end]));
                pos = block_end + 2; // Skip -}
                continue;
            } else {
                return Err(PaymsgError::ParseError(
                    "Block 4 missing closing '-}' marker".to_string(),
                ));
            }
        }

        if let Some(block_start) = remaining.find("{5:") {
            pos += block_start + 3;
            let block_end = find_block_end(input, pos, '5')?;
            blocks.push(format!("5{}", &input[pos..block_end]));
            pos = block_end + 1;
            continue;
        }

        // No more blocks found
        break;
    }

    // Validate we have mandatory blocks
    if blocks.len() < 3 {
        return Err(PaymsgError::ParseError(format!(
            "Invalid MT message: found only {} blocks, need at least 3",
            blocks.len()
        )));
    }

    Ok(blocks)
}

/// Find the end of a block (the closing brace, accounting for nested braces)
fn find_block_end(input: &str, start: usize, _block_num: char) -> Result<usize, PaymsgError> {
    let mut depth = 1; // We're already inside one brace
    let chars: Vec<char> = input.chars().collect();

    for (offset, ch) in chars.iter().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(offset);
                }
            }
            _ => {}
        }
    }

    Err(PaymsgError::ParseError(
        "Unmatched opening brace in block".to_string(),
    ))
}


/// Parse Block 1: Basic Header
///
/// Format: `1F01DEUTDEFFAXXX0000000000`
/// - Block number: 1
/// - Application ID: F (1 char)
/// - Service ID: 01 (2 chars)
/// - Logical Terminal Address: DEUTDEFFAXXX (12 chars)
/// - Session number: 0000 (4 chars)
/// - Sequence number: 000000 (6 chars)
fn parse_block1(block: &str) -> Result<BasicHeader, PaymsgError> {
    // Block format: "1F01DEUTDEFFAXXX0000000000"
    // Remove the leading "1" block number
    let content = block
        .strip_prefix('1')
        .ok_or_else(|| PaymsgError::ParseError("Block 1 must start with '1'".to_string()))?;

    // Expected length: 1 (app_id) + 2 (service_id) + 12 (LT addr) + 4 (session) + 6 (sequence) = 25
    if content.len() != 25 {
        return Err(PaymsgError::ParseError(format!(
            "Block 1 content must be 25 characters, got {}",
            content.len()
        )));
    }

    Ok(BasicHeader {
        application_id: content[0..1].to_string(),
        service_id: content[1..3].to_string(),
        logical_terminal_address: content[3..15].to_string(),
        session_number: content[15..19].to_string(),
        sequence_number: content[19..25].to_string(),
    })
}

/// Parse Block 2: Application Header
///
/// Input format: `2I103BNPAFRPPXXXXN`
/// Output format: `2O1030800010101DEUTDEFFAXXXN`
fn parse_block2(block: &str) -> Result<ApplicationHeader, PaymsgError> {
    // Remove the leading "2" block number
    let content = block
        .strip_prefix('2')
        .ok_or_else(|| PaymsgError::ParseError("Block 2 must start with '2'".to_string()))?;

    if content.is_empty() {
        return Err(PaymsgError::ParseError(
            "Block 2 content is empty".to_string(),
        ));
    }

    let direction = match &content[0..1] {
        "I" => Direction::Input,
        "O" => Direction::Output,
        _ => {
            return Err(PaymsgError::ParseError(format!(
                "Invalid direction in Block 2: {}",
                &content[0..1]
            )))
        }
    };

    match direction {
        Direction::Input => parse_block2_input(content),
        Direction::Output => parse_block2_output(content),
    }
}

/// Parse Block 2 Input format: `I103BNPAFRPPXXXXN`
fn parse_block2_input(content: &str) -> Result<ApplicationHeader, PaymsgError> {
    // Format: I + msg_type(3) + destination_bic(12) + priority(1)
    if content.len() < 16 {
        return Err(PaymsgError::ParseError(format!(
            "Block 2 Input too short: expected at least 16 chars, got {}",
            content.len()
        )));
    }

    Ok(ApplicationHeader {
        direction: Direction::Input,
        message_type: content[1..4].to_string(),
        bic: content[4..16].to_string(),
        priority: content[16..17].to_string(),
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Parse Block 2 Output format: `O1030800010101DEUTDEFFAXXXN`
fn parse_block2_output(content: &str) -> Result<ApplicationHeader, PaymsgError> {
    // Format: O + msg_type(3) + input_time(4) + input_mir(28: date(6) + lt_addr(12) + session(4) + seq(6))
    //         + output_date(6) + output_time(4) + priority(1)
    // But we primarily care about: msg_type, sender BIC (from MIR), priority

    if content.len() < 46 {
        return Err(PaymsgError::ParseError(format!(
            "Block 2 Output too short: expected at least 46 chars, got {}",
            content.len()
        )));
    }

    let message_type = content[1..4].to_string();
    // Extract BIC from the MIR (positions 11-23 in the MIR, which starts at position 7)
    // Position 7-34 is the MIR, within that positions 6-18 (0-indexed) is the LT address
    let mir_start = 7;
    let bic_in_mir_start = mir_start + 6; // Skip date (6 chars)
    let bic = content[bic_in_mir_start..bic_in_mir_start + 12].to_string();

    // Priority is at the end
    let priority = content[content.len() - 1..].to_string();

    // Extract delivery monitoring and obsolescence period if present (optional fields after priority)
    // In the full format, these appear before the priority, but we'll simplify for now

    Ok(ApplicationHeader {
        direction: Direction::Output,
        message_type,
        bic,
        priority,
        delivery_monitoring: None,
        obsolescence_period: None,
    })
}

/// Parse Block 3: User Header (optional)
///
/// Format: `3{108:TESTMUR}{121:UUID}`
fn parse_block3(block: &str) -> Result<UserHeader, PaymsgError> {
    // Remove the leading "3" block number
    let content = block
        .strip_prefix('3')
        .ok_or_else(|| PaymsgError::ParseError("Block 3 must start with '3'".to_string()))?;

    let tags = parse_tag_value_pairs(content)?;
    Ok(UserHeader { tags })
}

/// Parse Block 4: Text Block
///
/// Format: `4\n:20:REF\n:32A:...\n-`
/// We just extract the content; field parsing happens separately.
fn parse_block4(block: &str) -> Result<TextBlock, PaymsgError> {
    // Remove the leading "4" block number
    let content = block
        .strip_prefix('4')
        .ok_or_else(|| PaymsgError::ParseError("Block 4 must start with '4'".to_string()))?;

    // Remove the trailing dash if present
    let content = content.trim_end_matches('-').trim();

    Ok(TextBlock {
        content: content.to_string(),
    })
}

/// Parse Block 5: Trailer (optional)
///
/// Format: `5{CHK:ABC}{TNG:}`
fn parse_block5(block: &str) -> Result<Trailer, PaymsgError> {
    // Remove the leading "5" block number
    let content = block
        .strip_prefix('5')
        .ok_or_else(|| PaymsgError::ParseError("Block 5 must start with '5'".to_string()))?;

    let tags = parse_tag_value_pairs(content)?;
    Ok(Trailer { tags })
}

/// Parse tag-value pairs in format `{TAG:VALUE}{TAG2:VALUE2}`
fn parse_tag_value_pairs(content: &str) -> Result<HashMap<String, String>, PaymsgError> {
    let mut tags = HashMap::new();
    let re = Regex::new(r"\{([^:]+):([^}]*)\}")
        .map_err(|e| PaymsgError::ParseError(format!("Regex error: {}", e)))?;

    for cap in re.captures_iter(content) {
        let tag = cap[1].to_string();
        let value = cap[2].to_string();
        tags.insert(tag, value);
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_mt940() {
        let input = "{1:F01DEUTDEFFAXXX0000000000}{2:I940CORPORATEXXXN}{4:
:20:STMT20231115001
:25:DE89370400440532013000
:28C:235/1
:60F:C231114EUR10000,00
:62F:C231115EUR10000,00
-}";

        let msg = MtMessage::parse(input).unwrap();

        assert_eq!(msg.block1.application_id, "F");
        assert_eq!(msg.block1.service_id, "01");
        assert_eq!(msg.block1.logical_terminal_address, "DEUTDEFFAXXX");

        assert_eq!(msg.block2.direction, Direction::Input);
        assert_eq!(msg.block2.message_type, "940");
        assert_eq!(msg.block2.bic, "CORPORATEXXX");

        assert!(msg.block3.is_none());
        assert!(msg.block4.content.contains(":20:STMT20231115001"));
        assert!(msg.block5.is_none());
    }

    #[test]
    fn test_parse_minimal_mt103() {
        let input = "{1:F01DEUTDEFFAXXX0000000000}{2:I103BNPAFRPPXXXXN}{4:
:20:TESTREF12345
:23B:CRED
:32A:260210EUR1234,56
:50K:/DE89370400440532013000
HANS MUELLER
HAUPTSTRASSE 1
60313 FRANKFURT
:59:/FR1420041010050500013M02606
JEAN DUPONT
1 RUE DE LA PAIX
75001 PARIS
:71A:SHA
-}";

        let msg = MtMessage::parse(input).unwrap();

        assert_eq!(msg.block1.application_id, "F");
        assert_eq!(msg.block2.message_type, "103");
        assert_eq!(msg.block2.bic, "BNPAFRPPXXXX");
        assert!(msg.block4.content.contains(":20:TESTREF12345"));
        assert!(msg.block4.content.contains(":71A:SHA"));
    }

    #[test]
    fn test_parse_full_mt103_with_block3_and_5() {
        let input = "{1:F01DEUTDEFFAXXX0000000000}{2:I103BNPAFRPPXXXXN}{3:{108:TESTMUR123456}{121:a1b2c3d4-e5f6-7890-abcd-ef1234567890}}{4:
:20:FULLREF98765
:32A:260215EUR10000,00
:71A:SHA
-}{5:{CHK:123456789ABC}}";

        let msg = MtMessage::parse(input).unwrap();

        assert_eq!(msg.block1.application_id, "F");
        assert_eq!(msg.block2.message_type, "103");

        // Check block 3
        assert!(msg.block3.is_some());
        let block3 = msg.block3.unwrap();
        assert_eq!(block3.tags.get("108").unwrap(), "TESTMUR123456");
        assert_eq!(block3.tags.get("121").unwrap(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");

        // Check block 5
        assert!(msg.block5.is_some());
        let block5 = msg.block5.unwrap();
        assert_eq!(block5.tags.get("CHK").unwrap(), "123456789ABC");
    }

    #[test]
    fn test_parse_mt202() {
        let input = "{1:F01DEUTDEFFAXXX0000000000}{2:I202BNPAFRPPXXXXN}{4:
:20:FI20231115001
:21:MT103REF123456
:32A:231115EUR50000,00
:58A:BNPAFRPP
-}";

        let msg = MtMessage::parse(input).unwrap();

        assert_eq!(msg.block2.message_type, "202");
        assert!(msg.block4.content.contains(":20:FI20231115001"));
        assert!(msg.block4.content.contains(":58A:BNPAFRPP"));
    }

    #[test]
    fn test_parse_block1() {
        let block = "1F01DEUTDEFFAXXX0000000000";
        let header = parse_block1(block).unwrap();

        assert_eq!(header.application_id, "F");
        assert_eq!(header.service_id, "01");
        assert_eq!(header.logical_terminal_address, "DEUTDEFFAXXX");
        assert_eq!(header.session_number, "0000");
        assert_eq!(header.sequence_number, "000000");
    }

    #[test]
    fn test_parse_block2_input() {
        let block = "2I103BNPAFRPPXXXXN";
        let header = parse_block2(block).unwrap();

        assert_eq!(header.direction, Direction::Input);
        assert_eq!(header.message_type, "103");
        assert_eq!(header.bic, "BNPAFRPPXXXX");
        assert_eq!(header.priority, "N");
    }

    #[test]
    fn test_parse_block3() {
        let block = "3{108:TESTMUR}{121:uuid-here}";
        let header = parse_block3(block).unwrap();

        assert_eq!(header.tags.get("108").unwrap(), "TESTMUR");
        assert_eq!(header.tags.get("121").unwrap(), "uuid-here");
        assert_eq!(header.tags.len(), 2);
    }

    #[test]
    fn test_parse_block4() {
        let block = "4\n:20:REF\n:32A:260210EUR1234,56\n-";
        let text_block = parse_block4(block).unwrap();

        assert!(text_block.content.contains(":20:REF"));
        assert!(text_block.content.contains(":32A:260210EUR1234,56"));
    }

    #[test]
    fn test_parse_block5() {
        let block = "5{CHK:ABC123}{TNG:}";
        let trailer = parse_block5(block).unwrap();

        assert_eq!(trailer.tags.get("CHK").unwrap(), "ABC123");
        assert_eq!(trailer.tags.get("TNG").unwrap(), "");
    }

    #[test]
    fn test_invalid_message_missing_blocks() {
        let input = "{1:F01DEUTDEFFAXXX0000000000}";
        let result = MtMessage::parse(input);

        assert!(result.is_err());
    }
}
