//! ISO 20022 (MX) message parsing and serialization.
//!
//! This crate provides XML parsing and serialization for ISO 20022 messages:
//! - pacs.008.001.10 (Customer Credit Transfer)
//! - pacs.009.001.10 (Financial Institution Credit Transfer)
//! - camt.052.001.10 (Bank to Customer Account Report)
//! - camt.053.001.10 (Bank to Customer Statement)

use paymsg_core::PaymsgError;

pub mod pacs008;
pub mod pacs009;
pub mod camt052;
pub mod camt053;

/// Result type for ISO 20022 operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

/// Parse a pacs.008 message from XML string.
///
/// Parses an ISO 20022 pacs.008.001.10 (Customer Credit Transfer) message.
///
/// # Examples
///
/// ```no_run
/// use paymsg_iso20022::parse_pacs008;
///
/// let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
/// <Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
///   <!-- message content -->
/// </Document>"#;
///
/// let doc = parse_pacs008(xml).unwrap();
/// ```
///
/// # Errors
///
/// Returns `PaymsgError::ParseError` if the XML cannot be parsed.
pub fn parse_pacs008(xml: &str) -> Result<pacs008::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse pacs.008 XML: {}", e)))
}

/// Serialize a pacs.008 message to XML string.
///
/// Serializes an ISO 20022 pacs.008.001.10 (Customer Credit Transfer) message
/// with proper XML declaration and namespaces.
///
/// # Examples
///
/// ```ignore
/// use paymsg_iso20022::{pacs008, serialize_pacs008};
///
/// // Create a pacs.008 document with proper structure
/// let doc = pacs008::Document {
///     fi_to_fi_customer_credit_transfer: pacs008::FIToFICstmrCdtTrf {
///         group_header: pacs008::GroupHeader {
///             message_id: "MSG001".to_string(),
///             creation_date_time: "2024-01-01T00:00:00".to_string(),
///             number_of_transactions: "1".to_string(),
///             settlement_information: pacs008::SettlementInformation {
///                 settlement_method: "INDA".to_string(),
///                 clearing_system: None,
///             },
///             total_interbank_settlement_amount: None,
///             instructing_agent: None,
///             instructed_agent: None,
///         },
///         credit_transfer_transaction_information: vec![],
///     },
/// };
///
/// let xml = serialize_pacs008(&doc).unwrap();
/// assert!(xml.contains("<?xml version"));
/// ```
///
/// # Errors
///
/// Returns `PaymsgError::SerializationError` if the document cannot be serialized.
pub fn serialize_pacs008(doc: &pacs008::Document) -> Result<String> {
    let mut buffer = String::new();
    buffer.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    buffer.push('\n');

    let serialized = quick_xml::se::to_string_with_root("Document", doc)
        .map_err(|e| PaymsgError::SerializationError(format!("Failed to serialize pacs.008 XML: {}", e)))?;

    // Add namespace to the Document element
    let with_ns = serialized.replace(
        "<Document>",
        r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
    );

    buffer.push_str(&with_ns);
    Ok(buffer)
}

/// Parse a pacs.009 message from XML string.
///
/// Parses an ISO 20022 pacs.009.001.10 (Financial Institution Credit Transfer) message.
///
/// # Errors
///
/// Returns `PaymsgError::ParseError` if the XML cannot be parsed.
pub fn parse_pacs009(xml: &str) -> Result<pacs009::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse pacs.009 XML: {}", e)))
}

/// Serialize a pacs.009 message to XML string.
///
/// Serializes an ISO 20022 pacs.009.001.10 (Financial Institution Credit Transfer) message
/// with proper XML declaration and namespaces.
///
/// # Errors
///
/// Returns `PaymsgError::SerializationError` if the document cannot be serialized.
pub fn serialize_pacs009(doc: &pacs009::Document) -> Result<String> {
    let mut buffer = String::new();
    buffer.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    buffer.push('\n');

    let serialized = quick_xml::se::to_string_with_root("Document", doc)
        .map_err(|e| PaymsgError::SerializationError(format!("Failed to serialize pacs.009 XML: {}", e)))?;

    // Add namespace to the Document element
    let with_ns = serialized.replace(
        "<Document>",
        r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.10" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
    );

    buffer.push_str(&with_ns);
    Ok(buffer)
}

/// Parse a camt.052 message from XML string.
///
/// Parses an ISO 20022 camt.052.001.10 (Bank to Customer Account Report) message.
///
/// # Errors
///
/// Returns `PaymsgError::ParseError` if the XML cannot be parsed.
pub fn parse_camt052(xml: &str) -> Result<camt052::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse camt.052 XML: {}", e)))
}

/// Serialize a camt.052 message to XML string.
///
/// Serializes an ISO 20022 camt.052.001.10 (Bank to Customer Account Report) message
/// with proper XML declaration and namespaces.
///
/// # Errors
///
/// Returns `PaymsgError::SerializationError` if the document cannot be serialized.
pub fn serialize_camt052(doc: &camt052::Document) -> Result<String> {
    let mut buffer = String::new();
    buffer.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    buffer.push('\n');

    let serialized = quick_xml::se::to_string_with_root("Document", doc)
        .map_err(|e| PaymsgError::SerializationError(format!("Failed to serialize camt.052 XML: {}", e)))?;

    // Add namespace to the Document element
    let with_ns = serialized.replace(
        "<Document>",
        r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.052.001.10" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
    );

    buffer.push_str(&with_ns);
    Ok(buffer)
}

/// Parse a camt.053 message from XML string.
///
/// Parses an ISO 20022 camt.053.001.10 (Bank to Customer Statement) message.
///
/// # Errors
///
/// Returns `PaymsgError::ParseError` if the XML cannot be parsed.
pub fn parse_camt053(xml: &str) -> Result<camt053::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse camt.053 XML: {}", e)))
}

/// Serialize a camt.053 message to XML string.
///
/// Serializes an ISO 20022 camt.053.001.10 (Bank to Customer Statement) message
/// with proper XML declaration and namespaces.
///
/// # Errors
///
/// Returns `PaymsgError::SerializationError` if the document cannot be serialized.
pub fn serialize_camt053(doc: &camt053::Document) -> Result<String> {
    let mut buffer = String::new();
    buffer.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    buffer.push('\n');

    let serialized = quick_xml::se::to_string_with_root("Document", doc)
        .map_err(|e| PaymsgError::SerializationError(format!("Failed to serialize camt.053 XML: {}", e)))?;

    // Add namespace to the Document element
    let with_ns = serialized.replace(
        "<Document>",
        r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.10" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
    );

    buffer.push_str(&with_ns);
    Ok(buffer)
}
