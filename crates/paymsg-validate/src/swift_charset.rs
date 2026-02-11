//! SWIFT character set validation.

use crate::{ValidationIssue, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// SWIFT character set definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwiftCharsets {
    /// Character set X: Alphanumeric and basic punctuation.
    pub x: HashSet<char>,
    /// Character set Y: X plus additional characters.
    pub y: HashSet<char>,
    /// Character set Z: Y plus additional characters.
    pub z: HashSet<char>,
}

/// Character set specification from JSON.
#[derive(Debug, Clone, Deserialize)]
struct CharsetSpec {
    charset_definitions: CharsetDefinitions,
}

#[derive(Debug, Clone, Deserialize)]
struct CharsetDefinitions {
    #[serde(rename = "X")]
    x: CharsetDef,
    #[serde(rename = "Y")]
    y: CharsetDef,
    #[serde(rename = "Z")]
    z: CharsetDef,
}

#[derive(Debug, Clone, Deserialize)]
struct CharsetDef {
    characters: Vec<CharacterEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct CharacterEntry {
    #[serde(rename = "char")]
    character: String,
}

impl SwiftCharsets {
    /// Load SWIFT character sets from the spec directory.
    pub fn load(specs_dir: &Path) -> Result<Self, paymsg_core::PaymsgError> {
        let charset_path = specs_dir.join("reference/swift_charsets.json");
        let json_str = std::fs::read_to_string(&charset_path).map_err(|e| {
            paymsg_core::PaymsgError::SpecLoadError {
                file: charset_path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let spec: CharsetSpec = serde_json::from_str(&json_str).map_err(|e| {
            paymsg_core::PaymsgError::SpecLoadError {
                file: charset_path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        // Convert character entries to HashSets
        let x: HashSet<char> = spec
            .charset_definitions
            .x
            .characters
            .iter()
            .filter_map(|e| e.character.chars().next())
            .collect();

        let y: HashSet<char> = spec
            .charset_definitions
            .y
            .characters
            .iter()
            .filter_map(|e| e.character.chars().next())
            .collect();

        let z: HashSet<char> = spec
            .charset_definitions
            .z
            .characters
            .iter()
            .filter_map(|e| e.character.chars().next())
            .collect();

        Ok(Self { x, y, z })
    }

    /// Create charsets with default SWIFT X, Y, Z sets (for testing/fallback).
    pub fn default_sets() -> Self {
        // SWIFT X: Alphanumeric and basic punctuation
        let x_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 /-?:().,'+"
            .chars()
            .collect();

        // SWIFT Y: X plus additional characters
        let y_chars: HashSet<char> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 /-?:().,'+=!\"#$%&*;<>@[\\]^_`{|}~\r\n"
                .chars()
                .collect();

        // SWIFT Z: Y plus extended characters
        let z_chars = {
            let mut set: HashSet<char> = y_chars.clone();
            // Add extended Latin characters commonly used in European names/addresses
            for c in "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿ".chars() {
                set.insert(c);
            }
            set
        };

        Self {
            x: x_chars,
            y: y_chars,
            z: z_chars,
        }
    }

    /// Validate that a string contains only SWIFT X characters.
    pub fn validate_x(&self, value: &str) -> Option<char> {
        value.chars().find(|c| !self.x.contains(c))
    }

    /// Validate that a string contains only SWIFT Y characters.
    pub fn validate_y(&self, value: &str) -> Option<char> {
        value.chars().find(|c| !self.y.contains(c))
    }

    /// Validate that a string contains only SWIFT Z characters.
    pub fn validate_z(&self, value: &str) -> Option<char> {
        value.chars().find(|c| !self.z.contains(c))
    }
}

/// Validator for SWIFT character sets.
pub struct SwiftCharsetValidator {
    charsets: SwiftCharsets,
}

impl SwiftCharsetValidator {
    /// Create a new validator with loaded character sets.
    pub fn new(charsets: SwiftCharsets) -> Self {
        Self { charsets }
    }

    /// Validate a field value against a specific SWIFT character set.
    pub fn validate_field(
        &self,
        field_tag: &str,
        value: &str,
        charset: char,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        let invalid_char = match charset {
            'X' | 'x' => self.charsets.validate_x(value),
            'Y' | 'y' => self.charsets.validate_y(value),
            'Z' | 'z' => self.charsets.validate_z(value),
            _ => {
                result.add_issue(
                    ValidationIssue::error(
                        "CHARSET_UNKNOWN",
                        format!("Unknown character set: {}", charset),
                    )
                    .with_field_path(field_tag),
                );
                return result;
            }
        };

        if let Some(invalid) = invalid_char {
            result.add_issue(
                ValidationIssue::error(
                    "CHARSET_INVALID_CHAR",
                    format!(
                        "Invalid character '{}' (U+{:04X}) for SWIFT charset {}",
                        invalid, invalid as u32, charset
                    ),
                )
                .with_field_path(field_tag)
                .with_suggestion(format!(
                    "Field must contain only SWIFT {} character set",
                    charset
                )),
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_charsets() {
        let charsets = SwiftCharsets::default_sets();

        // Test X charset
        assert!(charsets.validate_x("ABC123").is_none());
        assert!(charsets.validate_x("Test-Ref/001").is_none());
        assert!(charsets.validate_x("Amount: 1,234.56").is_none());
        assert_eq!(charsets.validate_x("Test@Email"), Some('@')); // @ not in X

        // Test Y charset
        assert!(charsets.validate_y("ABC123").is_none());
        assert!(charsets.validate_y("Test@Email").is_none());
        assert!(charsets.validate_y("Line1\r\nLine2").is_none());

        // Test Z charset
        assert!(charsets.validate_z("ABC123").is_none());
        assert!(charsets.validate_z("Müller").is_none()); // German umlaut
        assert!(charsets.validate_z("Café").is_none()); // French accent
    }

    #[test]
    fn test_charset_validator() {
        let charsets = SwiftCharsets::default_sets();
        let validator = SwiftCharsetValidator::new(charsets);

        // Valid X charset
        let result = validator.validate_field("20", "REF123", 'X');
        assert!(result.is_valid());

        // Invalid X charset (contains @)
        let result = validator.validate_field("20", "REF@123", 'X');
        assert!(!result.is_valid());
        assert_eq!(result.errors().len(), 1);
        assert_eq!(result.errors()[0].id, "CHARSET_INVALID_CHAR");

        // Valid Y charset with @
        let result = validator.validate_field("70", "Email@test.com", 'Y');
        assert!(result.is_valid());

        // Unknown charset
        let result = validator.validate_field("20", "TEST", 'Q');
        assert!(!result.is_valid());
        assert_eq!(result.errors()[0].id, "CHARSET_UNKNOWN");
    }

    #[test]
    fn test_load_charsets_from_file() {
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
                let charsets = SwiftCharsets::load(&specs_dir);
                assert!(charsets.is_ok(), "Failed to load charsets: {:?}", charsets.err());

                let charsets = charsets.unwrap();
                // Verify X charset contains expected characters
                assert!(charsets.x.contains(&'A'));
                assert!(charsets.x.contains(&'0'));
                assert!(charsets.x.contains(&'/'));
                assert!(!charsets.x.contains(&'@')); // @ not in X

                // Verify Y charset is loaded
                assert!(!charsets.y.is_empty());
            }
        }
    }
}
