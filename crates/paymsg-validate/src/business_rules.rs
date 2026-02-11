/// Business rule evaluation engine
///
/// This module implements a rule evaluation engine that loads business validation rules
/// from JSON files and applies them to parsed messages using a simple expression evaluator.
use crate::types::{ValidationIssue, ValidationResult, Validator};
use paymsg_core::specs::{SpecLoader, SpecRegistries};
use paymsg_core::{Bic, Iban, PaymsgError};
use paymsg_iso20022::pacs008;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

/// Business rule definition loaded from JSON
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub description: String,
    pub severity: String, // "error", "warning", "info"
    pub condition: String, // "always" or "if X exists"
    pub assertion: String, // Expression to evaluate
    pub field_paths: Vec<String>,
    pub suggestion: String,
    pub category: String,
    #[serde(default)]
    pub references: Vec<String>,
}

/// Collection of business rules for a message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleSet {
    pub message_type: String,
    pub description: String,
    pub rules: Vec<BusinessRule>,
}

/// Expression value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Decimal(Decimal),
    Bool(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<chrono::Utc>),
    Null,
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Decimal(d) => !d.is_zero(),
            Value::Null => false,
            _ => true,
        }
    }

    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Value::Decimal(d) => Some(*d),
            Value::String(s) => Decimal::from_str(s).ok(),
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Decimal(d) => d.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Date(d) => d.to_string(),
            Value::DateTime(dt) => dt.to_rfc3339(),
            Value::Null => String::new(),
        }
    }
}

/// Expression evaluation context
pub struct EvaluationContext<'a> {
    pub message: &'a pacs008::Document,
    pub specs: &'a SpecRegistries,
    pub variables: HashMap<String, Value>,
}

impl<'a> EvaluationContext<'a> {
    pub fn new(message: &'a pacs008::Document, specs: &'a SpecRegistries) -> Self {
        Self {
            message,
            specs,
            variables: HashMap::new(),
        }
    }

    /// Resolve a field path to a value
    pub fn resolve_field_path(&self, path: &str) -> Value {
        // Split path by / separator
        let parts: Vec<&str> = path.split('/').collect();
        if parts.is_empty() {
            return Value::Null;
        }

        // Navigate through the message structure
        let msg = &self.message.fi_to_fi_customer_credit_transfer;

        // Handle GrpHdr paths
        if parts[0] == "GrpHdr" {
            return self.resolve_group_header_path(&msg.group_header, &parts[1..]);
        }

        // Handle CdtTrfTxInf paths - for now, use first transaction
        if parts[0] == "CdtTrfTxInf" && !msg.credit_transfer_transaction_information.is_empty() {
            let tx_info = &msg.credit_transfer_transaction_information[0];
            return self.resolve_transaction_path(tx_info, &parts[1..]);
        }

        Value::Null
    }

    fn resolve_group_header_path(&self, hdr: &pacs008::GroupHeader, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Null;
        }

        match parts[0] {
            "MsgId" => Value::String(hdr.message_id.clone()),
            "CreDtTm" => {
                // Parse ISO 8601 datetime string
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&hdr.creation_date_time) {
                    Value::DateTime(dt.with_timezone(&chrono::Utc))
                } else {
                    Value::String(hdr.creation_date_time.clone())
                }
            }
            "NbOfTxs" => Value::Decimal(Decimal::from_str(&hdr.number_of_transactions).unwrap_or_default()),
            "TtlIntrBkSttlmAmt" => {
                if let Some(amt) = &hdr.total_interbank_settlement_amount {
                    if parts.len() > 1 && parts[1] == "@Ccy" {
                        Value::String(amt.currency.clone())
                    } else {
                        Value::Decimal(amt.value)
                    }
                } else {
                    Value::Null
                }
            }
            "InstgAgt" => self.resolve_agent_path(&hdr.instructing_agent, &parts[1..]),
            "InstdAgt" => self.resolve_agent_path(&hdr.instructed_agent, &parts[1..]),
            _ => Value::Null,
        }
    }

    fn resolve_transaction_path(&self, tx: &pacs008::CreditTransferTransactionInformation, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Null;
        }

        match parts[0] {
            "PmtId" => self.resolve_payment_id_path(&tx.payment_id, &parts[1..]),
            "IntrBkSttlmAmt" => {
                if parts.len() > 1 && parts[1] == "@Ccy" {
                    Value::String(tx.interbank_settlement_amount.currency.clone())
                } else {
                    Value::Decimal(tx.interbank_settlement_amount.value)
                }
            }
            "InstdAmt" => {
                if let Some(amt) = &tx.instructed_amount {
                    if parts.len() > 1 && parts[1] == "@Ccy" {
                        Value::String(amt.currency.clone())
                    } else {
                        Value::Decimal(amt.value)
                    }
                } else {
                    Value::Null
                }
            }
            "XchgRate" => {
                if let Some(rate_str) = &tx.exchange_rate {
                    if let Ok(rate) = Decimal::from_str(rate_str) {
                        Value::Decimal(rate)
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            }
            "ChrgBr" => Value::String(tx.charge_bearer.clone()),
            "IntrBkSttlmDt" => {
                if let Some(date_str) = &tx.interbank_settlement_date {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        Value::Date(date)
                    } else {
                        Value::String(date_str.clone())
                    }
                } else {
                    Value::Null
                }
            }
            "DbtrAgt" => self.resolve_agent_path(&tx.debtor_agent, &parts[1..]),
            "CdtrAgt" => self.resolve_agent_path(&tx.creditor_agent, &parts[1..]),
            "Dbtr" => self.resolve_party_path(&tx.debtor, &parts[1..]),
            "Cdtr" => self.resolve_party_path(&tx.creditor, &parts[1..]),
            "DbtrAcct" => self.resolve_account_path(&tx.debtor_account, &parts[1..]),
            "CdtrAcct" => self.resolve_account_path(&tx.creditor_account, &parts[1..]),
            "PmtTpInf" => self.resolve_payment_type_path(tx.payment_type_information.as_ref(), &parts[1..]),
            "IntrmyAgt1" => {
                if let Some(agent) = &tx.intermediary_agent_1 {
                    self.resolve_agent_path(agent, &parts[1..])
                } else {
                    Value::Null
                }
            }
            "RgltryRptg" => {
                if tx.regulatory_reporting.is_some() {
                    Value::Bool(true)
                } else {
                    Value::Null
                }
            }
            "InstrForCdtrAgt" => {
                if tx.instruction_for_creditor_agent.is_some() {
                    Value::Bool(true)
                } else {
                    Value::Null
                }
            }
            "Purp" => {
                if tx.purpose.is_some() {
                    Value::Bool(true)
                } else {
                    Value::Null
                }
            }
            "ChrgsInf" => {
                if parts.len() > 1 && parts[1] == "Amt" {
                    if let Some(charges) = tx.charges_information.as_ref().and_then(|v| v.first()) {
                        Value::Decimal(charges.amount.value)
                    } else {
                        Value::Null
                    }
                } else if !tx.charges_information.as_ref().map_or(true, |v| v.is_empty()) {
                    Value::Bool(true)
                } else {
                    Value::Null
                }
            }
            _ => Value::Null,
        }
    }

    fn resolve_payment_id_path(&self, pmt_id: &pacs008::PaymentIdentification, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Null;
        }

        match parts[0] {
            "InstrId" => pmt_id.instruction_id.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
            "EndToEndId" => Value::String(pmt_id.end_to_end_id.clone()),
            "TxId" => pmt_id.transaction_id.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
            "UETR" => pmt_id.uetr.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    fn resolve_agent_path(&self, agent: &pacs008::BranchAndFinancialInstitutionIdentification, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Bool(true);
        }

        if parts[0] == "FinInstnId" {
            if parts.len() > 1 && parts[1] == "BICFI" {
                agent.financial_institution_id.bic.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null)
            } else {
                Value::Bool(true)
            }
        } else {
            Value::Null
        }
    }

    fn resolve_party_path(&self, party: &pacs008::PartyIdentification, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Bool(true);
        }

        match parts[0] {
            "Nm" => party.name.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    fn resolve_account_path(&self, account: &pacs008::CashAccount, parts: &[&str]) -> Value {
        if parts.is_empty() {
            return Value::Bool(true);
        }

        if parts[0] == "Id" && parts.len() > 1 {
            match parts[1] {
                "IBAN" => account.id.iban.as_ref().map(|s| Value::String(s.clone())).unwrap_or(Value::Null),
                _ => Value::Null,
            }
        } else {
            Value::Null
        }
    }

    fn resolve_payment_type_path(&self, pmt_type: Option<&pacs008::PaymentTypeInformation>, parts: &[&str]) -> Value {
        let pmt_type = match pmt_type {
            Some(p) => p,
            None => return Value::Null,
        };

        if parts.is_empty() {
            return Value::Bool(true);
        }

        if parts[0] == "SvcLvl" && parts.len() > 1 && parts[1] == "Cd" {
            pmt_type.service_level.as_ref().and_then(|sl| sl.code.as_ref()).map(|s| Value::String(s.clone())).unwrap_or(Value::Null)
        } else if parts[0] == "CtgyPurp" {
            if pmt_type.category_purpose.is_some() {
                Value::Bool(true)
            } else {
                Value::Null
            }
        } else {
            Value::Null
        }
    }
}

/// Simple expression evaluator for business rules
pub fn evaluate_expression(expr: &str, ctx: &EvaluationContext) -> Result<Value, PaymsgError> {
    let expr = expr.trim();

    // Handle simple boolean operations
    if expr.contains(" AND ") {
        let parts: Vec<&str> = expr.splitn(2, " AND ").collect();
        let left = evaluate_expression(parts[0], ctx)?;
        let right = evaluate_expression(parts[1], ctx)?;
        return Ok(Value::Bool(left.as_bool() && right.as_bool()));
    }

    if expr.contains(" OR ") {
        let parts: Vec<&str> = expr.splitn(2, " OR ").collect();
        let left = evaluate_expression(parts[0], ctx)?;
        let right = evaluate_expression(parts[1], ctx)?;
        return Ok(Value::Bool(left.as_bool() || right.as_bool()));
    }

    // Handle NOT
    if let Some(inner) = expr.strip_prefix("NOT ") {
        let val = evaluate_expression(inner, ctx)?;
        return Ok(Value::Bool(!val.as_bool()));
    }

    // Handle comparison operators
    for op in &[">=", "<=", "==", "!=", ">", "<"] {
        if expr.contains(op) {
            let parts: Vec<&str> = expr.splitn(2, op).collect();
            if parts.len() == 2 {
                let left = evaluate_operand(parts[0].trim(), ctx)?;
                let right = evaluate_operand(parts[1].trim(), ctx)?;

                return Ok(Value::Bool(match *op {
                    ">" => compare_values(&left, &right) == Some(std::cmp::Ordering::Greater),
                    "<" => compare_values(&left, &right) == Some(std::cmp::Ordering::Less),
                    ">=" => matches!(compare_values(&left, &right), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
                    "<=" => matches!(compare_values(&left, &right), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
                    "==" => left == right,
                    "!=" => left != right,
                    _ => false,
                }));
            }
        }
    }

    // Handle "in" operator
    if expr.contains(" in ") {
        let parts: Vec<&str> = expr.splitn(2, " in ").collect();
        if parts.len() == 2 {
            let value = evaluate_operand(parts[0].trim(), ctx)?;
            let list_expr = parts[1].trim();

            // Parse list: ['A', 'B', 'C'] or function()
            if list_expr.starts_with('[') && list_expr.ends_with(']') {
                let items: Vec<String> = list_expr[1..list_expr.len()-1]
                    .split(',')
                    .map(|s| s.trim().trim_matches('\'').to_string())
                    .collect();
                return Ok(Value::Bool(items.contains(&value.as_string())));
            } else if list_expr.contains("()") {
                // Handle function calls
                return evaluate_function(list_expr, &value, ctx);
            }
        }
    }

    // Handle "is not empty"
    if expr.ends_with(" is not empty") {
        let field_path = expr.trim_end_matches(" is not empty").trim();
        let value = ctx.resolve_field_path(field_path);
        return Ok(Value::Bool(!value.as_string().is_empty()));
    }

    // Handle "exists"
    if expr.ends_with(" exists") {
        let field_path = expr.trim_end_matches(" exists").trim();
        let value = ctx.resolve_field_path(field_path);
        return Ok(Value::Bool(value != Value::Null));
    }

    // Handle function calls
    if expr.contains('(') && expr.ends_with(')') {
        return evaluate_function(expr, &Value::Null, ctx);
    }

    // Otherwise treat as field path or literal
    evaluate_operand(expr, ctx)
}

fn evaluate_operand(expr: &str, ctx: &EvaluationContext) -> Result<Value, PaymsgError> {
    let expr = expr.trim();

    // String literal
    if (expr.starts_with('\'') && expr.ends_with('\'')) || (expr.starts_with('"') && expr.ends_with('"')) {
        return Ok(Value::String(expr[1..expr.len()-1].to_string()));
    }

    // Numeric literal
    if let Ok(d) = Decimal::from_str(expr) {
        return Ok(Value::Decimal(d));
    }

    // Boolean literal
    if expr == "true" {
        return Ok(Value::Bool(true));
    }
    if expr == "false" {
        return Ok(Value::Bool(false));
    }

    // Field path
    Ok(ctx.resolve_field_path(expr))
}

fn evaluate_function(expr: &str, check_value: &Value, ctx: &EvaluationContext) -> Result<Value, PaymsgError> {
    let paren_pos = expr.find('(').unwrap();
    let func_name = &expr[..paren_pos];
    let args_str = &expr[paren_pos+1..expr.len()-1];

    match func_name {
        "is_valid_bic" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            let bic_str = arg.as_string();
            Ok(Value::Bool(Bic::from_str(&bic_str).is_ok()))
        }
        "is_valid_iban" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            let iban_str = arg.as_string();
            Ok(Value::Bool(Iban::from_str(&iban_str).is_ok()))
        }
        "is_valid_uuid" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            let uuid_str = arg.as_string();
            Ok(Value::Bool(uuid::Uuid::parse_str(&uuid_str).is_ok()))
        }
        "length" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            Ok(Value::Decimal(Decimal::from(arg.as_string().len())))
        }
        "decimal_places" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            if let Some(d) = arg.as_decimal() {
                Ok(Value::Decimal(Decimal::from(d.scale())))
            } else {
                Ok(Value::Decimal(Decimal::ZERO))
            }
        }
        "currency_decimal_places" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            let currency_code = arg.as_string();
            if let Some(currency) = ctx.specs.currencies.lookup_by_code(&currency_code) {
                Ok(Value::Decimal(Decimal::from(currency.decimal_places)))
            } else {
                Ok(Value::Decimal(Decimal::from(2))) // Default to 2
            }
        }
        "iso4217_currency_codes" => {
            // Used with "in" operator - check if the value is a valid currency
            let currency_code = check_value.as_string();
            Ok(Value::Bool(ctx.specs.currencies.lookup_by_code(&currency_code).is_some()))
        }
        "current_date" => {
            Ok(Value::Date(chrono::Utc::now().date_naive()))
        }
        "current_datetime" => {
            Ok(Value::DateTime(chrono::Utc::now()))
        }
        "count" => {
            // Count CdtTrfTxInf elements
            if args_str.trim() == "CdtTrfTxInf" {
                let count = ctx.message.fi_to_fi_customer_credit_transfer.credit_transfer_transaction_information.len();
                Ok(Value::Decimal(Decimal::from(count)))
            } else {
                Ok(Value::Decimal(Decimal::ZERO))
            }
        }
        "sum" => {
            // Sum of amounts
            if args_str.trim().contains("IntrBkSttlmAmt") {
                let sum: Decimal = ctx.message.fi_to_fi_customer_credit_transfer.credit_transfer_transaction_information
                    .iter()
                    .map(|tx| tx.interbank_settlement_amount.value)
                    .sum();
                Ok(Value::Decimal(sum))
            } else {
                Ok(Value::Decimal(Decimal::ZERO))
            }
        }
        "bic_country" => {
            let arg = evaluate_operand(args_str.trim(), ctx)?;
            let bic_str = arg.as_string();
            if let Ok(bic) = Bic::from_str(&bic_str) {
                Ok(Value::String(bic.country.clone()))
            } else {
                Ok(Value::Null)
            }
        }
        "sepa_countries" | "eu_countries" => {
            // Return true for "in" checks - simplified for now
            Ok(Value::Bool(true))
        }
        _ => Ok(Value::Null),
    }
}

fn compare_values(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (Value::Decimal(l), Value::Decimal(r)) => Some(l.cmp(r)),
        (Value::String(l), Value::String(r)) => Some(l.cmp(r)),
        (Value::Date(l), Value::Date(r)) => Some(l.cmp(r)),
        (Value::DateTime(l), Value::DateTime(r)) => Some(l.cmp(r)),
        _ => None,
    }
}

/// Load business rules from JSON file
pub fn load_business_rules(path: &Path) -> Result<BusinessRuleSet, PaymsgError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| PaymsgError::SpecLoadError {
            file: path.display().to_string(),
            reason: e.to_string(),
        })?;

    serde_json::from_str(&content)
        .map_err(|e| PaymsgError::SpecLoadError {
            file: path.display().to_string(),
            reason: e.to_string(),
        })
}

/// Business rule validator for pacs.008 messages
pub struct BusinessRuleValidator {
    rules: BusinessRuleSet,
    specs: SpecRegistries,
}

impl BusinessRuleValidator {
    pub fn new(rules_path: &Path, specs_path: &Path) -> Result<Self, PaymsgError> {
        let rules = load_business_rules(rules_path)?;
        let loader = SpecLoader::new(Some(specs_path.to_path_buf()));
        let specs = loader.load_all()?;
        Ok(Self { rules, specs })
    }

    pub fn from_rules_and_specs(rules: BusinessRuleSet, specs: SpecRegistries) -> Self {
        Self { rules, specs }
    }

    fn evaluate_condition(&self, condition: &str, ctx: &EvaluationContext) -> Result<bool, PaymsgError> {
        if condition == "always" {
            return Ok(true);
        }

        // Parse "if X exists" condition
        if let Some(cond_expr) = condition.strip_prefix("if ") {
            let result = evaluate_expression(cond_expr, ctx)?;
            Ok(result.as_bool())
        } else {
            Ok(true)
        }
    }
}

impl Validator<pacs008::Document> for BusinessRuleValidator {
    fn validate(&self, message: &pacs008::Document) -> ValidationResult {
        let mut result = ValidationResult::new();
        let ctx = EvaluationContext::new(message, &self.specs);

        for rule in &self.rules.rules {
            // Check if rule condition applies
            let applies = match self.evaluate_condition(&rule.condition, &ctx) {
                Ok(b) => b,
                Err(_) => {
                    // If condition evaluation fails, skip this rule
                    continue;
                }
            };

            if !applies {
                continue;
            }

            // Evaluate assertion
            let assertion_result = match evaluate_expression(&rule.assertion, &ctx) {
                Ok(val) => val.as_bool(),
                Err(_) => {
                    // If assertion fails to evaluate, treat as failed
                    false
                }
            };

            if !assertion_result {
                result.add_issue(ValidationIssue {
                    id: rule.id.clone(),
                    severity: match rule.severity.as_str() {
                        "error" => crate::types::Severity::Error,
                        "warning" => crate::types::Severity::Warning,
                        _ => crate::types::Severity::Info,
                    },
                    field_path: rule.field_paths.first().cloned(),
                    message: rule.description.clone(),
                    suggestion: Some(rule.suggestion.clone()),
                });
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_evaluate_simple_comparison() {
        let doc = create_test_document();
        let specs = create_test_specs();
        let ctx = EvaluationContext::new(&doc, &specs);

        // Test numeric comparison - use full path
        let result = evaluate_expression("CdtTrfTxInf/IntrBkSttlmAmt > 0", &ctx).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_field_path() {
        let doc = create_test_document();
        let specs = create_test_specs();
        let ctx = EvaluationContext::new(&doc, &specs);

        // Test field path resolution
        let result = ctx.resolve_field_path("CdtTrfTxInf/IntrBkSttlmAmt");
        if let Value::Decimal(d) = result {
            assert!(d > Decimal::ZERO);
        } else {
            panic!("Expected decimal value");
        }
    }

    #[test]
    fn test_evaluate_exists() {
        let doc = create_test_document();
        let specs = create_test_specs();
        let ctx = EvaluationContext::new(&doc, &specs);

        // Test exists check - use full path
        let result = evaluate_expression("CdtTrfTxInf/ChrgBr exists", &ctx).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_is_valid_bic() {
        let doc = create_test_document();
        let specs = create_test_specs();
        let ctx = EvaluationContext::new(&doc, &specs);

        // Test BIC validation function
        let result = evaluate_expression("is_valid_bic('DEUTDEFF')", &ctx).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = evaluate_expression("is_valid_bic('INVALID')", &ctx).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_evaluate_and_or() {
        let doc = create_test_document();
        let specs = create_test_specs();
        let ctx = EvaluationContext::new(&doc, &specs);

        // Test AND - use full paths
        let result = evaluate_expression("CdtTrfTxInf/IntrBkSttlmAmt > 0 AND CdtTrfTxInf/ChrgBr exists", &ctx).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Test OR - use full paths
        let result = evaluate_expression("CdtTrfTxInf/IntrBkSttlmAmt < 0 OR CdtTrfTxInf/ChrgBr exists", &ctx).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    fn create_test_document() -> pacs008::Document {
        use paymsg_iso20022::pacs008::*;

        Document {
            fi_to_fi_customer_credit_transfer: FIToFICstmrCdtTrf {
                group_header: GroupHeader {
                    message_id: "MSG001".to_string(),
                    creation_date_time: Utc::now().to_rfc3339(),
                    number_of_transactions: "1".to_string(),
                    total_interbank_settlement_amount: None,
                    interbank_settlement_date: None,
                    settlement_information: None,
                    instructing_agent: BranchAndFinancialInstitutionIdentification {
                        financial_institution_id: FinancialInstitutionIdentification {
                            bic: Some("DEUTDEFF".to_string()),
                            name: None,
                            postal_address: None,
                        },
                    },
                    instructed_agent: BranchAndFinancialInstitutionIdentification {
                        financial_institution_id: FinancialInstitutionIdentification {
                            bic: Some("BNPAFRPP".to_string()),
                            name: None,
                            postal_address: None,
                        },
                    },
                },
                credit_transfer_transaction_information: vec![
                    CreditTransferTransactionInformation {
                        payment_id: PaymentIdentification {
                            instruction_id: Some("INSTR001".to_string()),
                            end_to_end_id: "E2E001".to_string(),
                            transaction_id: None,
                            uetr: None,
                        },
                        payment_type_information: None,
                        interbank_settlement_amount: ActiveCurrencyAndAmount {
                            currency: "EUR".to_string(),
                            value: Decimal::from_str("1000.00").unwrap(),
                        },
                        interbank_settlement_date: None,
                        instructed_amount: None,
                        exchange_rate: None,
                        charge_bearer: "SHAR".to_string(),
                        charges_information: None,
                        debtor: PartyIdentification { name: Some("Test Debtor".to_string()), postal_address: None, id: None },
                        debtor_account: CashAccount { id: AccountIdentification { iban: Some("DE89370400440532013000".to_string()), other: None }, account_type: None, currency: None },
                        debtor_agent: BranchAndFinancialInstitutionIdentification {
                            financial_institution_id: FinancialInstitutionIdentification { bic: Some("DEUTDEFF".to_string()), name: None, postal_address: None },
                        },
                        creditor_agent: BranchAndFinancialInstitutionIdentification {
                            financial_institution_id: FinancialInstitutionIdentification { bic: Some("BNPAFRPP".to_string()), name: None, postal_address: None },
                        },
                        creditor: PartyIdentification { name: Some("Test Creditor".to_string()), postal_address: None, id: None },
                        creditor_account: CashAccount { id: AccountIdentification { iban: Some("FR1420041010050500013M02606".to_string()), other: None }, account_type: None, currency: None },
                        intermediary_agent_1: None,
                        intermediary_agent_2: None,
                        intermediary_agent_3: None,
                        instruction_for_creditor_agent: None,
                        instruction_for_next_agent: None,
                        purpose: None,
                        regulatory_reporting: None,
                        remittance_information: None,
                    },
                ],
            },
        }
    }

    fn create_test_specs() -> SpecRegistries {
        // Load from actual specs directory for tests
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let specs_path = std::path::PathBuf::from(&manifest_dir)
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join("paymsg-specs");

        let loader = SpecLoader::new(Some(specs_path));
        loader.load_all().unwrap()
    }
}
