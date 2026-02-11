//! MT message schema validation.

use crate::swift_charset::SwiftCharsetValidator;
use crate::{ValidationIssue, ValidationResult, Validator};
use paymsg_core::PaymsgError;
use paymsg_mt::fields::MtFieldSpec;
use paymsg_mt::{MtMessage, MtMessageSpec};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// MT schema validator.
pub struct MtSchemaValidator {
    /// Loaded MT message specifications.
    specs: HashMap<String, MtMessageSpec>,
    /// SWIFT character set validator.
    charset_validator: SwiftCharsetValidator,
}

impl MtSchemaValidator {
    /// Load MT specifications from the specs directory.
    pub fn load(specs_dir: &Path, charset_validator: SwiftCharsetValidator) -> Result<Self, PaymsgError> {
        let mt103_spec = paymsg_mt::load_mt_spec(&specs_dir.join("mt-specs/mt103.json"))?;
        let mt202_spec = paymsg_mt::load_mt_spec(&specs_dir.join("mt-specs/mt202.json"))?;
        let mt940_spec = paymsg_mt::load_mt_spec(&specs_dir.join("mt-specs/mt940.json"))?;
        let mt942_spec = paymsg_mt::load_mt_spec(&specs_dir.join("mt-specs/mt942.json"))?;

        let mut specs = HashMap::new();
        specs.insert("MT103".to_string(), mt103_spec);
        specs.insert("MT202".to_string(), mt202_spec);
        specs.insert("MT940".to_string(), mt940_spec);
        specs.insert("MT942".to_string(), mt942_spec);

        Ok(Self {
            specs,
            charset_validator,
        })
    }

    /// Get the specification for a message type.
    fn get_spec(&self, message_type: &str) -> Option<&MtMessageSpec> {
        self.specs.get(message_type)
    }

    /// Validate a field against its specification.
    fn validate_field(
        &self,
        field_spec: &MtFieldSpec,
        field_value: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check max length
        let max_len = field_spec.max_length;
        if max_len > 0
            && field_value.len() > max_len {
                result.add_issue(
                    ValidationIssue::error(
                        "MT_FIELD_TOO_LONG",
                        format!(
                            "Field {} exceeds maximum length {} (actual: {})",
                            field_spec.tag,
                            max_len,
                            field_value.len()
                        ),
                    )
                    .with_field_path(format!("Block4.Field{}", field_spec.tag))
                    .with_suggestion(format!("Maximum length is {} characters", max_len)),
                );
            }

        // Check format pattern
        if let Some(pattern) = &field_spec.format_pattern {
            match Regex::new(pattern) {
                Ok(regex) => {
                    if !regex.is_match(field_value) {
                        result.add_issue(
                            ValidationIssue::error(
                                "MT_FIELD_INVALID_FORMAT",
                                format!(
                                    "Field {} does not match required format pattern",
                                    field_spec.tag
                                ),
                            )
                            .with_field_path(format!("Block4.Field{}", field_spec.tag))
                            .with_suggestion(format!("Expected format: {}", pattern)),
                        );
                    }
                }
                Err(e) => {
                    result.add_issue(
                        ValidationIssue::warning(
                            "MT_INVALID_REGEX",
                            format!(
                                "Invalid regex pattern in spec for field {}: {}",
                                field_spec.tag, e
                            ),
                        )
                        .with_field_path(format!("Block4.Field{}", field_spec.tag)),
                    );
                }
            }
        }

        // Validate SWIFT character set
        // Extract charset from description (e.g., "SWIFT character set X")
        let desc = &field_spec.description;
        if !desc.is_empty() {
            if let Some(charset) = extract_charset_from_description(desc) {
                let charset_result =
                    self.charset_validator
                        .validate_field(&field_spec.tag, field_value, charset);
                result.merge(charset_result);
            }
        }

        result
    }
}

impl Validator<MtMessage> for MtSchemaValidator {
    fn validate(&self, message: &MtMessage) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Determine message type from Block 2
        let msg_type = &message.block2.message_type;
        let msg_type_key = format!("MT{}", msg_type);

        // Get the specification for this message type
        let spec = match self.get_spec(&msg_type_key) {
            Some(s) => s,
            None => {
                result.add_issue(ValidationIssue::error(
                    "MT_UNKNOWN_MESSAGE_TYPE",
                    format!("Unknown message type: {}", msg_type_key),
                ));
                return result;
            }
        };

        // Parse Block 4 fields
        let fields = match message.block4.parse_fields() {
            Ok(f) => f,
            Err(e) => {
                result.add_issue(
                    ValidationIssue::error("MT_FIELD_PARSE_ERROR", format!("Failed to parse Block 4 fields: {}", e))
                        .with_field_path("Block4"),
                );
                return result;
            }
        };

        // Build a map of present fields
        let mut field_map: HashMap<String, Vec<String>> = HashMap::new();
        for field in &fields {
            field_map
                .entry(field.tag.clone())
                .or_default()
                .push(field.value.clone());
        }

        // Check mandatory fields
        for field_spec in &spec.fields {
            if field_spec.status == "M" {
                // Mandatory field
                if !field_map.contains_key(&field_spec.tag) {
                    result.add_issue(
                        ValidationIssue::error(
                            "MT_MISSING_MANDATORY_FIELD",
                            format!("Missing mandatory field: {}", field_spec.tag),
                        )
                        .with_field_path(format!("Block4.Field{}", field_spec.tag))
                        .with_suggestion(format!("Field {} ({}) is required", field_spec.tag, field_spec.name)),
                    );
                }
            }
        }

        // Validate each present field
        for (tag, values) in &field_map {
            // Find the spec for this field
            if let Some(field_spec) = spec.fields.iter().find(|f| &f.tag == tag) {
                // Validate each occurrence
                for value in values {
                    let field_result = self.validate_field(field_spec, value);
                    result.merge(field_result);
                }
            } else {
                // Unknown field
                result.add_issue(
                    ValidationIssue::warning(
                        "MT_UNKNOWN_FIELD",
                        format!("Unknown field tag: {}", tag),
                    )
                    .with_field_path(format!("Block4.Field{}", tag)),
                );
            }
        }

        result
    }
}

/// Extract SWIFT character set from field description.
fn extract_charset_from_description(desc: &str) -> Option<char> {
    let desc_lower = desc.to_lowercase();
    if desc_lower.contains("character set x") || desc_lower.contains("charset x") {
        Some('X')
    } else if desc_lower.contains("character set y") || desc_lower.contains("charset y") {
        Some('Y')
    } else if desc_lower.contains("character set z") || desc_lower.contains("charset z") {
        Some('Z')
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swift_charset::SwiftCharsets;

    #[test]
    fn test_extract_charset_from_description() {
        assert_eq!(
            extract_charset_from_description("SWIFT character set X"),
            Some('X')
        );
        assert_eq!(
            extract_charset_from_description("Uses SWIFT Character Set Y for encoding"),
            Some('Y')
        );
        assert_eq!(
            extract_charset_from_description("This field uses charset Z"),
            Some('Z')
        );
        assert_eq!(
            extract_charset_from_description("No charset specified"),
            None
        );
    }

    #[test]
    fn test_mt_schema_validator_load() {
        // Try to load from specs directory
        let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .and_then(|p| {
                let path = Path::new(&p);
                // Navigate from crates/paymsg-validate to workspace root
                path.parent()?.parent().map(|p| p.to_path_buf())
            });

        if let Some(root) = workspace_root {
            let specs_dir = root.parent().unwrap().join("paymsg-specs");
            if specs_dir.exists() {
                let charsets = SwiftCharsets::load(&specs_dir).unwrap();
                let charset_validator = SwiftCharsetValidator::new(charsets);

                let validator = MtSchemaValidator::load(&specs_dir, charset_validator);
                assert!(validator.is_ok(), "Failed to load validator: {:?}", validator.err());

                let validator = validator.unwrap();
                assert!(validator.get_spec("MT103").is_some());
                assert!(validator.get_spec("MT202").is_some());
                assert!(validator.get_spec("MT940").is_some());
                assert!(validator.get_spec("MT942").is_some());
            }
        }
    }
}
