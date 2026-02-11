//! camt.053.001.10 Bank-to-Customer Statement
//!
//! Equivalent to MT940 (Customer Statement Message)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Document root for camt.053
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "BkToCstmrStmt")]
    pub bank_to_customer_statement: BankToCustomerStatement,
}

/// BankToCustomerStatement main container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankToCustomerStatement {
    #[serde(rename = "GrpHdr")]
    pub group_header: GroupHeader,

    #[serde(rename = "Stmt")]
    pub statement: Vec<AccountStatement>,
}

/// GroupHeader for statement messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupHeader {
    #[serde(rename = "MsgId")]
    pub message_id: String,

    #[serde(rename = "CreDtTm")]
    pub creation_date_time: String,

    #[serde(rename = "MsgRcpt", skip_serializing_if = "Option::is_none")]
    pub message_recipient: Option<PartyIdentification>,

    #[serde(rename = "MsgPgntn", skip_serializing_if = "Option::is_none")]
    pub message_pagination: Option<Pagination>,

    #[serde(rename = "AddtlInf", skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
}

/// Pagination information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(rename = "PgNb")]
    pub page_number: String,

    #[serde(rename = "LastPgInd")]
    pub last_page_indicator: bool,
}

/// Party Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyIdentification {
    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<PostalAddress>,

    #[serde(rename = "Id", skip_serializing_if = "Option::is_none")]
    pub identification: Option<Party>,
}

/// Postal Address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalAddress {
    #[serde(rename = "AdrTp", skip_serializing_if = "Option::is_none")]
    pub address_type: Option<String>,

    #[serde(rename = "AdrLine", skip_serializing_if = "Option::is_none")]
    pub address_line: Option<Vec<String>>,

    #[serde(rename = "StrtNm", skip_serializing_if = "Option::is_none")]
    pub street_name: Option<String>,

    #[serde(rename = "BldgNb", skip_serializing_if = "Option::is_none")]
    pub building_number: Option<String>,

    #[serde(rename = "PstCd", skip_serializing_if = "Option::is_none")]
    pub post_code: Option<String>,

    #[serde(rename = "TwnNm", skip_serializing_if = "Option::is_none")]
    pub town_name: Option<String>,

    #[serde(rename = "Ctry", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// Party
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "OrgId", skip_serializing_if = "Option::is_none")]
    pub organization_identification: Option<OrganizationIdentification>,

    #[serde(rename = "PrvtId", skip_serializing_if = "Option::is_none")]
    pub private_identification: Option<PersonIdentification>,
}

/// Organization Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationIdentification {
    #[serde(rename = "AnyBIC", skip_serializing_if = "Option::is_none")]
    pub any_bic: Option<String>,

    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<GenericIdentification>>,
}

/// Person Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonIdentification {
    #[serde(rename = "DtAndPlcOfBirth", skip_serializing_if = "Option::is_none")]
    pub date_and_place_of_birth: Option<DateAndPlaceOfBirth>,

    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<GenericIdentification>>,
}

/// Date and Place of Birth
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateAndPlaceOfBirth {
    #[serde(rename = "BirthDt")]
    pub birth_date: String,

    #[serde(rename = "CityOfBirth")]
    pub city_of_birth: String,

    #[serde(rename = "CtryOfBirth")]
    pub country_of_birth: String,
}

/// Generic Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericIdentification {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "SchmeNm", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<SchemeName>,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Scheme Name
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemeName {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Account Statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountStatement {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "ElctrncSeqNb", skip_serializing_if = "Option::is_none")]
    pub electronic_sequence_number: Option<String>,

    #[serde(rename = "LglSeqNb", skip_serializing_if = "Option::is_none")]
    pub legal_sequence_number: Option<u64>,

    #[serde(rename = "CreDtTm")]
    pub creation_date_time: String,

    #[serde(rename = "FrToDt", skip_serializing_if = "Option::is_none")]
    pub from_to_date: Option<DateTimePeriod>,

    #[serde(rename = "CpyDplctInd", skip_serializing_if = "Option::is_none")]
    pub copy_duplicate_indicator: Option<String>,

    #[serde(rename = "RptgSrc", skip_serializing_if = "Option::is_none")]
    pub reporting_source: Option<ReportingSource>,

    #[serde(rename = "Acct")]
    pub account: CashAccount,

    #[serde(rename = "RltdAcct", skip_serializing_if = "Option::is_none")]
    pub related_account: Option<CashAccount>,

    #[serde(rename = "Intrst", skip_serializing_if = "Option::is_none")]
    pub interest: Option<Vec<InterestRecord>>,

    #[serde(rename = "Bal")]
    pub balance: Vec<Balance>,

    #[serde(rename = "TxsSummary", skip_serializing_if = "Option::is_none")]
    pub transactions_summary: Option<TransactionsSummary>,

    #[serde(rename = "Ntry", skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<Entry>>,

    #[serde(rename = "AddtlStmtInf", skip_serializing_if = "Option::is_none")]
    pub additional_statement_info: Option<String>,
}

/// Date Time Period
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimePeriod {
    #[serde(rename = "FrDtTm")]
    pub from_date_time: String,

    #[serde(rename = "ToDtTm")]
    pub to_date_time: String,
}

/// Reporting Source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportingSource {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Cash Account
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashAccount {
    #[serde(rename = "Id")]
    pub id: AccountIdentification,

    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub account_type: Option<CashAccountType>,

    #[serde(rename = "Ccy", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "Ownr", skip_serializing_if = "Option::is_none")]
    pub owner: Option<PartyIdentification>,

    #[serde(rename = "Svcr", skip_serializing_if = "Option::is_none")]
    pub servicer: Option<BranchAndFinancialInstitutionIdentification>,
}

/// Account Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountIdentification {
    #[serde(rename = "IBAN", skip_serializing_if = "Option::is_none")]
    pub iban: Option<String>,

    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<GenericAccountIdentification>,
}

/// Generic Account Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericAccountIdentification {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "SchmeNm", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<SchemeName>,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Cash Account Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashAccountType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Branch And Financial Institution Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchAndFinancialInstitutionIdentification {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_identification: FinancialInstitutionIdentification,

    #[serde(rename = "BrnchId", skip_serializing_if = "Option::is_none")]
    pub branch_identification: Option<BranchData>,
}

/// Financial Institution Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialInstitutionIdentification {
    #[serde(rename = "BICFI", skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,

    #[serde(rename = "ClrSysMmbId", skip_serializing_if = "Option::is_none")]
    pub clearing_system_member_identification: Option<ClearingSystemMemberIdentification>,

    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<PostalAddress>,

    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<GenericIdentification>,
}

/// Clearing System Member Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearingSystemMemberIdentification {
    #[serde(rename = "ClrSysId", skip_serializing_if = "Option::is_none")]
    pub clearing_system_identification: Option<ClearingSystemIdentification>,

    #[serde(rename = "MmbId")]
    pub member_id: String,
}

/// Clearing System Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearingSystemIdentification {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Branch Data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchData {
    #[serde(rename = "Id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<PostalAddress>,
}

/// Interest Record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterestRecord {
    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd")]
    pub credit_debit_indicator: String,

    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub interest_type: Option<InterestType>,

    #[serde(rename = "Rate", skip_serializing_if = "Option::is_none")]
    pub rate: Option<Vec<Rate>>,
}

/// Active Or Historic Currency And Amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveOrHistoricCurrencyAndAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$text")]
    pub value: Decimal,
}

/// Interest Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterestType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Rate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rate {
    #[serde(rename = "Tp")]
    pub rate_type: RateType,

    #[serde(rename = "VldtyRg", skip_serializing_if = "Option::is_none")]
    pub validity_range: Option<DateTimePeriod>,
}

/// Rate Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateType {
    #[serde(rename = "Pctg")]
    pub percentage: Decimal,
}

/// Balance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    #[serde(rename = "Tp")]
    pub balance_type: BalanceType,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd")]
    pub credit_debit_indicator: String,

    #[serde(rename = "Dt")]
    pub date: DateOrDateTime,

    #[serde(rename = "Avlbty", skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<CashBalanceAvailability>>,
}

/// Balance Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceType {
    #[serde(rename = "CdOrPrtry")]
    pub code_or_proprietary: CodeOrProprietary,

    #[serde(rename = "SubTp", skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<BalanceSubType>,
}

/// Code Or Proprietary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeOrProprietary {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Balance Sub Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceSubType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Date Or DateTime
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateOrDateTime {
    #[serde(rename = "Dt", skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    #[serde(rename = "DtTm", skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
}

/// Cash Balance Availability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashBalanceAvailability {
    #[serde(rename = "Dt")]
    pub date: DateOrDateTime,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd")]
    pub credit_debit_indicator: String,
}

/// Transactions Summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionsSummary {
    #[serde(rename = "TtlNtries", skip_serializing_if = "Option::is_none")]
    pub total_entries: Option<NumberAndSumOfTransactions>,

    #[serde(rename = "TtlCdtNtries", skip_serializing_if = "Option::is_none")]
    pub total_credit_entries: Option<NumberAndSumOfTransactions>,

    #[serde(rename = "TtlDbtNtries", skip_serializing_if = "Option::is_none")]
    pub total_debit_entries: Option<NumberAndSumOfTransactions>,
}

/// Number And Sum Of Transactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberAndSumOfTransactions {
    #[serde(rename = "NbOfNtries", skip_serializing_if = "Option::is_none")]
    pub number_of_entries: Option<u64>,

    #[serde(rename = "Sum", skip_serializing_if = "Option::is_none")]
    pub sum: Option<Decimal>,
}

/// Entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "NtryRef", skip_serializing_if = "Option::is_none")]
    pub entry_reference: Option<String>,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd")]
    pub credit_debit_indicator: String,

    #[serde(rename = "RvslInd", skip_serializing_if = "Option::is_none")]
    pub reversal_indicator: Option<bool>,

    #[serde(rename = "Sts")]
    pub status: EntryStatus,

    #[serde(rename = "BookgDt", skip_serializing_if = "Option::is_none")]
    pub booking_date: Option<DateOrDateTime>,

    #[serde(rename = "ValDt", skip_serializing_if = "Option::is_none")]
    pub value_date: Option<DateOrDateTime>,

    #[serde(rename = "AcctSvcrRef", skip_serializing_if = "Option::is_none")]
    pub account_servicer_reference: Option<String>,

    #[serde(rename = "Avlbty", skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<CashBalanceAvailability>>,

    #[serde(rename = "BkTxCd")]
    pub bank_transaction_code: BankTransactionCode,

    #[serde(rename = "ComssnWvrInd", skip_serializing_if = "Option::is_none")]
    pub commission_waiver_indicator: Option<bool>,

    #[serde(rename = "AddtlInfInd", skip_serializing_if = "Option::is_none")]
    pub additional_info_indicator: Option<MessageIdentification>,

    #[serde(rename = "AmtDtls", skip_serializing_if = "Option::is_none")]
    pub amount_details: Option<AmountDetails>,

    #[serde(rename = "Chrgs", skip_serializing_if = "Option::is_none")]
    pub charges: Option<Vec<Charges>>,

    #[serde(rename = "TechInptChanl", skip_serializing_if = "Option::is_none")]
    pub technical_input_channel: Option<TechnicalInputChannel>,

    #[serde(rename = "Intrst", skip_serializing_if = "Option::is_none")]
    pub interest: Option<Vec<InterestRecord>>,

    #[serde(rename = "NtryDtls", skip_serializing_if = "Option::is_none")]
    pub entry_details: Option<Vec<EntryDetails>>,

    #[serde(rename = "AddtlNtryInf", skip_serializing_if = "Option::is_none")]
    pub additional_entry_info: Option<String>,
}

/// Entry Status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryStatus {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Bank Transaction Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankTransactionCode {
    #[serde(rename = "Domn", skip_serializing_if = "Option::is_none")]
    pub domain: Option<BankTransactionCodeDomain>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<ProprietaryBankTransactionCode>,
}

/// Bank Transaction Code Domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankTransactionCodeDomain {
    #[serde(rename = "Cd")]
    pub code: String,

    #[serde(rename = "Fmly")]
    pub family: BankTransactionCodeFamily,
}

/// Bank Transaction Code Family
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankTransactionCodeFamily {
    #[serde(rename = "Cd")]
    pub code: String,

    #[serde(rename = "SubFmlyCd")]
    pub sub_family_code: String,
}

/// Proprietary Bank Transaction Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryBankTransactionCode {
    #[serde(rename = "Cd")]
    pub code: String,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Message Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageIdentification {
    #[serde(rename = "MsgId")]
    pub message_id: String,
}

/// Amount Details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmountDetails {
    #[serde(rename = "InstdAmt", skip_serializing_if = "Option::is_none")]
    pub instructed_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "TxAmt", skip_serializing_if = "Option::is_none")]
    pub transaction_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "CntrValAmt", skip_serializing_if = "Option::is_none")]
    pub counter_value_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "AnncdPstngAmt", skip_serializing_if = "Option::is_none")]
    pub announced_posting_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "PrtryAmt", skip_serializing_if = "Option::is_none")]
    pub proprietary_amount: Option<Vec<ProprietaryAmount>>,
}

/// Proprietary Amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryAmount {
    #[serde(rename = "Tp")]
    pub amount_type: String,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,
}

/// Charges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Charges {
    #[serde(rename = "TtlChrgsAndTaxAmt", skip_serializing_if = "Option::is_none")]
    pub total_charges_and_tax_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "Rcrd", skip_serializing_if = "Option::is_none")]
    pub record: Option<Vec<ChargesRecord>>,
}

/// Charges Record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargesRecord {
    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd", skip_serializing_if = "Option::is_none")]
    pub credit_debit_indicator: Option<String>,

    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub charge_type: Option<ChargeType>,
}

/// Charge Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargeType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<ProprietaryChargeType>,
}

/// Proprietary Charge Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryChargeType {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Technical Input Channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalInputChannel {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Entry Details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryDetails {
    #[serde(rename = "Btch", skip_serializing_if = "Option::is_none")]
    pub batch: Option<BatchInformation>,

    #[serde(rename = "TxDtls", skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<Vec<TransactionDetails>>,
}

/// Batch Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchInformation {
    #[serde(rename = "MsgId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    #[serde(rename = "PmtInfId", skip_serializing_if = "Option::is_none")]
    pub payment_information_id: Option<String>,

    #[serde(rename = "NbOfTxs", skip_serializing_if = "Option::is_none")]
    pub number_of_transactions: Option<String>,

    #[serde(rename = "TtlAmt", skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<ActiveOrHistoricCurrencyAndAmount>,
}

/// Transaction Details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionDetails {
    #[serde(rename = "Refs", skip_serializing_if = "Option::is_none")]
    pub references: Option<TransactionReferences>,

    #[serde(rename = "AmtDtls", skip_serializing_if = "Option::is_none")]
    pub amount_details: Option<AmountDetails>,

    #[serde(rename = "Avlbty", skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<CashBalanceAvailability>>,

    #[serde(rename = "BkTxCd", skip_serializing_if = "Option::is_none")]
    pub bank_transaction_code: Option<BankTransactionCode>,

    #[serde(rename = "Chrgs", skip_serializing_if = "Option::is_none")]
    pub charges: Option<Vec<Charges>>,

    #[serde(rename = "Intrst", skip_serializing_if = "Option::is_none")]
    pub interest: Option<Vec<InterestRecord>>,

    #[serde(rename = "RltdPties", skip_serializing_if = "Option::is_none")]
    pub related_parties: Option<RelatedParties>,

    #[serde(rename = "RltdAgts", skip_serializing_if = "Option::is_none")]
    pub related_agents: Option<RelatedAgents>,

    #[serde(rename = "Purp", skip_serializing_if = "Option::is_none")]
    pub purpose: Option<Purpose>,

    #[serde(rename = "RltdRmtInf", skip_serializing_if = "Option::is_none")]
    pub related_remittance_information: Option<Vec<RemittanceLocation>>,

    #[serde(rename = "RmtInf", skip_serializing_if = "Option::is_none")]
    pub remittance_information: Option<RemittanceInformation>,

    #[serde(rename = "RltdDts", skip_serializing_if = "Option::is_none")]
    pub related_dates: Option<RelatedDates>,

    #[serde(rename = "RltdPric", skip_serializing_if = "Option::is_none")]
    pub related_price: Option<RelatedPrice>,

    #[serde(rename = "RltdQties", skip_serializing_if = "Option::is_none")]
    pub related_quantities: Option<Vec<RelatedQuantities>>,

    #[serde(rename = "FinInstrmId", skip_serializing_if = "Option::is_none")]
    pub financial_instrument_id: Option<SecurityIdentification>,

    #[serde(rename = "Tax", skip_serializing_if = "Option::is_none")]
    pub tax: Option<TaxInformation>,

    #[serde(rename = "RtrInf", skip_serializing_if = "Option::is_none")]
    pub return_information: Option<ReturnInformation>,

    #[serde(rename = "CorpActn", skip_serializing_if = "Option::is_none")]
    pub corporate_action: Option<CorporateAction>,

    #[serde(rename = "SfkpgAcct", skip_serializing_if = "Option::is_none")]
    pub safekeeping_account: Option<SecuritiesAccount>,

    #[serde(rename = "CshDpst", skip_serializing_if = "Option::is_none")]
    pub cash_deposit: Option<Vec<CashDeposit>>,

    #[serde(rename = "CardTx", skip_serializing_if = "Option::is_none")]
    pub card_transaction: Option<CardTransaction>,

    #[serde(rename = "AddtlTxInf", skip_serializing_if = "Option::is_none")]
    pub additional_transaction_info: Option<String>,
}

/// Transaction References
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionReferences {
    #[serde(rename = "MsgId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    #[serde(rename = "AcctSvcrRef", skip_serializing_if = "Option::is_none")]
    pub account_servicer_reference: Option<String>,

    #[serde(rename = "PmtInfId", skip_serializing_if = "Option::is_none")]
    pub payment_information_id: Option<String>,

    #[serde(rename = "InstrId", skip_serializing_if = "Option::is_none")]
    pub instruction_id: Option<String>,

    #[serde(rename = "EndToEndId", skip_serializing_if = "Option::is_none")]
    pub end_to_end_id: Option<String>,

    #[serde(rename = "TxId", skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    #[serde(rename = "UETR", skip_serializing_if = "Option::is_none")]
    pub uetr: Option<String>,

    #[serde(rename = "MndtId", skip_serializing_if = "Option::is_none")]
    pub mandate_id: Option<String>,

    #[serde(rename = "ChqNb", skip_serializing_if = "Option::is_none")]
    pub cheque_number: Option<String>,

    #[serde(rename = "ClrSysRef", skip_serializing_if = "Option::is_none")]
    pub clearing_system_reference: Option<String>,

    #[serde(rename = "AcctOwnrTxId", skip_serializing_if = "Option::is_none")]
    pub account_owner_transaction_id: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<Vec<ProprietaryReference>>,
}

/// Proprietary Reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryReference {
    #[serde(rename = "Tp")]
    pub reference_type: String,

    #[serde(rename = "Ref")]
    pub reference: String,
}

/// Related Parties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedParties {
    #[serde(rename = "InitgPty", skip_serializing_if = "Option::is_none")]
    pub initiating_party: Option<PartyIdentification>,

    #[serde(rename = "Dbtr", skip_serializing_if = "Option::is_none")]
    pub debtor: Option<PartyIdentification>,

    #[serde(rename = "DbtrAcct", skip_serializing_if = "Option::is_none")]
    pub debtor_account: Option<CashAccount>,

    #[serde(rename = "UltmtDbtr", skip_serializing_if = "Option::is_none")]
    pub ultimate_debtor: Option<PartyIdentification>,

    #[serde(rename = "Cdtr", skip_serializing_if = "Option::is_none")]
    pub creditor: Option<PartyIdentification>,

    #[serde(rename = "CdtrAcct", skip_serializing_if = "Option::is_none")]
    pub creditor_account: Option<CashAccount>,

    #[serde(rename = "UltmtCdtr", skip_serializing_if = "Option::is_none")]
    pub ultimate_creditor: Option<PartyIdentification>,

    #[serde(rename = "TradgPty", skip_serializing_if = "Option::is_none")]
    pub trading_party: Option<PartyIdentification>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<Vec<ProprietaryParty>>,
}

/// Proprietary Party
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryParty {
    #[serde(rename = "Tp")]
    pub party_type: String,

    #[serde(rename = "Pty")]
    pub party: PartyIdentification,
}

/// Related Agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedAgents {
    #[serde(rename = "DbtrAgt", skip_serializing_if = "Option::is_none")]
    pub debtor_agent: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "CdtrAgt", skip_serializing_if = "Option::is_none")]
    pub creditor_agent: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt1", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent1: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt2", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent2: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt3", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent3: Option<BranchAndFinancialInstitutionIdentification>,
}

/// Purpose
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Purpose {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Remittance Location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemittanceLocation {
    #[serde(rename = "RmtId", skip_serializing_if = "Option::is_none")]
    pub remittance_id: Option<String>,

    #[serde(rename = "RmtLctnDtls", skip_serializing_if = "Option::is_none")]
    pub remittance_location_details: Option<Vec<RemittanceLocationDetails>>,
}

/// Remittance Location Details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemittanceLocationDetails {
    #[serde(rename = "Mtd")]
    pub method: String,

    #[serde(rename = "ElctrncAdr", skip_serializing_if = "Option::is_none")]
    pub electronic_address: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<NameAndAddress>,
}

/// Name And Address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameAndAddress {
    #[serde(rename = "Nm")]
    pub name: String,

    #[serde(rename = "Adr")]
    pub address: PostalAddress,
}

/// Remittance Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemittanceInformation {
    #[serde(rename = "Ustrd", skip_serializing_if = "Option::is_none")]
    pub unstructured: Option<Vec<String>>,

    #[serde(rename = "Strd", skip_serializing_if = "Option::is_none")]
    pub structured: Option<Vec<StructuredRemittanceInformation>>,
}

/// Structured Remittance Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredRemittanceInformation {
    #[serde(rename = "RfrdDocInf", skip_serializing_if = "Option::is_none")]
    pub referred_document_information: Option<Vec<ReferredDocumentInformation>>,

    #[serde(rename = "RfrdDocAmt", skip_serializing_if = "Option::is_none")]
    pub referred_document_amount: Option<RemittanceAmount>,

    #[serde(rename = "CdtrRefInf", skip_serializing_if = "Option::is_none")]
    pub creditor_reference_information: Option<CreditorReferenceInformation>,

    #[serde(rename = "Invcr", skip_serializing_if = "Option::is_none")]
    pub invoicer: Option<PartyIdentification>,

    #[serde(rename = "Invcee", skip_serializing_if = "Option::is_none")]
    pub invoicee: Option<PartyIdentification>,

    #[serde(rename = "AddtlRmtInf", skip_serializing_if = "Option::is_none")]
    pub additional_remittance_information: Option<Vec<String>>,
}

/// Referred Document Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferredDocumentInformation {
    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub document_type: Option<DocumentType>,

    #[serde(rename = "Nb", skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    #[serde(rename = "RltdDt", skip_serializing_if = "Option::is_none")]
    pub related_date: Option<String>,
}

/// Document Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentType {
    #[serde(rename = "CdOrPrtry")]
    pub code_or_proprietary: DocumentTypeCodeOrProprietary,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Document Type Code Or Proprietary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTypeCodeOrProprietary {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Remittance Amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemittanceAmount {
    #[serde(rename = "DuePyblAmt", skip_serializing_if = "Option::is_none")]
    pub due_payable_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "DscntApldAmt", skip_serializing_if = "Option::is_none")]
    pub discount_applied_amount: Option<Vec<DiscountAmountAndType>>,

    #[serde(rename = "CdtNoteAmt", skip_serializing_if = "Option::is_none")]
    pub credit_note_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "TaxAmt", skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<Vec<TaxAmountAndType>>,

    #[serde(rename = "AdjstmntAmtAndRsn", skip_serializing_if = "Option::is_none")]
    pub adjustment_amount_and_reason: Option<Vec<DocumentAdjustment>>,

    #[serde(rename = "RmtdAmt", skip_serializing_if = "Option::is_none")]
    pub remitted_amount: Option<ActiveOrHistoricCurrencyAndAmount>,
}

/// Discount Amount And Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscountAmountAndType {
    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<DiscountType>,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,
}

/// Discount Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscountType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Tax Amount And Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxAmountAndType {
    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<TaxType>,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,
}

/// Tax Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxType {
    #[serde(rename = "Ctgy", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Document Adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentAdjustment {
    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "CdtDbtInd", skip_serializing_if = "Option::is_none")]
    pub credit_debit_indicator: Option<String>,

    #[serde(rename = "Rsn", skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(rename = "AddtlInf", skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

/// Creditor Reference Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditorReferenceInformation {
    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub creditor_reference_type: Option<CreditorReferenceType>,

    #[serde(rename = "Ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

/// Creditor Reference Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditorReferenceType {
    #[serde(rename = "CdOrPrtry")]
    pub code_or_proprietary: CreditorReferenceTypeCodeOrProprietary,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Creditor Reference Type Code Or Proprietary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditorReferenceTypeCodeOrProprietary {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Related Dates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedDates {
    #[serde(rename = "AccptncDtTm", skip_serializing_if = "Option::is_none")]
    pub acceptance_date_time: Option<String>,

    #[serde(rename = "TradActvtyCtrctlSttlmDt", skip_serializing_if = "Option::is_none")]
    pub trade_activity_contractual_settlement_date: Option<String>,

    #[serde(rename = "TradDt", skip_serializing_if = "Option::is_none")]
    pub trade_date: Option<String>,

    #[serde(rename = "IntrBkSttlmDt", skip_serializing_if = "Option::is_none")]
    pub interbank_settlement_date: Option<String>,

    #[serde(rename = "StartDt", skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,

    #[serde(rename = "EndDt", skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,

    #[serde(rename = "TxDtTm", skip_serializing_if = "Option::is_none")]
    pub transaction_date_time: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<Vec<ProprietaryDate>>,
}

/// Proprietary Date
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryDate {
    #[serde(rename = "Tp")]
    pub date_type: String,

    #[serde(rename = "Dt")]
    pub date: DateOrDateTime,
}

/// Related Price
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedPrice {
    #[serde(rename = "DealPric", skip_serializing_if = "Option::is_none")]
    pub deal_price: Option<Price>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<Vec<ProprietaryPrice>>,
}

/// Price
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    #[serde(rename = "Tp")]
    pub price_type: PriceType,

    #[serde(rename = "Val")]
    pub value: PriceValue,
}

/// Price Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<ProprietaryPriceType>,
}

/// Proprietary Price Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryPriceType {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Price Value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceValue {
    #[serde(rename = "Amt", skip_serializing_if = "Option::is_none")]
    pub amount: Option<ActiveOrHistoricCurrencyAndAmount>,
}

/// Proprietary Price
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProprietaryPrice {
    #[serde(rename = "Tp")]
    pub price_type: String,

    #[serde(rename = "Pric")]
    pub price: Price,
}

/// Related Quantities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedQuantities {
    #[serde(rename = "Qty")]
    pub quantity: Vec<Quantity>,
}

/// Quantity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    #[serde(rename = "Tp")]
    pub quantity_type: QuantityType,

    #[serde(rename = "Val")]
    pub value: Decimal,
}

/// Quantity Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantityType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Security Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityIdentification {
    #[serde(rename = "ISIN", skip_serializing_if = "Option::is_none")]
    pub isin: Option<String>,

    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<OtherIdentification>>,
}

/// Other Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtherIdentification {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "SchmeNm", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<IdentificationScheme>,

    #[serde(rename = "Issr", skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

/// Identification Scheme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentificationScheme {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Tax Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxInformation {
    #[serde(rename = "Cdtr", skip_serializing_if = "Option::is_none")]
    pub creditor: Option<TaxParty>,

    #[serde(rename = "Dbtr", skip_serializing_if = "Option::is_none")]
    pub debtor: Option<TaxParty>,

    #[serde(rename = "TtlTaxblBaseAmt", skip_serializing_if = "Option::is_none")]
    pub total_taxable_base_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "TtlTaxAmt", skip_serializing_if = "Option::is_none")]
    pub total_tax_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "Rcrd", skip_serializing_if = "Option::is_none")]
    pub record: Option<Vec<TaxRecord>>,
}

/// Tax Party
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxParty {
    #[serde(rename = "TaxId", skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,

    #[serde(rename = "RegnId", skip_serializing_if = "Option::is_none")]
    pub registration_id: Option<String>,

    #[serde(rename = "TaxTp", skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,
}

/// Tax Record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxRecord {
    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub tax_record_type: Option<String>,

    #[serde(rename = "Ctgy", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    #[serde(rename = "TaxAmt", skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<TaxAmount>,
}

/// Tax Amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxAmount {
    #[serde(rename = "Rate", skip_serializing_if = "Option::is_none")]
    pub rate: Option<Decimal>,

    #[serde(rename = "TaxblBaseAmt", skip_serializing_if = "Option::is_none")]
    pub taxable_base_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "TtlAmt", skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<ActiveOrHistoricCurrencyAndAmount>,
}

/// Return Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnInformation {
    #[serde(rename = "OrgnlBkTxCd", skip_serializing_if = "Option::is_none")]
    pub original_bank_transaction_code: Option<BankTransactionCode>,

    #[serde(rename = "Orgtr", skip_serializing_if = "Option::is_none")]
    pub originator: Option<PartyIdentification>,

    #[serde(rename = "Rsn", skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReturnReason>,

    #[serde(rename = "AddtlInf", skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<Vec<String>>,
}

/// Return Reason
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnReason {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Corporate Action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorporateAction {
    #[serde(rename = "EvtTp")]
    pub event_type: String,

    #[serde(rename = "EvtId")]
    pub event_id: String,
}

/// Securities Account
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecuritiesAccount {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "Tp", skip_serializing_if = "Option::is_none")]
    pub account_type: Option<GenericIdentification>,
}

/// Cash Deposit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashDeposit {
    #[serde(rename = "NoteDnmtn")]
    pub note_denomination: ActiveCurrencyAndAmount,

    #[serde(rename = "NbOfNotes")]
    pub number_of_notes: String,

    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,
}

/// Active Currency And Amount
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveCurrencyAndAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$text")]
    pub value: Decimal,
}

/// Card Transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardTransaction {
    #[serde(rename = "Card", skip_serializing_if = "Option::is_none")]
    pub card: Option<PaymentCard>,

    #[serde(rename = "POI", skip_serializing_if = "Option::is_none")]
    pub point_of_interaction: Option<PointOfInteraction>,

    #[serde(rename = "Tx", skip_serializing_if = "Option::is_none")]
    pub transaction: Option<CardTransactionDetail>,

    #[serde(rename = "PrePdAcct", skip_serializing_if = "Option::is_none")]
    pub prepaid_account: Option<CashAccount>,
}

/// Payment Card
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentCard {
    #[serde(rename = "Tp")]
    pub card_type: String,

    #[serde(rename = "Nb")]
    pub number: String,
}

/// Point Of Interaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointOfInteraction {
    #[serde(rename = "Id")]
    pub id: GenericIdentification,

    #[serde(rename = "SysNm", skip_serializing_if = "Option::is_none")]
    pub system_name: Option<String>,

    #[serde(rename = "GrpId", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
}

/// Card Transaction Detail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardTransactionDetail {
    #[serde(rename = "ICCRltdData", skip_serializing_if = "Option::is_none")]
    pub icc_related_data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_camt053() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.10">
  <BkToCstmrStmt>
    <GrpHdr>
      <MsgId>STMT-001</MsgId>
      <CreDtTm>2026-02-10T23:59:00Z</CreDtTm>
    </GrpHdr>
    <Stmt>
      <Id>STMT-20260210</Id>
      <CreDtTm>2026-02-10T23:59:00Z</CreDtTm>
      <Acct>
        <Id>
          <IBAN>DE89370400440532013000</IBAN>
        </Id>
        <Ccy>EUR</Ccy>
        <Svcr>
          <FinInstnId>
            <BICFI>DEUTDEFF</BICFI>
          </FinInstnId>
        </Svcr>
      </Acct>
      <Bal>
        <Tp>
          <CdOrPrtry>
            <Cd>OPBD</Cd>
          </CdOrPrtry>
        </Tp>
        <Amt Ccy="EUR">5000.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <Dt>
          <Dt>2026-02-10</Dt>
        </Dt>
      </Bal>
    </Stmt>
  </BkToCstmrStmt>
</Document>"#;

        let doc: Document = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(doc.bank_to_customer_statement.group_header.message_id, "STMT-001");
        assert_eq!(doc.bank_to_customer_statement.statement.len(), 1);

        let stmt = &doc.bank_to_customer_statement.statement[0];
        assert_eq!(stmt.id, "STMT-20260210");
        assert_eq!(stmt.balance.len(), 1);
        assert_eq!(stmt.balance[0].amount.value.to_string(), "5000.00");
        assert_eq!(stmt.balance[0].amount.currency, "EUR");
    }
}
