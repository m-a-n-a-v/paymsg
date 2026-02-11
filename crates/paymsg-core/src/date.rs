//! Date and time types for financial messages.

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::error::{PaymsgError, Result};

/// Represents a date in financial messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Date {
    #[serde(with = "date_format")]
    pub inner: NaiveDate,
}

impl Date {
    /// Creates a new Date from year, month, and day.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Date;
    ///
    /// let date = Date::from_ymd(2024, 1, 15).unwrap();
    /// assert_eq!(date.year(), 2024);
    /// assert_eq!(date.month(), 1);
    /// assert_eq!(date.day(), 15);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidDate` if the date is not valid (e.g., February 30).
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self> {
        NaiveDate::from_ymd_opt(year, month, day)
            .map(|inner| Self { inner })
            .ok_or_else(|| {
                PaymsgError::InvalidDate(format!("invalid date: {}-{:02}-{:02}", year, month, day))
            })
    }

    /// Parses a date from YYYYMMDD format (common in SWIFT MT).
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Date;
    ///
    /// let date = Date::from_yyyymmdd("20240115").unwrap();
    /// assert_eq!(date.year(), 2024);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidDate` if:
    /// - String is not exactly 8 characters
    /// - Characters are not valid digits
    /// - The resulting date is not valid
    pub fn from_yyyymmdd(s: &str) -> Result<Self> {
        if s.len() != 8 {
            return Err(PaymsgError::InvalidDate(format!(
                "YYYYMMDD must be 8 characters, got {}",
                s.len()
            )));
        }

        let year = s[0..4].parse::<i32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid year in YYYYMMDD: {}", s))
        })?;
        let month = s[4..6].parse::<u32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid month in YYYYMMDD: {}", s))
        })?;
        let day = s[6..8].parse::<u32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid day in YYYYMMDD: {}", s))
        })?;

        Self::from_ymd(year, month, day)
    }

    /// Parses a date from YYMMDD format (common in SWIFT MT).
    ///
    /// Assumes 20XX for years 00-99.
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Date;
    ///
    /// let date = Date::from_yymmdd("240115").unwrap();
    /// assert_eq!(date.year(), 2024);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidDate` if:
    /// - String is not exactly 6 characters
    /// - Characters are not valid digits
    /// - The resulting date is not valid
    pub fn from_yymmdd(s: &str) -> Result<Self> {
        if s.len() != 6 {
            return Err(PaymsgError::InvalidDate(format!(
                "YYMMDD must be 6 characters, got {}",
                s.len()
            )));
        }

        let yy = s[0..2].parse::<i32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid year in YYMMDD: {}", s))
        })?;
        let month = s[2..4].parse::<u32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid month in YYMMDD: {}", s))
        })?;
        let day = s[4..6].parse::<u32>().map_err(|_| {
            PaymsgError::InvalidDate(format!("invalid day in YYMMDD: {}", s))
        })?;

        let year = 2000 + yy;

        Self::from_ymd(year, month, day)
    }

    /// Parses a date from ISO 8601 format (YYYY-MM-DD).
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::Date;
    ///
    /// let date = Date::from_iso8601("2024-01-15").unwrap();
    /// assert_eq!(date.year(), 2024);
    /// assert_eq!(date.month(), 1);
    /// assert_eq!(date.day(), 15);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidDate` if the string cannot be parsed as ISO 8601 date.
    pub fn from_iso8601(s: &str) -> Result<Self> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(|inner| Self { inner })
            .map_err(|e| PaymsgError::InvalidDate(format!("invalid ISO 8601 date '{}': {}", s, e)))
    }

    /// Formats the date as YYYYMMDD.
    pub fn to_yyyymmdd(&self) -> String {
        self.inner.format("%Y%m%d").to_string()
    }

    /// Formats the date as YYMMDD.
    pub fn to_yymmdd(&self) -> String {
        self.inner.format("%y%m%d").to_string()
    }

    /// Formats the date as ISO 8601 (YYYY-MM-DD).
    pub fn to_iso8601(&self) -> String {
        self.inner.format("%Y-%m-%d").to_string()
    }

    /// Returns the year.
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Returns the month (1-12).
    pub fn month(&self) -> u32 {
        self.inner.month()
    }

    /// Returns the day of month (1-31).
    pub fn day(&self) -> u32 {
        self.inner.day()
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

/// Represents a date and time in financial messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime {
    #[serde(with = "datetime_format")]
    pub inner: NaiveDateTime,
}

impl DateTime {
    /// Creates a new DateTime from a NaiveDateTime.
    pub fn new(inner: NaiveDateTime) -> Self {
        Self { inner }
    }

    /// Parses a DateTime from ISO 8601 format.
    ///
    /// Accepts formats with or without fractional seconds:
    /// - `2024-01-15T10:30:45`
    /// - `2024-01-15T10:30:45.123`
    ///
    /// # Examples
    ///
    /// ```
    /// use paymsg_core::DateTime;
    ///
    /// let dt = DateTime::from_iso8601("2024-01-15T10:30:45").unwrap();
    /// assert_eq!(dt.to_iso8601(), "2024-01-15T10:30:45");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::InvalidDate` if the string cannot be parsed as ISO 8601 datetime.
    pub fn from_iso8601(s: &str) -> Result<Self> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
            .map(|inner| Self { inner })
            .map_err(|e| {
                PaymsgError::InvalidDate(format!("invalid ISO 8601 datetime '{}': {}", s, e))
            })
    }

    /// Formats the DateTime as ISO 8601.
    pub fn to_iso8601(&self) -> String {
        self.inner.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

// Custom serde modules for date formatting
mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S>(datetime: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = datetime.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT)
            .or_else(|_| NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f"))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_date_from_ymd() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_date_from_ymd_invalid() {
        assert!(Date::from_ymd(2024, 13, 1).is_err());
        assert!(Date::from_ymd(2024, 2, 30).is_err());
    }

    #[test]
    fn test_date_from_yyyymmdd() {
        let date = Date::from_yyyymmdd("20240115").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_date_from_yymmdd() {
        let date = Date::from_yymmdd("240115").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_date_from_iso8601() {
        let date = Date::from_iso8601("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_date_to_yyyymmdd() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        assert_eq!(date.to_yyyymmdd(), "20240115");
    }

    #[test]
    fn test_date_to_yymmdd() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        assert_eq!(date.to_yymmdd(), "240115");
    }

    #[test]
    fn test_date_to_iso8601() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        assert_eq!(date.to_iso8601(), "2024-01-15");
    }

    #[test]
    fn test_date_display() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        assert_eq!(date.to_string(), "2024-01-15");
    }

    #[test]
    fn test_date_serde() {
        let date = Date::from_ymd(2024, 1, 15).unwrap();
        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, "\"2024-01-15\"");
        let deserialized: Date = serde_json::from_str(&json).unwrap();
        assert_eq!(date, deserialized);
    }

    #[test]
    fn test_datetime_from_iso8601() {
        let dt = DateTime::from_iso8601("2024-01-15T10:30:45").unwrap();
        assert_eq!(dt.inner.hour(), 10);
        assert_eq!(dt.inner.minute(), 30);
        assert_eq!(dt.inner.second(), 45);
    }

    #[test]
    fn test_datetime_to_iso8601() {
        let dt = DateTime::from_iso8601("2024-01-15T10:30:45").unwrap();
        assert_eq!(dt.to_iso8601(), "2024-01-15T10:30:45");
    }

    #[test]
    fn test_datetime_serde() {
        let dt = DateTime::from_iso8601("2024-01-15T10:30:45").unwrap();
        let json = serde_json::to_string(&dt).unwrap();
        let deserialized: DateTime = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, deserialized);
    }
}
