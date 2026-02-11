//! IBAN (International Bank Account Number) types.

use serde::{Deserialize, Serialize};

use crate::error::{PaymsgError, Result};

/// Represents an IBAN (International Bank Account Number).
///
/// IBAN format: COUNTRY(2) + CHECK(2) + BBAN(variable length)
/// Check digits are validated using the Mod-97 algorithm (ISO 7064).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Iban {
    /// The full IBAN string (without spaces)
    pub code: String,
    /// ISO 3166-1 alpha-2 country code (2 characters)
    pub country: String,
    /// Check digits (2 characters)
    pub check_digits: String,
    /// Basic Bank Account Number (BBAN) - remainder after country and check digits
    pub bban: String,
}

impl Iban {
    /// Creates a new IBAN from a code string.
    ///
    /// Validates the IBAN format and check digits using the Mod-97 algorithm (ISO 7064).
    /// Spaces are automatically removed and the code is normalized to uppercase.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Iban;
    ///
    /// let iban = Iban::new("GB82 WEST 1234 5698 7654 32").unwrap();
    /// assert_eq!(iban.country, "GB");
    /// assert_eq!(iban.check_digits, "82");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidIban` if:
    /// - Length is less than 4 or greater than 34 characters
    /// - Country code is not 2 alphabetic characters
    /// - Check digits are not 2 numeric characters
    /// - BBAN contains non-alphanumeric characters
    /// - Check digits validation fails (Mod-97 algorithm)
    pub fn new(code: &str) -> Result<Self> {
        // Remove spaces and convert to uppercase
        let normalized = code.replace(' ', "").to_uppercase();

        if normalized.len() < 4 {
            return Err(PaymsgError::InvalidIban(
                "IBAN must be at least 4 characters".to_string(),
            ));
        }

        if normalized.len() > 34 {
            return Err(PaymsgError::InvalidIban(format!(
                "IBAN must not exceed 34 characters, got {}",
                normalized.len()
            )));
        }

        // Extract parts
        let country = &normalized[0..2];
        let check_digits = &normalized[2..4];
        let bban = &normalized[4..];

        // Validate country: must be 2 alphabetic characters
        if !country.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(PaymsgError::InvalidIban(format!(
                "country code must be 2 letters, got '{}'",
                country
            )));
        }

        // Validate check digits: must be 2 numeric characters
        if !check_digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(PaymsgError::InvalidIban(format!(
                "check digits must be 2 numbers, got '{}'",
                check_digits
            )));
        }

        // Validate BBAN: must be alphanumeric
        if !bban.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PaymsgError::InvalidIban(format!(
                "BBAN must be alphanumeric, got '{}'",
                bban
            )));
        }

        let iban = Self {
            code: normalized.clone(),
            country: country.to_string(),
            check_digits: check_digits.to_string(),
            bban: bban.to_string(),
        };

        // Validate check digits using Mod-97
        if !iban.validate_check_digits() {
            return Err(PaymsgError::InvalidIban(
                "invalid check digits (Mod-97 validation failed)".to_string(),
            ));
        }

        Ok(iban)
    }

    /// Validates the check digits using the Mod-97 algorithm (ISO 7064).
    ///
    /// Algorithm:
    /// 1. Move first 4 characters to end: BBAN + COUNTRY + CHECK
    /// 2. Replace letters with numbers: A=10, B=11, ..., Z=35
    /// 3. Calculate mod 97 of the resulting number
    /// 4. Result should be 1
    pub fn validate_check_digits(&self) -> bool {
        // Step 1: Move first 4 characters to end
        let rearranged = format!("{}{}{}", self.bban, self.country, self.check_digits);

        // Step 2: Replace letters with numbers
        let mut numeric_string = String::new();
        for ch in rearranged.chars() {
            if ch.is_ascii_digit() {
                numeric_string.push(ch);
            } else if ch.is_ascii_alphabetic() {
                // A=10, B=11, ..., Z=35
                let value = (ch as u8 - b'A') + 10;
                numeric_string.push_str(&value.to_string());
            } else {
                return false;
            }
        }

        // Step 3: Calculate mod 97
        // For large numbers, we need to process in chunks
        let remainder = Self::mod97(&numeric_string);

        // Step 4: Result should be 1
        remainder == 1
    }

    /// Calculates mod 97 for a large numeric string.
    fn mod97(s: &str) -> u32 {
        let mut remainder = 0u32;
        for ch in s.chars() {
            let digit = ch.to_digit(10).unwrap();
            remainder = (remainder * 10 + digit) % 97;
        }
        remainder
    }

    /// Formats the IBAN with spaces every 4 characters for readability.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Iban;
    ///
    /// let iban = Iban::new("GB82WEST12345698765432").unwrap();
    /// assert_eq!(iban.format_pretty(), "GB82 WEST 1234 5698 7654 32");
    /// ```
    pub fn format_pretty(&self) -> String {
        self.code
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 4 == 0 {
                    vec![' ', c]
                } else {
                    vec![c]
                }
            })
            .collect()
    }
}

impl std::fmt::Display for Iban {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::str::FromStr for Iban {
    type Err = PaymsgError;

    fn from_str(s: &str) -> Result<Self> {
        Iban::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iban_valid() {
        // Valid IBANs from different countries
        let iban = Iban::new("GB82WEST12345698765432").unwrap();
        assert_eq!(iban.country, "GB");
        assert_eq!(iban.check_digits, "82");
        assert_eq!(iban.bban, "WEST12345698765432");

        // With spaces
        let iban = Iban::new("GB82 WEST 1234 5698 7654 32").unwrap();
        assert_eq!(iban.code, "GB82WEST12345698765432");
    }

    #[test]
    fn test_iban_valid_german() {
        let iban = Iban::new("DE89370400440532013000").unwrap();
        assert_eq!(iban.country, "DE");
        assert_eq!(iban.check_digits, "89");
    }

    #[test]
    fn test_iban_valid_french() {
        let iban = Iban::new("FR1420041010050500013M02606").unwrap();
        assert_eq!(iban.country, "FR");
        assert_eq!(iban.check_digits, "14");
    }

    #[test]
    fn test_iban_invalid_too_short() {
        assert!(Iban::new("GB8").is_err());
    }

    #[test]
    fn test_iban_invalid_too_long() {
        let long_iban = "GB82WEST123456987654321234567890123";
        assert!(Iban::new(long_iban).is_err());
    }

    #[test]
    fn test_iban_invalid_country() {
        assert!(Iban::new("G282WEST12345698765432").is_err());
        assert!(Iban::new("1B82WEST12345698765432").is_err());
    }

    #[test]
    fn test_iban_invalid_check_digits() {
        assert!(Iban::new("GBA2WEST12345698765432").is_err());
    }

    #[test]
    fn test_iban_invalid_check_digits_mod97() {
        // Valid format but wrong check digits
        assert!(Iban::new("GB83WEST12345698765432").is_err());
    }

    #[test]
    fn test_iban_format_pretty() {
        let iban = Iban::new("GB82WEST12345698765432").unwrap();
        assert_eq!(iban.format_pretty(), "GB82 WEST 1234 5698 7654 32");
    }

    #[test]
    fn test_iban_display() {
        let iban = Iban::new("GB82WEST12345698765432").unwrap();
        assert_eq!(iban.to_string(), "GB82WEST12345698765432");
    }

    #[test]
    fn test_iban_from_str() {
        let iban: Iban = "DE89370400440532013000".parse().unwrap();
        assert_eq!(iban.country, "DE");

        let result: Result<Iban> = "INVALID".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_iban_mod97() {
        assert_eq!(Iban::mod97("1"), 1);
        assert_eq!(Iban::mod97("97"), 0);
        assert_eq!(Iban::mod97("98"), 1);
        assert_eq!(Iban::mod97("123456789"), 123456789 % 97);
    }

    #[test]
    fn test_iban_serde() {
        let iban = Iban::new("GB82WEST12345698765432").unwrap();
        let json = serde_json::to_string(&iban).unwrap();
        let deserialized: Iban = serde_json::from_str(&json).unwrap();
        assert_eq!(iban, deserialized);
    }
}
