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

/// Result type for ISO 20022 operations.
pub type Result<T> = std::result::Result<T, PaymsgError>;

/// Parse a pacs.008 message from XML string
pub fn parse_pacs008(xml: &str) -> Result<pacs008::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse pacs.008 XML: {}", e)))
}

/// Serialize a pacs.008 message to XML string
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

/// Parse a pacs.009 message from XML string
pub fn parse_pacs009(xml: &str) -> Result<pacs009::Document> {
    quick_xml::de::from_str(xml)
        .map_err(|e| PaymsgError::ParseError(format!("Failed to parse pacs.009 XML: {}", e)))
}

/// Serialize a pacs.009 message to XML string
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
