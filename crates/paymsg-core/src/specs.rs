//! Specification loading and registry for reference data.
//!
//! This module provides types and registries for loading reference data from the
//! paymsg-specs repository, including currencies, countries, IBAN formats, BIC specs,
//! and SWIFT character sets.

use crate::error::{PaymsgError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a currency specification from ISO 4217.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrencySpec {
    /// ISO 4217 alphabetic code (e.g., "USD", "EUR")
    pub code: String,
    /// Full name of the currency
    pub name: String,
    /// ISO 4217 numeric code (e.g., "840" for USD)
    pub numeric_code: String,
    /// Number of decimal places (e.g., 2 for USD, 0 for JPY, 3 for BHD)
    pub decimal_places: u8,
    /// Whether this currency is currently active
    pub is_active: bool,
}

/// Container for the currencies.json file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrenciesFile {
    version: String,
    last_updated: String,
    currencies: Vec<CurrencySpec>,
}

/// Represents a country specification from ISO 3166.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountrySpec {
    /// ISO 3166-1 alpha-2 code (e.g., "US", "GB")
    pub alpha2: String,
    /// ISO 3166-1 alpha-3 code (e.g., "USA", "GBR")
    pub alpha3: String,
    /// ISO 3166-1 numeric code (e.g., "840")
    pub numeric: String,
    /// Full country name
    pub name: String,
    /// Whether this country is an EU member
    pub is_eu_member: bool,
}

/// Represents IBAN format specification for a country.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IbanFormat {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: String,
    /// Country name
    pub country_name: String,
    /// Total IBAN length for this country (including country code and check digits)
    pub length: usize,
    /// Regex pattern for the BBAN (Basic Bank Account Number) part
    pub bban_format: String,
    /// Example IBAN for this country
    pub example: String,
    /// Position of the bank identifier within the BBAN
    pub bank_id_position: BankIdPosition,
}

/// Position of the bank identifier in the BBAN.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankIdPosition {
    /// Start position (0-indexed)
    pub start: usize,
    /// End position (exclusive)
    pub end: usize,
}

/// BIC structure specification component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BicComponent {
    pub name: String,
    pub position: String,
    pub length: usize,
    pub format: String,
    pub description: String,
    pub character_set: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub examples: Vec<String>,
}

/// BIC specification document structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BicSpec {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub title: String,
    pub description: String,
    pub version: String,
    pub structure: BicStructure,
}

/// BIC structure information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BicStructure {
    pub description: String,
    pub components: Vec<BicComponent>,
}

/// SWIFT character set definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwiftCharset {
    pub name: String,
    pub description: String,
    pub characters: Vec<CharacterDef>,
}

/// Individual character definition in a SWIFT character set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterDef {
    #[serde(rename = "char")]
    pub character: String,
    pub unicode: String,
    pub description: String,
}

/// Container for the swift_charsets.json file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SwiftCharsetsFile {
    charset_definitions: HashMap<String, SwiftCharset>,
}

/// Registry for looking up currency specifications.
#[derive(Debug, Clone)]
pub struct CurrencyRegistry {
    by_code: HashMap<String, CurrencySpec>,
    by_numeric: HashMap<String, CurrencySpec>,
}

impl CurrencyRegistry {
    /// Creates a new registry from a list of currency specs.
    pub fn new(currencies: Vec<CurrencySpec>) -> Self {
        let by_code = currencies
            .iter()
            .map(|c| (c.code.clone(), c.clone()))
            .collect();
        let by_numeric = currencies
            .iter()
            .map(|c| (c.numeric_code.clone(), c.clone()))
            .collect();

        Self { by_code, by_numeric }
    }

    /// Looks up a currency by its alphabetic code (e.g., "USD").
    pub fn lookup_by_code(&self, code: &str) -> Option<&CurrencySpec> {
        self.by_code.get(code)
    }

    /// Looks up a currency by its numeric code (e.g., "840").
    pub fn lookup_by_numeric(&self, numeric: &str) -> Option<&CurrencySpec> {
        self.by_numeric.get(numeric)
    }

    /// Returns all currency codes.
    pub fn all_codes(&self) -> Vec<String> {
        self.by_code.keys().cloned().collect()
    }

    /// Returns the number of currencies in the registry.
    pub fn len(&self) -> usize {
        self.by_code.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_code.is_empty()
    }
}

/// Registry for looking up country specifications.
#[derive(Debug, Clone)]
pub struct CountryRegistry {
    by_alpha2: HashMap<String, CountrySpec>,
    by_alpha3: HashMap<String, CountrySpec>,
    by_numeric: HashMap<String, CountrySpec>,
}

impl CountryRegistry {
    /// Creates a new registry from a list of country specs.
    pub fn new(countries: Vec<CountrySpec>) -> Self {
        let by_alpha2 = countries
            .iter()
            .map(|c| (c.alpha2.clone(), c.clone()))
            .collect();
        let by_alpha3 = countries
            .iter()
            .map(|c| (c.alpha3.clone(), c.clone()))
            .collect();
        let by_numeric = countries
            .iter()
            .map(|c| (c.numeric.clone(), c.clone()))
            .collect();

        Self {
            by_alpha2,
            by_alpha3,
            by_numeric,
        }
    }

    /// Looks up a country by its ISO 3166-1 alpha-2 code (e.g., "US").
    pub fn lookup_by_alpha2(&self, alpha2: &str) -> Option<&CountrySpec> {
        self.by_alpha2.get(alpha2)
    }

    /// Looks up a country by its ISO 3166-1 alpha-3 code (e.g., "USA").
    pub fn lookup_by_alpha3(&self, alpha3: &str) -> Option<&CountrySpec> {
        self.by_alpha3.get(alpha3)
    }

    /// Looks up a country by its numeric code (e.g., "840").
    pub fn lookup_by_numeric(&self, numeric: &str) -> Option<&CountrySpec> {
        self.by_numeric.get(numeric)
    }

    /// Returns all alpha-2 country codes.
    pub fn all_alpha2_codes(&self) -> Vec<String> {
        self.by_alpha2.keys().cloned().collect()
    }

    /// Returns the number of countries in the registry.
    pub fn len(&self) -> usize {
        self.by_alpha2.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_alpha2.is_empty()
    }
}

/// Registry for looking up IBAN format specifications.
#[derive(Debug, Clone)]
pub struct IbanRegistry {
    by_country: HashMap<String, IbanFormat>,
}

impl IbanRegistry {
    /// Creates a new registry from a list of IBAN format specs.
    pub fn new(formats: Vec<IbanFormat>) -> Self {
        let by_country = formats
            .iter()
            .map(|f| (f.country_code.clone(), f.clone()))
            .collect();

        Self { by_country }
    }

    /// Looks up an IBAN format by country code (e.g., "DE").
    pub fn lookup(&self, country_code: &str) -> Option<&IbanFormat> {
        self.by_country.get(country_code)
    }

    /// Returns all supported country codes for IBAN.
    pub fn supported_countries(&self) -> Vec<String> {
        self.by_country.keys().cloned().collect()
    }

    /// Returns the number of IBAN formats in the registry.
    pub fn len(&self) -> usize {
        self.by_country.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.by_country.is_empty()
    }
}

/// Main specification loader that reads JSON files from the paymsg-specs directory.
#[derive(Debug)]
pub struct SpecLoader {
    specs_dir: PathBuf,
}

impl SpecLoader {
    /// Creates a new spec loader.
    ///
    /// If `specs_dir` is None, uses the PAYMSG_SPECS_DIR environment variable,
    /// or defaults to "../paymsg-specs" relative to the workspace root.
    /// During tests, uses CARGO_MANIFEST_DIR to find the correct path.
    pub fn new(specs_dir: Option<PathBuf>) -> Self {
        let specs_dir = specs_dir.unwrap_or_else(|| {
            std::env::var("PAYMSG_SPECS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    // During tests, CARGO_MANIFEST_DIR points to the crate directory
                    // e.g., /path/to/paymsg/crates/paymsg-core
                    // We need to go up 3 levels to get to /path/to/, then into paymsg-specs
                    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                        PathBuf::from(manifest_dir)
                            .parent() // → crates/
                            .and_then(|p| p.parent()) // → workspace root
                            .and_then(|p| p.parent()) // → /path/to/code/
                            .map(|p| p.join("paymsg-specs"))
                            .unwrap_or_else(|| PathBuf::from("../paymsg-specs"))
                    } else {
                        PathBuf::from("../paymsg-specs")
                    }
                })
        });

        Self { specs_dir }
    }

    /// Returns the path to the specs directory.
    pub fn specs_dir(&self) -> &Path {
        &self.specs_dir
    }

    /// Loads currency specifications from reference/currencies.json.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use paymsg_core::SpecLoader;
    ///
    /// let loader = SpecLoader::new(None);
    /// let registry = loader.load_currencies().unwrap();
    /// let usd = registry.lookup_by_code("USD").unwrap();
    /// assert_eq!(usd.code, "USD");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if the file cannot be read or parsed.
    pub fn load_currencies(&self) -> Result<CurrencyRegistry> {
        let path = self.specs_dir.join("reference/currencies.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let file: CurrenciesFile = serde_json::from_str(&content).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(CurrencyRegistry::new(file.currencies))
    }

    /// Loads country specifications from reference/countries.json.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use paymsg_core::SpecLoader;
    ///
    /// let loader = SpecLoader::new(None);
    /// let registry = loader.load_countries().unwrap();
    /// let us = registry.lookup_by_alpha2("US").unwrap();
    /// assert_eq!(us.alpha2, "US");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if the file cannot be read or parsed.
    pub fn load_countries(&self) -> Result<CountryRegistry> {
        let path = self.specs_dir.join("reference/countries.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let countries: Vec<CountrySpec> = serde_json::from_str(&content).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(CountryRegistry::new(countries))
    }

    /// Loads IBAN format specifications from reference/iban_formats.json.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use paymsg_core::SpecLoader;
    ///
    /// let loader = SpecLoader::new(None);
    /// let registry = loader.load_iban_formats().unwrap();
    /// let de_format = registry.lookup("DE").unwrap();
    /// assert_eq!(de_format.length, 22);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if the file cannot be read or parsed.
    pub fn load_iban_formats(&self) -> Result<IbanRegistry> {
        let path = self.specs_dir.join("reference/iban_formats.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let formats: Vec<IbanFormat> = serde_json::from_str(&content).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(IbanRegistry::new(formats))
    }

    /// Loads BIC specification from reference/bic_spec.json.
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if the file cannot be read or parsed.
    pub fn load_bic_spec(&self) -> Result<BicSpec> {
        let path = self.specs_dir.join("reference/bic_spec.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let spec: BicSpec = serde_json::from_str(&content).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(spec)
    }

    /// Loads SWIFT character set definitions from reference/swift_charsets.json.
    ///
    /// Returns a map of character set names (X, Y, Z) to their definitions.
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if the file cannot be read or parsed.
    pub fn load_swift_charsets(&self) -> Result<HashMap<String, SwiftCharset>> {
        let path = self.specs_dir.join("reference/swift_charsets.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let file: SwiftCharsetsFile = serde_json::from_str(&content).map_err(|e| {
            PaymsgError::SpecLoadError {
                file: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(file.charset_definitions)
    }

    /// Loads all reference data into registries.
    ///
    /// This is a convenience method that loads currencies, countries, IBAN formats,
    /// BIC spec, and SWIFT character sets in one call.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use paymsg_core::SpecLoader;
    ///
    /// let loader = SpecLoader::new(None);
    /// let registries = loader.load_all().unwrap();
    ///
    /// // Access any registry
    /// let usd = registries.currencies.lookup_by_code("USD").unwrap();
    /// let us = registries.countries.lookup_by_alpha2("US").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `PaymsgError::SpecLoadError` if any file cannot be read or parsed.
    pub fn load_all(&self) -> Result<SpecRegistries> {
        Ok(SpecRegistries {
            currencies: self.load_currencies()?,
            countries: self.load_countries()?,
            iban_formats: self.load_iban_formats()?,
            bic_spec: self.load_bic_spec()?,
            swift_charsets: self.load_swift_charsets()?,
        })
    }
}

/// Container for all loaded specification registries.
#[derive(Debug)]
pub struct SpecRegistries {
    pub currencies: CurrencyRegistry,
    pub countries: CountryRegistry,
    pub iban_formats: IbanRegistry,
    pub bic_spec: BicSpec,
    pub swift_charsets: HashMap<String, SwiftCharset>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_currencies() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_currencies().expect("Failed to load currencies");

        assert!(!registry.is_empty());

        // Test lookup by code
        let usd = registry.lookup_by_code("USD").expect("USD not found");
        assert_eq!(usd.code, "USD");
        assert_eq!(usd.decimal_places, 2);
        assert!(usd.is_active);

        // Test zero decimal places currency
        let jpy = registry.lookup_by_code("JPY").expect("JPY not found");
        assert_eq!(jpy.decimal_places, 0);

        // Test three decimal places currency
        let bhd = registry.lookup_by_code("BHD").expect("BHD not found");
        assert_eq!(bhd.decimal_places, 3);

        // Test lookup by numeric code
        let usd_by_num = registry.lookup_by_numeric("840").expect("USD by numeric not found");
        assert_eq!(usd_by_num.code, "USD");
    }

    #[test]
    fn test_load_countries() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_countries().expect("Failed to load countries");

        assert!(!registry.is_empty());

        // Test lookup by alpha2
        let us = registry.lookup_by_alpha2("US").expect("US not found");
        assert_eq!(us.alpha2, "US");
        assert_eq!(us.alpha3, "USA");
        assert_eq!(us.numeric, "840");
        assert!(!us.is_eu_member);

        // Test lookup by alpha3
        let us_by_alpha3 = registry.lookup_by_alpha3("USA").expect("USA not found");
        assert_eq!(us_by_alpha3.alpha2, "US");

        // Test EU member
        let de = registry.lookup_by_alpha2("DE").expect("DE not found");
        assert!(de.is_eu_member);
    }

    #[test]
    fn test_load_iban_formats() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_iban_formats().expect("Failed to load IBAN formats");

        assert!(!registry.is_empty());

        // Test German IBAN
        let de = registry.lookup("DE").expect("DE IBAN format not found");
        assert_eq!(de.country_code, "DE");
        assert_eq!(de.length, 22);

        // Test GB IBAN
        let gb = registry.lookup("GB").expect("GB IBAN format not found");
        assert_eq!(gb.country_code, "GB");
        assert_eq!(gb.length, 22);
    }

    #[test]
    fn test_load_bic_spec() {
        let loader = SpecLoader::new(None);
        let spec = loader.load_bic_spec().expect("Failed to load BIC spec");

        assert_eq!(spec.version, "1.0.0");
        assert!(!spec.structure.components.is_empty());

        // Verify structure has expected components
        let component_names: Vec<_> = spec.structure.components.iter()
            .map(|c| c.name.as_str())
            .collect();
        assert!(component_names.contains(&"institution_code"));
        assert!(component_names.contains(&"country_code"));
        assert!(component_names.contains(&"location_code"));
        assert!(component_names.contains(&"branch_code"));
    }

    #[test]
    fn test_load_swift_charsets() {
        let loader = SpecLoader::new(None);
        let charsets = loader.load_swift_charsets().expect("Failed to load SWIFT charsets");

        assert!(!charsets.is_empty());

        // Check for X, Y, Z character sets
        assert!(charsets.contains_key("X"));
        assert!(charsets.contains_key("Y"));
        assert!(charsets.contains_key("Z"));

        // Verify X charset has characters
        let x_charset = &charsets["X"];
        assert!(!x_charset.characters.is_empty());
    }

    #[test]
    fn test_load_all() {
        let loader = SpecLoader::new(None);
        let registries = loader.load_all().expect("Failed to load all specs");

        assert!(!registries.currencies.is_empty());
        assert!(!registries.countries.is_empty());
        assert!(!registries.iban_formats.is_empty());
        assert!(!registries.swift_charsets.is_empty());
    }

    #[test]
    fn test_currency_registry_methods() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_currencies().expect("Failed to load currencies");

        let all_codes = registry.all_codes();
        assert!(all_codes.contains(&"USD".to_string()));
        assert!(all_codes.contains(&"EUR".to_string()));

        assert_eq!(registry.len(), all_codes.len());
    }

    #[test]
    fn test_country_registry_methods() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_countries().expect("Failed to load countries");

        let all_codes = registry.all_alpha2_codes();
        assert!(all_codes.contains(&"US".to_string()));
        assert!(all_codes.contains(&"GB".to_string()));

        assert_eq!(registry.len(), all_codes.len());
    }

    #[test]
    fn test_iban_registry_methods() {
        let loader = SpecLoader::new(None);
        let registry = loader.load_iban_formats().expect("Failed to load IBAN formats");

        let supported = registry.supported_countries();
        assert!(supported.contains(&"DE".to_string()));
        assert!(supported.contains(&"GB".to_string()));

        assert_eq!(registry.len(), supported.len());
    }
}
