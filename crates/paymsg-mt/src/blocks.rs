//! SWIFT MT message block structures.
//!
//! This module defines the 5-block structure of SWIFT MT messages:
//! - Block 1: Basic Header
//! - Block 2: Application Header (Input or Output)
//! - Block 3: User Header (optional)
//! - Block 4: Text Block (message body)
//! - Block 5: Trailer (optional)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block 1: Basic Header
///
/// Format: `{1:F01BANKBICAXXX0000000000}`
/// - Application ID: F (FIN) or A (automatic)
/// - Service ID: 01 (FIN), 21 (GPA), etc.
/// - Logical Terminal Address: 12 characters (BIC8 + LT code + branch)
/// - Session number: 4 digits
/// - Sequence number: 6 digits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicHeader {
    /// Application ID (F=FIN, A=automatic)
    pub application_id: String,
    /// Service ID (01=FIN, 21=GPA)
    pub service_id: String,
    /// Logical Terminal Address (12 chars: BIC8 + LT code + branch)
    pub logical_terminal_address: String,
    /// Session number (4 digits)
    pub session_number: String,
    /// Sequence number (6 digits)
    pub sequence_number: String,
}

/// Block 2 direction: Input (I) or Output (O)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Input message (I)
    Input,
    /// Output message (O)
    Output,
}

/// Block 2: Application Header
///
/// Input format: `{2:I103BNPAFRPPXXXXN}`
/// Output format: `{2:O1030800010101DEUTDEFFAXXXN}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationHeader {
    /// Message direction (Input or Output)
    pub direction: Direction,
    /// Message type (e.g., "103", "202", "940", "942")
    pub message_type: String,
    /// Destination BIC (Input) or Sender BIC (Output)
    pub bic: String,
    /// Priority (N=normal, U=urgent, S=system)
    pub priority: String,
    /// Delivery monitoring (Output only, e.g., "2" or "3")
    pub delivery_monitoring: Option<String>,
    /// Obsolescence period (Output only, e.g., "020")
    pub obsolescence_period: Option<String>,
}

/// Block 3: User Header (optional)
///
/// Format: `{3:{108:TESTMUR}{121:UUID}}`
/// Contains optional tag-value pairs like:
/// - 108: Message User Reference (MUR)
/// - 119: Validation flag (STP, etc.)
/// - 121: Unique End-to-End Transaction Reference (UETR)
/// - 423: Balance checkpoint
/// - 424: Related reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserHeader {
    /// Tag-value pairs (e.g., "108" -> "TESTMUR", "121" -> "UUID")
    pub tags: HashMap<String, String>,
}

/// Block 4: Text Block (message body)
///
/// Format: `{4:\n:20:REF\n:32A:...\n-}`
/// Contains the actual message fields in tag:value format.
/// This block stores the raw content; field parsing happens separately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBlock {
    /// Raw text content of block 4 (without the {4: prefix and -} suffix)
    pub content: String,
}

/// Block 5: Trailer (optional)
///
/// Format: `{5:{CHK:ABC}{TNG:}}`
/// Contains optional tags like:
/// - CHK: Checksum
/// - TNG: Training
/// - PDE: Possible Duplicate Emission
/// - PDM: Possible Duplicate Message
/// - SYS: System originated message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Trailer {
    /// Tag-value pairs (e.g., "CHK" -> "ABC123")
    pub tags: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_header_creation() {
        let header = BasicHeader {
            application_id: "F".to_string(),
            service_id: "01".to_string(),
            logical_terminal_address: "DEUTDEFFAXXX".to_string(),
            session_number: "0000".to_string(),
            sequence_number: "000000".to_string(),
        };

        assert_eq!(header.application_id, "F");
        assert_eq!(header.logical_terminal_address, "DEUTDEFFAXXX");
    }

    #[test]
    fn test_application_header_input() {
        let header = ApplicationHeader {
            direction: Direction::Input,
            message_type: "103".to_string(),
            bic: "BNPAFRPPXXXX".to_string(),
            priority: "N".to_string(),
            delivery_monitoring: None,
            obsolescence_period: None,
        };

        assert_eq!(header.direction, Direction::Input);
        assert_eq!(header.message_type, "103");
    }

    #[test]
    fn test_application_header_output() {
        let header = ApplicationHeader {
            direction: Direction::Output,
            message_type: "103".to_string(),
            bic: "DEUTDEFFAXXX".to_string(),
            priority: "N".to_string(),
            delivery_monitoring: Some("2".to_string()),
            obsolescence_period: Some("020".to_string()),
        };

        assert_eq!(header.direction, Direction::Output);
        assert!(header.delivery_monitoring.is_some());
    }

    #[test]
    fn test_user_header_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("108".to_string(), "TESTMUR".to_string());
        tags.insert("121".to_string(), "uuid-here".to_string());

        let header = UserHeader { tags };

        assert_eq!(header.tags.get("108").unwrap(), "TESTMUR");
        assert_eq!(header.tags.len(), 2);
    }

    #[test]
    fn test_text_block() {
        let block = TextBlock {
            content: ":20:REF123\n:32A:260210EUR1234,56\n".to_string(),
        };

        assert!(block.content.contains(":20:REF123"));
    }

    #[test]
    fn test_trailer_with_checksum() {
        let mut tags = HashMap::new();
        tags.insert("CHK".to_string(), "123456789ABC".to_string());

        let trailer = Trailer { tags };

        assert_eq!(trailer.tags.get("CHK").unwrap(), "123456789ABC");
    }
}
