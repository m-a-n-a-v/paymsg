// ISO 20022 pacs.008.001.10 - FI to FI Customer Credit Transfer
// Customer Credit Transfer (equivalent to MT103)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Root document wrapper for pacs.008 messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Document")]
pub struct Document {
    #[serde(rename = "FIToFICstmrCdtTrf")]
    pub fi_to_fi_customer_credit_transfer: FIToFICstmrCdtTrf,
}

/// FI to FI Customer Credit Transfer message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FIToFICstmrCdtTrf {
    #[serde(rename = "GrpHdr")]
    pub group_header: GroupHeader,

    #[serde(rename = "CdtTrfTxInf")]
    pub credit_transfer_transaction_information: Vec<CreditTransferTransactionInformation>,
}

/// Group Header - contains message-level information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupHeader {
    #[serde(rename = "MsgId")]
    pub message_id: String,

    #[serde(rename = "CreDtTm")]
    pub creation_date_time: String,

    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: String,

    #[serde(rename = "TtlIntrBkSttlmAmt", skip_serializing_if = "Option::is_none")]
    pub total_interbank_settlement_amount: Option<ActiveCurrencyAndAmount>,

    #[serde(rename = "IntrBkSttlmDt", skip_serializing_if = "Option::is_none")]
    pub interbank_settlement_date: Option<String>,

    #[serde(rename = "SttlmInf", skip_serializing_if = "Option::is_none")]
    pub settlement_information: Option<SettlementInformation>,

    #[serde(rename = "InstgAgt")]
    pub instructing_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "InstdAgt")]
    pub instructed_agent: BranchAndFinancialInstitutionIdentification,
}

/// Settlement Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettlementInformation {
    #[serde(rename = "SttlmMtd")]
    pub settlement_method: String,

    #[serde(rename = "ClrSys", skip_serializing_if = "Option::is_none")]
    pub clearing_system: Option<ClearingSystemIdentification>,
}

/// Clearing System Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearingSystemIdentification {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Credit Transfer Transaction Information - contains transaction-level details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditTransferTransactionInformation {
    #[serde(rename = "PmtId")]
    pub payment_id: PaymentIdentification,

    #[serde(rename = "PmtTpInf", skip_serializing_if = "Option::is_none")]
    pub payment_type_information: Option<PaymentTypeInformation>,

    #[serde(rename = "IntrBkSttlmAmt")]
    pub interbank_settlement_amount: ActiveCurrencyAndAmount,

    #[serde(rename = "IntrBkSttlmDt", skip_serializing_if = "Option::is_none")]
    pub interbank_settlement_date: Option<String>,

    #[serde(rename = "InstdAmt", skip_serializing_if = "Option::is_none")]
    pub instructed_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "XchgRate", skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<String>,

    #[serde(rename = "ChrgBr")]
    pub charge_bearer: String,

    #[serde(rename = "ChrgsInf", skip_serializing_if = "Option::is_none")]
    pub charges_information: Option<Vec<ChargesInformation>>,

    #[serde(rename = "IntrmyAgt1", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_1: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt2", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_2: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt3", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_3: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "Dbtr")]
    pub debtor: PartyIdentification,

    #[serde(rename = "DbtrAcct")]
    pub debtor_account: CashAccount,

    #[serde(rename = "DbtrAgt")]
    pub debtor_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "CdtrAgt")]
    pub creditor_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "Cdtr")]
    pub creditor: PartyIdentification,

    #[serde(rename = "CdtrAcct")]
    pub creditor_account: CashAccount,

    #[serde(rename = "InstrForCdtrAgt", skip_serializing_if = "Option::is_none")]
    pub instruction_for_creditor_agent: Option<Vec<InstructionForCreditorAgent>>,

    #[serde(rename = "InstrForNxtAgt", skip_serializing_if = "Option::is_none")]
    pub instruction_for_next_agent: Option<Vec<InstructionForNextAgent>>,

    #[serde(rename = "Purp", skip_serializing_if = "Option::is_none")]
    pub purpose: Option<Purpose>,

    #[serde(rename = "RgltryRptg", skip_serializing_if = "Option::is_none")]
    pub regulatory_reporting: Option<Vec<RegulatoryReporting>>,

    #[serde(rename = "RmtInf", skip_serializing_if = "Option::is_none")]
    pub remittance_information: Option<RemittanceInformation>,
}

/// Payment Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentIdentification {
    #[serde(rename = "InstrId", skip_serializing_if = "Option::is_none")]
    pub instruction_id: Option<String>,

    #[serde(rename = "EndToEndId")]
    pub end_to_end_id: String,

    #[serde(rename = "TxId", skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    #[serde(rename = "UETR", skip_serializing_if = "Option::is_none")]
    pub uetr: Option<String>,
}

/// Payment Type Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentTypeInformation {
    #[serde(rename = "InstrPrty", skip_serializing_if = "Option::is_none")]
    pub instruction_priority: Option<String>,

    #[serde(rename = "SvcLvl", skip_serializing_if = "Option::is_none")]
    pub service_level: Option<ServiceLevel>,

    #[serde(rename = "LclInstrm", skip_serializing_if = "Option::is_none")]
    pub local_instrument: Option<LocalInstrument>,

    #[serde(rename = "CtgyPurp", skip_serializing_if = "Option::is_none")]
    pub category_purpose: Option<CategoryPurpose>,
}

/// Service Level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceLevel {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Local Instrument
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalInstrument {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Category Purpose
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryPurpose {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Active Currency and Amount - for settled amounts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveCurrencyAndAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$text")]
    pub value: Decimal,
}

/// Active or Historic Currency and Amount - for instructed amounts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveOrHistoricCurrencyAndAmount {
    #[serde(rename = "@Ccy")]
    pub currency: String,

    #[serde(rename = "$text")]
    pub value: Decimal,
}

/// Charges Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargesInformation {
    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "Agt")]
    pub agent: BranchAndFinancialInstitutionIdentification,
}

/// Branch and Financial Institution Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchAndFinancialInstitutionIdentification {
    #[serde(rename = "FinInstnId")]
    pub financial_institution_id: FinancialInstitutionIdentification,
}

/// Financial Institution Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinancialInstitutionIdentification {
    #[serde(rename = "BICFI", skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,

    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<PostalAddress>,
}

/// Party Identification (Debtor/Creditor)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartyIdentification {
    #[serde(rename = "Nm", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "PstlAdr", skip_serializing_if = "Option::is_none")]
    pub postal_address: Option<PostalAddress>,

    #[serde(rename = "Id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Party>,
}

/// Party - organization or person identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "OrgId", skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<OrganizationIdentification>,

    #[serde(rename = "PrvtId", skip_serializing_if = "Option::is_none")]
    pub private_id: Option<PersonIdentification>,
}

/// Organization Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationIdentification {
    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<GenericOrganizationIdentification>>,
}

/// Person Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonIdentification {
    #[serde(rename = "Othr", skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<GenericPersonIdentification>>,
}

/// Generic Organization Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericOrganizationIdentification {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "SchmeNm", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<OrganizationIdentificationSchemeName>,
}

/// Generic Person Identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericPersonIdentification {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "SchmeNm", skip_serializing_if = "Option::is_none")]
    pub scheme_name: Option<PersonIdentificationSchemeName>,
}

/// Organization Identification Scheme Name
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationIdentificationSchemeName {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Person Identification Scheme Name
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonIdentificationSchemeName {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Postal Address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalAddress {
    #[serde(rename = "StrtNm", skip_serializing_if = "Option::is_none")]
    pub street_name: Option<String>,

    #[serde(rename = "BldgNb", skip_serializing_if = "Option::is_none")]
    pub building_number: Option<String>,

    #[serde(rename = "PstCd", skip_serializing_if = "Option::is_none")]
    pub post_code: Option<String>,

    #[serde(rename = "TwnNm", skip_serializing_if = "Option::is_none")]
    pub town_name: Option<String>,

    #[serde(rename = "CtrySubDvsn", skip_serializing_if = "Option::is_none")]
    pub country_subdivision: Option<String>,

    #[serde(rename = "Ctry", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    #[serde(rename = "AdrLine", skip_serializing_if = "Option::is_none")]
    pub address_line: Option<Vec<String>>,
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
    pub scheme_name: Option<AccountSchemeName>,
}

/// Account Scheme Name
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountSchemeName {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Cash Account Type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashAccountType {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Instruction for Creditor Agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstructionForCreditorAgent {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "InstrInf", skip_serializing_if = "Option::is_none")]
    pub instruction_information: Option<String>,
}

/// Instruction for Next Agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstructionForNextAgent {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "InstrInf", skip_serializing_if = "Option::is_none")]
    pub instruction_information: Option<String>,
}

/// Purpose
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Purpose {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

/// Regulatory Reporting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryReporting {
    #[serde(rename = "Dtls", skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<StructuredRegulatoryReporting>>,
}

/// Structured Regulatory Reporting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredRegulatoryReporting {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Inf", skip_serializing_if = "Option::is_none")]
    pub information: Option<String>,
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
    pub code_or_proprietary: CodeOrProprietary,
}

/// Code or Proprietary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeOrProprietary {
    #[serde(rename = "Cd", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(rename = "Prtry", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_parse_minimal_pacs008() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.10">
  <FIToFICstmrCdtTrf>
    <GrpHdr>
      <MsgId>MSGID-001</MsgId>
      <CreDtTm>2026-02-10T14:30:00</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPPXXX</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E-001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="EUR">1000.00</IntrBkSttlmAmt>
      <ChrgBr>SHAR</ChrgBr>
      <Dbtr>
        <Nm>ABC Corp</Nm>
      </Dbtr>
      <DbtrAcct>
        <Id>
          <IBAN>DE89370400440532013000</IBAN>
        </Id>
      </DbtrAcct>
      <DbtrAgt>
        <FinInstnId>
          <BICFI>DEUTDEFFXXX</BICFI>
        </FinInstnId>
      </DbtrAgt>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPPXXX</BICFI>
        </FinInstnId>
      </CdtrAgt>
      <Cdtr>
        <Nm>XYZ Services</Nm>
      </Cdtr>
      <CdtrAcct>
        <Id>
          <IBAN>FR1420041010050500013M02606</IBAN>
        </Id>
      </CdtrAcct>
    </CdtTrfTxInf>
  </FIToFICstmrCdtTrf>
</Document>"#;

        let result: Result<Document, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok(), "Failed to parse XML: {:?}", result.err());

        let doc = result.unwrap();
        let msg = &doc.fi_to_fi_customer_credit_transfer;

        assert_eq!(msg.group_header.message_id, "MSGID-001");
        assert_eq!(msg.group_header.number_of_transactions, "1");
        assert_eq!(msg.credit_transfer_transaction_information.len(), 1);

        let tx = &msg.credit_transfer_transaction_information[0];
        assert_eq!(tx.payment_id.end_to_end_id, "E2E-001");
        assert_eq!(tx.interbank_settlement_amount.currency, "EUR");
        assert_eq!(tx.interbank_settlement_amount.value, Decimal::from_str("1000.00").unwrap());
        assert_eq!(tx.charge_bearer, "SHAR");
    }

    #[test]
    fn test_parse_amount_with_currency() {
        let xml = r#"<IntrBkSttlmAmt Ccy="USD">25000.00</IntrBkSttlmAmt>"#;
        let result: Result<ActiveCurrencyAndAmount, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());

        let amt = result.unwrap();
        assert_eq!(amt.currency, "USD");
        assert_eq!(amt.value, Decimal::from_str("25000.00").unwrap());
    }
}
