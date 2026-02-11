//! ISO 4217 currency types.

use serde::{Deserialize, Serialize};

use crate::error::{PaymsgError, Result};

/// Represents an ISO 4217 currency.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Currency {
    /// ISO 4217 3-letter currency code (e.g., "USD", "EUR", "JPY")
    pub code: String,
}

impl Currency {
    /// Creates a new Currency from a code.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Currency;
    ///
    /// let currency = Currency::new("USD").unwrap();
    /// assert_eq!(currency.code, "USD");
    /// ```
    pub fn new(code: &str) -> Result<Self> {
        // Basic validation: must be 3 uppercase letters
        if code.len() != 3 {
            return Err(PaymsgError::InvalidCurrency(format!(
                "currency code must be 3 characters, got {}",
                code.len()
            )));
        }

        if !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(PaymsgError::InvalidCurrency(format!(
                "currency code must be uppercase letters, got '{}'",
                code
            )));
        }

        Ok(Self {
            code: code.to_string(),
        })
    }

    /// Returns the currency code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.code
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::str::FromStr for Currency {
    type Err = PaymsgError;

    fn from_str(s: &str) -> Result<Self> {
        Currency::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_new_valid() {
        let currency = Currency::new("USD").unwrap();
        assert_eq!(currency.code, "USD");

        let currency = Currency::new("EUR").unwrap();
        assert_eq!(currency.code, "EUR");

        let currency = Currency::new("JPY").unwrap();
        assert_eq!(currency.code, "JPY");
    }

    #[test]
    fn test_currency_new_invalid_length() {
        assert!(Currency::new("US").is_err());
        assert!(Currency::new("USDA").is_err());
        assert!(Currency::new("").is_err());
    }

    #[test]
    fn test_currency_new_invalid_chars() {
        assert!(Currency::new("usd").is_err());
        assert!(Currency::new("Us1").is_err());
        assert!(Currency::new("U$D").is_err());
    }

    #[test]
    fn test_currency_display() {
        let currency = Currency::new("USD").unwrap();
        assert_eq!(currency.to_string(), "USD");
    }

    #[test]
    fn test_currency_from_str() {
        let currency: Currency = "EUR".parse().unwrap();
        assert_eq!(currency.code, "EUR");

        let result: Result<Currency> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_currency_serde() {
        let currency = Currency::new("GBP").unwrap();
        let json = serde_json::to_string(&currency).unwrap();
        let deserialized: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(currency, deserialized);
    }
}
