//! BIC (Business Identifier Code) types, also known as SWIFT codes.

use serde::{Deserialize, Serialize};

use crate::error::{PaymsgError, Result};

/// Represents a BIC (Business Identifier Code), also known as SWIFT code.
///
/// BIC format:
/// - 8 characters (BIC8): INST(4) + COUNTRY(2) + LOCATION(2) - head office
/// - 11 characters (BIC11): INST(4) + COUNTRY(2) + LOCATION(2) + BRANCH(3)
/// - BIC11 with branch code "XXX" also represents head office
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bic {
    /// The BIC code (8 or 11 characters)
    pub code: String,
    /// Institution code (4 characters)
    pub institution: String,
    /// ISO 3166-1 alpha-2 country code (2 characters)
    pub country: String,
    /// Location code (2 characters)
    pub location: String,
    /// Branch code (3 characters, optional - present only for BIC11)
    pub branch: Option<String>,
}

impl Bic {
    /// Creates a new BIC from a code string.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Bic;
    ///
    /// // BIC8 (head office)
    /// let bic = Bic::new("DEUTDEFF").unwrap();
    /// assert_eq!(bic.institution, "DEUT");
    /// assert_eq!(bic.country, "DE");
    /// assert_eq!(bic.location, "FF");
    /// assert_eq!(bic.branch, None);
    ///
    /// // BIC11 (with branch)
    /// let bic = Bic::new("DEUTDEFF500").unwrap();
    /// assert_eq!(bic.branch, Some("500".to_string()));
    /// ```
    pub fn new(code: &str) -> Result<Self> {
        let len = code.len();
        if len != 8 && len != 11 {
            return Err(PaymsgError::InvalidBic(format!(
                "BIC must be 8 or 11 characters, got {}",
                len
            )));
        }

        // Extract parts
        let institution = &code[0..4];
        let country = &code[4..6];
        let location = &code[6..8];
        let branch = if len == 11 {
            Some(&code[8..11])
        } else {
            None
        };

        // Validate institution: must be 4 alphabetic characters
        if !institution.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(PaymsgError::InvalidBic(format!(
                "institution code must be 4 letters, got '{}'",
                institution
            )));
        }

        // Validate country: must be 2 alphabetic characters
        if !country.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(PaymsgError::InvalidBic(format!(
                "country code must be 2 letters, got '{}'",
                country
            )));
        }

        // Validate location: must be 2 alphanumeric characters
        if !location.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PaymsgError::InvalidBic(format!(
                "location code must be 2 alphanumeric characters, got '{}'",
                location
            )));
        }

        // Validate branch (if present): must be 3 alphanumeric characters
        if let Some(branch_code) = branch {
            if !branch_code.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(PaymsgError::InvalidBic(format!(
                    "branch code must be 3 alphanumeric characters, got '{}'",
                    branch_code
                )));
            }
        }

        Ok(Self {
            code: code.to_uppercase(),
            institution: institution.to_uppercase(),
            country: country.to_uppercase(),
            location: location.to_uppercase(),
            branch: branch.map(|b| b.to_uppercase()),
        })
    }

    /// Returns true if this is a BIC8 (head office).
    pub fn is_bic8(&self) -> bool {
        self.branch.is_none()
    }

    /// Returns true if this is a BIC11.
    pub fn is_bic11(&self) -> bool {
        self.branch.is_some()
    }

    /// Returns true if this BIC represents a head office.
    /// (BIC8 or BIC11 with branch code "XXX")
    pub fn is_head_office(&self) -> bool {
        match &self.branch {
            None => true,
            Some(branch) => branch == "XXX",
        }
    }

    /// Converts this BIC to BIC8 format (strips branch code).
    pub fn to_bic8(&self) -> String {
        format!("{}{}{}", self.institution, self.country, self.location)
    }

    /// Converts this BIC to BIC11 format (adds XXX if needed).
    pub fn to_bic11(&self) -> String {
        match &self.branch {
            Some(branch) => format!(
                "{}{}{}{}",
                self.institution, self.country, self.location, branch
            ),
            None => format!("{}{}{}XXX", self.institution, self.country, self.location),
        }
    }
}

impl std::fmt::Display for Bic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::str::FromStr for Bic {
    type Err = PaymsgError;

    fn from_str(s: &str) -> Result<Self> {
        Bic::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bic8_valid() {
        let bic = Bic::new("DEUTDEFF").unwrap();
        assert_eq!(bic.code, "DEUTDEFF");
        assert_eq!(bic.institution, "DEUT");
        assert_eq!(bic.country, "DE");
        assert_eq!(bic.location, "FF");
        assert_eq!(bic.branch, None);
        assert!(bic.is_bic8());
        assert!(!bic.is_bic11());
        assert!(bic.is_head_office());
    }

    #[test]
    fn test_bic11_valid() {
        let bic = Bic::new("DEUTDEFF500").unwrap();
        assert_eq!(bic.code, "DEUTDEFF500");
        assert_eq!(bic.institution, "DEUT");
        assert_eq!(bic.country, "DE");
        assert_eq!(bic.location, "FF");
        assert_eq!(bic.branch, Some("500".to_string()));
        assert!(!bic.is_bic8());
        assert!(bic.is_bic11());
        assert!(!bic.is_head_office());
    }

    #[test]
    fn test_bic11_head_office() {
        let bic = Bic::new("DEUTDEFFXXX").unwrap();
        assert_eq!(bic.branch, Some("XXX".to_string()));
        assert!(bic.is_head_office());
    }

    #[test]
    fn test_bic_lowercase_normalized() {
        let bic = Bic::new("deutdeff").unwrap();
        assert_eq!(bic.code, "DEUTDEFF");
        assert_eq!(bic.institution, "DEUT");
    }

    #[test]
    fn test_bic_invalid_length() {
        assert!(Bic::new("DEUT").is_err());
        assert!(Bic::new("DEUTDE").is_err());
        assert!(Bic::new("DEUTDEFF5").is_err());
        assert!(Bic::new("DEUTDEFF5001").is_err());
    }

    #[test]
    fn test_bic_invalid_institution() {
        assert!(Bic::new("DE1TDEFF").is_err());
        assert!(Bic::new("DE TDEFF").is_err());
    }

    #[test]
    fn test_bic_invalid_country() {
        assert!(Bic::new("DEUTD1FF").is_err());
        assert!(Bic::new("DEUTD FF").is_err());
    }

    #[test]
    fn test_bic_invalid_location() {
        // Location can be alphanumeric, so "F1" is valid
        assert!(Bic::new("DEUTDEF ").is_err());
    }

    #[test]
    fn test_bic_to_bic8() {
        let bic = Bic::new("DEUTDEFF500").unwrap();
        assert_eq!(bic.to_bic8(), "DEUTDEFF");
    }

    #[test]
    fn test_bic_to_bic11() {
        let bic = Bic::new("DEUTDEFF").unwrap();
        assert_eq!(bic.to_bic11(), "DEUTDEFFXXX");

        let bic = Bic::new("DEUTDEFF500").unwrap();
        assert_eq!(bic.to_bic11(), "DEUTDEFF500");
    }

    #[test]
    fn test_bic_display() {
        let bic = Bic::new("DEUTDEFF").unwrap();
        assert_eq!(bic.to_string(), "DEUTDEFF");
    }

    #[test]
    fn test_bic_from_str() {
        let bic: Bic = "CHASUS33".parse().unwrap();
        assert_eq!(bic.institution, "CHAS");
        assert_eq!(bic.country, "US");

        let result: Result<Bic> = "INVALID".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_bic_serde() {
        let bic = Bic::new("CHASUS33XXX").unwrap();
        let json = serde_json::to_string(&bic).unwrap();
        let deserialized: Bic = serde_json::from_str(&json).unwrap();
        assert_eq!(bic, deserialized);
    }
}
