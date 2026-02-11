//! Message type enumeration for SWIFT MT and ISO 20022 MX messages.

use serde::{Deserialize, Serialize};

use crate::error::{PaymsgError, Result};

/// Represents the type of a financial message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// SWIFT MT103 - Customer Credit Transfer
    Mt103,
    /// SWIFT MT202 - Financial Institution Credit Transfer
    Mt202,
    /// SWIFT MT940 - Customer Statement Message
    Mt940,
    /// SWIFT MT942 - Interim Transaction Report
    Mt942,
    /// ISO 20022 pacs.008.001.10 - Customer Credit Transfer
    Pacs008,
    /// ISO 20022 pacs.009.001.10 - Financial Institution Credit Transfer
    Pacs009,
    /// ISO 20022 camt.052.001.10 - Bank to Customer Account Report
    Camt052,
    /// ISO 20022 camt.053.001.10 - Bank to Customer Statement
    Camt053,
}

impl MessageType {
    /// Returns true if this is an MT message type.
    pub fn is_mt(&self) -> bool {
        matches!(
            self,
            MessageType::Mt103 | MessageType::Mt202 | MessageType::Mt940 | MessageType::Mt942
        )
    }

    /// Returns true if this is an MX (ISO 20022) message type.
    pub fn is_mx(&self) -> bool {
        matches!(
            self,
            MessageType::Pacs008 | MessageType::Pacs009 | MessageType::Camt052 | MessageType::Camt053
        )
    }

    /// Returns the message type identifier (e.g., "103" for MT103, "pacs.008" for Pacs008).
    pub fn identifier(&self) -> &str {
        match self {
            MessageType::Mt103 => "103",
            MessageType::Mt202 => "202",
            MessageType::Mt940 => "940",
            MessageType::Mt942 => "942",
            MessageType::Pacs008 => "pacs.008",
            MessageType::Pacs009 => "pacs.009",
            MessageType::Camt052 => "camt.052",
            MessageType::Camt053 => "camt.053",
        }
    }

    /// Returns the full message type name (e.g., "pacs.008.001.10").
    pub fn full_name(&self) -> &str {
        match self {
            MessageType::Mt103 => "MT103",
            MessageType::Mt202 => "MT202",
            MessageType::Mt940 => "MT940",
            MessageType::Mt942 => "MT942",
            MessageType::Pacs008 => "pacs.008.001.10",
            MessageType::Pacs009 => "pacs.009.001.10",
            MessageType::Camt052 => "camt.052.001.10",
            MessageType::Camt053 => "camt.053.001.10",
        }
    }

    /// Returns the category of the message (payment, statement).
    pub fn category(&self) -> MessageCategory {
        match self {
            MessageType::Mt103 | MessageType::Mt202 | MessageType::Pacs008 | MessageType::Pacs009 => {
                MessageCategory::Payment
            }
            MessageType::Mt940 | MessageType::Mt942 | MessageType::Camt052 | MessageType::Camt053 => {
                MessageCategory::Statement
            }
        }
    }

    /// Returns the equivalent MT/MX message type for translation, if available.
    pub fn equivalent(&self) -> Option<MessageType> {
        match self {
            MessageType::Mt103 => Some(MessageType::Pacs008),
            MessageType::Mt202 => Some(MessageType::Pacs009),
            MessageType::Mt940 => Some(MessageType::Camt053),
            MessageType::Mt942 => Some(MessageType::Camt052),
            MessageType::Pacs008 => Some(MessageType::Mt103),
            MessageType::Pacs009 => Some(MessageType::Mt202),
            MessageType::Camt052 => Some(MessageType::Mt942),
            MessageType::Camt053 => Some(MessageType::Mt940),
        }
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl std::str::FromStr for MessageType {
    type Err = PaymsgError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "MT103" | "103" => Ok(MessageType::Mt103),
            "MT202" | "202" => Ok(MessageType::Mt202),
            "MT940" | "940" => Ok(MessageType::Mt940),
            "MT942" | "942" => Ok(MessageType::Mt942),
            "PACS.008" | "PACS008" | "PACS.008.001.10" => Ok(MessageType::Pacs008),
            "PACS.009" | "PACS009" | "PACS.009.001.10" => Ok(MessageType::Pacs009),
            "CAMT.052" | "CAMT052" | "CAMT.052.001.10" => Ok(MessageType::Camt052),
            "CAMT.053" | "CAMT053" | "CAMT.053.001.10" => Ok(MessageType::Camt053),
            _ => Err(PaymsgError::InvalidMessageType(format!(
                "unknown message type: '{}'",
                s
            ))),
        }
    }
}

/// Message category (payment or statement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageCategory {
    /// Payment message
    Payment,
    /// Account statement message
    Statement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_is_mt() {
        assert!(MessageType::Mt103.is_mt());
        assert!(MessageType::Mt202.is_mt());
        assert!(MessageType::Mt940.is_mt());
        assert!(MessageType::Mt942.is_mt());
        assert!(!MessageType::Pacs008.is_mt());
    }

    #[test]
    fn test_message_type_is_mx() {
        assert!(MessageType::Pacs008.is_mx());
        assert!(MessageType::Pacs009.is_mx());
        assert!(MessageType::Camt052.is_mx());
        assert!(MessageType::Camt053.is_mx());
        assert!(!MessageType::Mt103.is_mx());
    }

    #[test]
    fn test_message_type_identifier() {
        assert_eq!(MessageType::Mt103.identifier(), "103");
        assert_eq!(MessageType::Pacs008.identifier(), "pacs.008");
    }

    #[test]
    fn test_message_type_full_name() {
        assert_eq!(MessageType::Mt103.full_name(), "MT103");
        assert_eq!(MessageType::Pacs008.full_name(), "pacs.008.001.10");
    }

    #[test]
    fn test_message_type_category() {
        assert_eq!(MessageType::Mt103.category(), MessageCategory::Payment);
        assert_eq!(MessageType::Mt940.category(), MessageCategory::Statement);
        assert_eq!(MessageType::Pacs008.category(), MessageCategory::Payment);
        assert_eq!(MessageType::Camt053.category(), MessageCategory::Statement);
    }

    #[test]
    fn test_message_type_equivalent() {
        assert_eq!(MessageType::Mt103.equivalent(), Some(MessageType::Pacs008));
        assert_eq!(MessageType::Pacs008.equivalent(), Some(MessageType::Mt103));
        assert_eq!(MessageType::Mt202.equivalent(), Some(MessageType::Pacs009));
        assert_eq!(MessageType::Mt940.equivalent(), Some(MessageType::Camt053));
        assert_eq!(MessageType::Mt942.equivalent(), Some(MessageType::Camt052));
    }

    #[test]
    fn test_message_type_display() {
        assert_eq!(MessageType::Mt103.to_string(), "MT103");
        assert_eq!(MessageType::Pacs008.to_string(), "pacs.008.001.10");
    }

    #[test]
    fn test_message_type_from_str() {
        assert_eq!("MT103".parse::<MessageType>().unwrap(), MessageType::Mt103);
        assert_eq!("103".parse::<MessageType>().unwrap(), MessageType::Mt103);
        assert_eq!("mt103".parse::<MessageType>().unwrap(), MessageType::Mt103);
        assert_eq!("pacs.008".parse::<MessageType>().unwrap(), MessageType::Pacs008);
        assert_eq!("PACS008".parse::<MessageType>().unwrap(), MessageType::Pacs008);
        assert_eq!("pacs.008.001.10".parse::<MessageType>().unwrap(), MessageType::Pacs008);

        assert!("invalid".parse::<MessageType>().is_err());
    }

    #[test]
    fn test_message_type_serde() {
        let msg_type = MessageType::Pacs008;
        let json = serde_json::to_string(&msg_type).unwrap();
        assert_eq!(json, "\"pacs008\"");
        let deserialized: MessageType = serde_json::from_str(&json).unwrap();
        assert_eq!(msg_type, deserialized);
    }
}
