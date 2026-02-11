//! Financial amount types with precise decimal arithmetic.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{PaymsgError, Result};

/// Represents a financial amount with a currency.
///
/// Uses `rust_decimal::Decimal` for precise arithmetic, avoiding floating-point errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    /// The numeric value of the amount
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// ISO 4217 3-letter currency code (e.g., "USD", "EUR", "JPY")
    pub currency: String,
}

impl Amount {
    /// Creates a new Amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Amount;
    /// use rust_decimal::Decimal;
    ///
    /// let amount = Amount::new(Decimal::new(100050, 2), "USD".to_string());
    /// assert_eq!(amount.value.to_string(), "1000.50");
    /// ```
    pub fn new(value: Decimal, currency: String) -> Self {
        Self { value, currency }
    }

    /// Creates an Amount from a string value and currency.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Amount;
    ///
    /// let amount = Amount::from_str("1000.50", "USD").unwrap();
    /// assert_eq!(amount.currency, "USD");
    /// ```
    pub fn from_str(value: &str, currency: &str) -> Result<Self> {
        let decimal = value
            .parse::<Decimal>()
            .map_err(|e| PaymsgError::InvalidAmount(format!("failed to parse '{}': {}", value, e)))?;
        Ok(Self::new(decimal, currency.to_string()))
    }

    /// Returns true if the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    /// Returns true if the amount is positive.
    pub fn is_positive(&self) -> bool {
        self.value.is_sign_positive() && !self.value.is_zero()
    }

    /// Returns true if the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.value.is_sign_negative()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.currency, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_amount_creation() {
        let amount = Amount::new(Decimal::new(100050, 2), "USD".to_string());
        assert_eq!(amount.value.to_string(), "1000.50");
        assert_eq!(amount.currency, "USD");
    }

    #[test]
    fn test_amount_from_str() {
        let amount = Amount::from_str("1234.56", "EUR").unwrap();
        assert_eq!(amount.value.to_string(), "1234.56");
        assert_eq!(amount.currency, "EUR");
    }

    #[test]
    fn test_amount_from_str_invalid() {
        let result = Amount::from_str("not_a_number", "USD");
        assert!(result.is_err());
    }

    #[test]
    fn test_amount_zero() {
        let amount = Amount::new(Decimal::ZERO, "USD".to_string());
        assert!(amount.is_zero());
        assert!(!amount.is_positive());
        assert!(!amount.is_negative());
    }

    #[test]
    fn test_amount_positive() {
        let amount = Amount::new(Decimal::new(100, 0), "USD".to_string());
        assert!(!amount.is_zero());
        assert!(amount.is_positive());
        assert!(!amount.is_negative());
    }

    #[test]
    fn test_amount_negative() {
        let amount = Amount::new(Decimal::new(-100, 0), "USD".to_string());
        assert!(!amount.is_zero());
        assert!(!amount.is_positive());
        assert!(amount.is_negative());
    }

    #[test]
    fn test_amount_display() {
        let amount = Amount::new(Decimal::new(123456, 2), "USD".to_string());
        assert_eq!(amount.to_string(), "USD 1234.56");
    }

    #[test]
    fn test_amount_arithmetic() {
        let amount1 = Amount::new(Decimal::new(100, 0), "USD".to_string());
        let amount2 = Amount::new(Decimal::new(50, 0), "USD".to_string());

        let sum = amount1.value + amount2.value;
        assert_eq!(sum, Decimal::new(150, 0));

        let diff = amount1.value - amount2.value;
        assert_eq!(diff, Decimal::new(50, 0));
    }

    #[test]
    fn test_amount_serde() {
        let amount = Amount::new(Decimal::new(100050, 2), "USD".to_string());
        let json = serde_json::to_string(&amount).unwrap();
        let deserialized: Amount = serde_json::from_str(&json).unwrap();
        assert_eq!(amount, deserialized);
    }
}
