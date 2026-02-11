//! Core validation types.

use serde::{Deserialize, Serialize};

/// Severity level for validation issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Fatal error that prevents message processing.
    Error,
    /// Non-fatal issue that should be reviewed.
    Warning,
    /// Informational message.
    Info,
}

/// A single validation issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Unique identifier for this issue type.
    pub id: String,
    /// Severity level.
    pub severity: Severity,
    /// Field path where the issue was found (e.g., "Block4.Field20", "GrpHdr.MsgId").
    pub field_path: Option<String>,
    /// Human-readable error message.
    pub message: String,
    /// Optional suggestion for fixing the issue.
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new error-level validation issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_validate::ValidationIssue;
    ///
    /// let issue = ValidationIssue::error("E001", "Missing mandatory field")
    ///     .with_field_path("Block4.Field20");
    /// ```
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Error,
            field_path: None,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a new warning-level validation issue.
    ///
    /// Warnings indicate non-fatal issues that should be reviewed but don't prevent processing.
    pub fn warning(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Warning,
            field_path: None,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a new info-level validation issue.
    ///
    /// Info issues are purely informational and do not affect validity.
    pub fn info(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            severity: Severity::Info,
            field_path: None,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Set the field path for this issue.
    pub fn with_field_path(mut self, field_path: impl Into<String>) -> Self {
        self.field_path = Some(field_path.into());
        self
    }

    /// Set a suggestion for this issue.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Result of validating a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// List of validation issues found.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Create a new empty validation result.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_validate::{ValidationResult, ValidationIssue};
    ///
    /// let mut result = ValidationResult::new();
    /// result.add_issue(ValidationIssue::error("E001", "Missing field"));
    /// assert!(!result.is_valid());
    /// ```
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Add an issue to the result.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Check if validation passed (no errors).
    ///
    /// Returns `true` if there are no error-level issues. Warnings and info do not affect validity.
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    /// Check if there are any error-level issues.
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    /// Get all error-level issues.
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .collect()
    }

    /// Get all warning-level issues.
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .collect()
    }

    /// Get all info-level issues.
    pub fn infos(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Info)
            .collect()
    }

    /// Merge another validation result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
    }

    /// Get the total count of issues.
    pub fn count(&self) -> usize {
        self.issues.len()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for message validators.
///
/// Implement this trait to create custom validators for message types.
///
/// # Examples
///
/// ```
/// use paymsg_validate::{Validator, ValidationResult, ValidationIssue};
///
/// struct MyMessage {
///     value: String,
/// }
///
/// struct MyValidator;
///
/// impl Validator<MyMessage> for MyValidator {
///     fn validate(&self, message: &MyMessage) -> ValidationResult {
///         let mut result = ValidationResult::new();
///         if message.value.is_empty() {
///             result.add_issue(ValidationIssue::error("E001", "Value cannot be empty"));
///         }
///         result
///     }
/// }
/// ```
pub trait Validator<T> {
    /// Validate a message and return validation results.
    ///
    /// Returns a `ValidationResult` containing all issues found during validation.
    fn validate(&self, message: &T) -> ValidationResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue_builders() {
        let error = ValidationIssue::error("TEST001", "Test error");
        assert_eq!(error.severity, Severity::Error);
        assert_eq!(error.id, "TEST001");
        assert_eq!(error.message, "Test error");

        let warning = ValidationIssue::warning("TEST002", "Test warning")
            .with_field_path("Field20")
            .with_suggestion("Use uppercase");
        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(warning.field_path, Some("Field20".to_string()));
        assert_eq!(warning.suggestion, Some("Use uppercase".to_string()));

        let info = ValidationIssue::info("TEST003", "Test info");
        assert_eq!(info.severity, Severity::Info);
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.count(), 0);

        result.add_issue(ValidationIssue::warning("W001", "Warning 1"));
        assert!(result.is_valid()); // Still valid (only warnings)
        assert_eq!(result.count(), 1);

        result.add_issue(ValidationIssue::error("E001", "Error 1"));
        assert!(!result.is_valid()); // Now invalid
        assert_eq!(result.count(), 2);
        assert_eq!(result.errors().len(), 1);
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_issue(ValidationIssue::error("E001", "Error 1"));

        let mut result2 = ValidationResult::new();
        result2.add_issue(ValidationIssue::warning("W001", "Warning 1"));

        result1.merge(result2);
        assert_eq!(result1.count(), 2);
        assert_eq!(result1.errors().len(), 1);
        assert_eq!(result1.warnings().len(), 1);
    }
}
