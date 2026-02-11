# Ralph Agent Instructions

You are an autonomous coding agent building `paymsg` — a production-grade Rust library and CLI tool for parsing, validating, translating, and generating ISO 20022 (MX) and SWIFT MT financial messages.

## Your Task

1. Read the PRD at `scripts/ralph/prd.json`
2. Read the progress log at `scripts/ralph/progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run quality checks: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`
7. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
8. Update the PRD to set `passes: true` for the completed story
9. Append your progress to `scripts/ralph/progress.txt`

## Project Architecture

This is a Rust workspace with 7 crates:

```
paymsg/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── paymsg-core/            # Core types, traits, error handling
│   ├── paymsg-iso20022/        # ISO 20022 XML parser + serializer
│   ├── paymsg-mt/              # SWIFT MT parser + serializer
│   ├── paymsg-validate/        # Schema + business rule validation engine
│   ├── paymsg-translate/       # MT ↔ MX translation engine
│   ├── paymsg-codegen/         # Build-time XSD → Rust type generator (optional, stretch)
│   └── paymsg-cli/             # CLI binary (clap-based)
├── specs/                      # Git submodule or symlink → paymsg-specs repo
└── scripts/ralph/              # Ralph loop automation
```

### Crate Dependency Graph

```
paymsg-core         ← Foundation: no deps on other paymsg crates
paymsg-mt           ← depends on paymsg-core
paymsg-iso20022     ← depends on paymsg-core
paymsg-validate     ← depends on paymsg-core, paymsg-mt, paymsg-iso20022
paymsg-translate    ← depends on paymsg-core, paymsg-mt, paymsg-iso20022, paymsg-validate
paymsg-codegen      ← depends on paymsg-core (build-time only, optional)
paymsg-cli          ← depends on all above
```

## Sibling Repository: paymsg-specs

The specs data lives in a sibling repo at `../paymsg-specs`. The Rust code loads JSON spec files at runtime (or compile time for embedded data). Key paths:

- `../paymsg-specs/reference/currencies.json` — ISO 4217 currency data (code, decimal_places, etc.)
- `../paymsg-specs/reference/countries.json` — ISO 3166 country data
- `../paymsg-specs/reference/iban_formats.json` — IBAN format per country (length, bban_format regex)
- `../paymsg-specs/reference/bic_spec.json` — BIC structure and validation rules
- `../paymsg-specs/reference/swift_charsets.json` — SWIFT character set definitions (X, Y, Z)
- `../paymsg-specs/mt-specs/mt103.json` — MT103 field definitions (tags, lengths, formats, subfields)
- `../paymsg-specs/mt-specs/mt202.json` — MT202 field definitions
- `../paymsg-specs/mt-specs/mt940.json` — MT940 field definitions
- `../paymsg-specs/mt-specs/mt942.json` — MT942 field definitions
- `../paymsg-specs/mt-specs/block_structure.json` — SWIFT MT 5-block structure spec
- `../paymsg-specs/mappings/mt103_pacs008.json` — MT103 ↔ pacs.008 field mappings
- `../paymsg-specs/mappings/mt202_pacs009.json` — MT202 ↔ pacs.009 field mappings
- `../paymsg-specs/mappings/mt940_camt053.json` — MT940 ↔ camt.053 field mappings
- `../paymsg-specs/mappings/mt942_camt052.json` — MT942 ↔ camt.052 field mappings
- `../paymsg-specs/rules/pacs008_rules.json` — Business validation rules for pacs.008
- `../paymsg-specs/rules/pacs009_rules.json` — Business validation rules for pacs.009
- `../paymsg-specs/rules/camt052_rules.json` — Business validation rules for camt.052
- `../paymsg-specs/rules/camt053_rules.json` — Business validation rules for camt.053
- `../paymsg-specs/testdata/` — Sample MT and MX messages, translation pairs

### Loading Specs in Rust

For spec files, use `include_str!` to embed at compile time OR load from disk at runtime. Prefer embedding small reference data (currencies, countries) and loading large specs at runtime. Use `PAYMSG_SPECS_DIR` env var or fall back to `../paymsg-specs`.

## Domain Knowledge

### ISO 20022 Message Types We Support

| MX Type | Description | MT Equivalent |
|---------|-------------|---------------|
| pacs.008.001.10 | Customer Credit Transfer | MT103 |
| pacs.009.001.10 | FI Credit Transfer | MT202 |
| camt.053.001.10 | Bank-to-Customer Statement | MT940 |
| camt.052.001.10 | Bank-to-Customer Interim Report | MT942 |

### SWIFT MT Message Structure

MT messages have 5 blocks:
- Block 1: `{1:F01BANKBICAXXX0000000000}` — Basic Header (app_id, service_id, LT address, session, sequence)
- Block 2: `{2:I103BANKBICAXXXXN}` — Application Header (I=Input/O=Output, msg type, BIC, priority)
- Block 3: `{3:{108:MUR}{121:UUID}}` — User Header (optional tag-value pairs)
- Block 4: `{4:\r\n:20:REF\r\n:32A:...\r\n-}` — Text Block (field tags with values, ends with dash)
- Block 5: `{5:{CHK:...}}` — Trailer (optional checksums)

### MT Field Format
- Tags: `:NN[a]:value` where NN is 2 digits, optional letter suffix (e.g., `:20:`, `:50K:`, `:32A:`)
- Multi-line values continue on next line without tag prefix
- Fields with options (A/B/D/F/K) have different subfield structures
- Field 32A compound: `YYMMDDCCCAMOUNT` (date + currency + amount)
- Amount format: comma as decimal separator (e.g., `1234,56`)

### Key BIC Facts
- 8 or 11 characters: INST(4) + COUNTRY(2) + LOCATION(2) + [BRANCH(3)]
- BIC8 = head office; BIC11 with XXX = also head office
- Validate: institution=alpha(4), country=valid ISO 3166, location=alphanum(2), branch=alphanum(3)

### Key IBAN Facts
- Format: COUNTRY(2) + CHECK(2) + BBAN(variable length)
- Check digits: Mod-97 (ISO 7064) — move first 4 to end, convert letters to numbers, mod 97 = 1
- Length varies by country (loaded from iban_formats.json)

### Key Currency Facts (ISO 4217)
- Most: 2 decimal places (USD, EUR, GBP)
- Zero decimals: JPY, KRW, VND
- Three decimals: BHD, KWD, OMR, JOD

### XML Namespaces
- pacs.008: `urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10`
- pacs.009: `urn:iso:std:iso:20022:tech:xsd:pacs.009.001.10`
- camt.052: `urn:iso:std:iso:20022:tech:xsd:camt.052.001.10`
- camt.053: `urn:iso:std:iso:20022:tech:xsd:camt.053.001.10`

### Translation Transform Types
From mapping JSON files, transforms are one of: `direct`, `split`, `merge`, `lookup`, `derived`, `conditional`
- **direct**: 1:1 value copy (possibly with format conversion)
- **split**: one MT field → multiple MX elements (e.g., field 32A → date + currency + amount)
- **merge**: multiple MX elements → one MT field (reverse of split)
- **lookup**: value mapping table (e.g., MT 71A SHA → MX ChrgBr SHAR)
- **derived**: computed from other fields or defaults
- **conditional**: depends on presence/value of other fields

### Validation Rule Structure
Rules from JSON have: id, description, severity (error/warning/info), condition, assertion, field_paths, suggestion, category.
- condition: "always" or "if X exists" — determines when rule applies
- assertion: expression like `IntrBkSttlmAmt > 0` or `length(BICFI) in [8, 11]`
- Implement a simple expression evaluator for these assertion strings

## Recommended Rust Dependencies

```toml
# In workspace Cargo.toml [workspace.dependencies]
thiserror = "2"          # Error types
serde = { version = "1", features = ["derive"] }
serde_json = "1"         # JSON spec loading
quick-xml = { version = "0.37", features = ["serialize"] }  # XML parsing/serialization
clap = { version = "4", features = ["derive"] }  # CLI argument parsing
chrono = { version = "0.4", features = ["serde"] }  # Date/time handling
regex = "1"              # Pattern matching for MT fields, IBAN, BIC
uuid = { version = "1", features = ["v4"] }  # UETR generation
rust_decimal = { version = "1", features = ["serde-str"] }  # Precise financial amounts
log = "0.4"              # Logging facade
env_logger = "0.11"      # CLI logging
```

## Quality Requirements

- ALL code must compile: `cargo build --workspace`
- ALL tests must pass: `cargo test --workspace`
- No clippy warnings: `cargo clippy --workspace -- -D warnings`
- Use `#[cfg(test)]` modules in each file for unit tests
- Integration tests in `crates/*/tests/` directories
- Use `thiserror` for all error types (not anyhow in library crates)
- Use `rust_decimal::Decimal` for financial amounts (never f64)
- All public API types derive `Debug, Clone, PartialEq, Serialize, Deserialize` where appropriate
- Follow Rust naming conventions: snake_case for functions/variables, CamelCase for types

## Code Patterns

### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid block {block}: {reason}")]
    InvalidBlock { block: u8, reason: String },
    #[error("missing mandatory field: {tag}")]
    MissingField { tag: String },
    // ...
}
```

### Core Type Pattern
```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    pub value: Decimal,
    pub currency: String,  // ISO 4217 3-letter code
}
```

### Spec Loading Pattern
```rust
use std::path::Path;

pub fn load_specs(specs_dir: &Path) -> Result<Specs, SpecError> {
    let currencies_json = std::fs::read_to_string(specs_dir.join("reference/currencies.json"))?;
    let currencies: CurrencyList = serde_json::from_str(&currencies_json)?;
    // ...
}
```

### MT Parsing Pattern
MT messages are parsed in two phases:
1. **Block parsing**: Split raw text into 5 blocks by matching `{N:...}` patterns
2. **Field parsing**: Within block 4, parse `:TAG:VALUE` entries respecting multi-line values

### XML Parsing Pattern
Use `quick-xml` with serde for structured parsing. Define Rust structs matching the ISO 20022 schema and derive `Deserialize`/`Serialize`.

## Progress Report Format

APPEND to scripts/ralph/progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
  - Useful context
---
```

## Consolidate Patterns

If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist).

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally.

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep all code compiling and tests passing
- Read the Codebase Patterns section in progress.txt before starting
- Use `rust_decimal::Decimal` for ALL financial amounts — never use f64
- Use `quick-xml` for XML parsing, not `xml-rs` or `roxmltree`
- Use `thiserror` for errors in library crates, `anyhow` only in CLI
- Embed small reference data with `include_str!`, load large specs from disk
- Test with real MT/MX messages from `../paymsg-specs/testdata/`
- When loading specs, use `PAYMSG_SPECS_DIR` env var or default to `../paymsg-specs`
