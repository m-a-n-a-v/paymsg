// ISO 20022 pacs.009.001.10 - Financial Institution Credit Transfer
// FI Credit Transfer (equivalent to MT202)

use serde::{Deserialize, Serialize};

// Re-use common types from pacs008
use crate::pacs008::{
    ActiveCurrencyAndAmount, ActiveOrHistoricCurrencyAndAmount, BranchAndFinancialInstitutionIdentification,
    CashAccount, InstructionForNextAgent, PaymentIdentification,
    PaymentTypeInformation, Purpose, RegulatoryReporting, RemittanceInformation, SettlementInformation,
};

/// Root document wrapper for pacs.009 messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Document")]
pub struct Document {
    #[serde(rename = "FICdtTrf")]
    pub fi_credit_transfer: FICdtTrf,
}

/// FI Credit Transfer message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FICdtTrf {
    #[serde(rename = "GrpHdr")]
    pub group_header: GroupHeader,

    #[serde(rename = "CdtTrfTxInf")]
    pub credit_transfer_transaction_information: Vec<CreditTransferTransactionInformation>,
}

/// Group Header - contains message-level information
/// Similar to pacs.008 but for FI transfers
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

/// Credit Transfer Transaction Information - contains transaction-level details
/// Different from pacs.008: no Dbtr/DbtrAcct (only FI agents), Cdtr is FI not customer
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

    #[serde(rename = "SttlmTmIndctn", skip_serializing_if = "Option::is_none")]
    pub settlement_time_indication: Option<SettlementTimeIndication>,

    #[serde(rename = "SttlmTmReq", skip_serializing_if = "Option::is_none")]
    pub settlement_time_request: Option<SettlementTimeRequest>,

    #[serde(rename = "InstdAmt", skip_serializing_if = "Option::is_none")]
    pub instructed_amount: Option<ActiveOrHistoricCurrencyAndAmount>,

    #[serde(rename = "XchgRate", skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<String>,

    #[serde(rename = "ChrgBr", skip_serializing_if = "Option::is_none")]
    pub charge_bearer: Option<String>,

    #[serde(rename = "ChrgsInf", skip_serializing_if = "Option::is_none")]
    pub charges_information: Option<Vec<ChargesInformation>>,

    #[serde(rename = "InstgAgt")]
    pub instructing_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "InstdAgt")]
    pub instructed_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "IntrmyAgt1", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_1: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt2", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_2: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "IntrmyAgt3", skip_serializing_if = "Option::is_none")]
    pub intermediary_agent_3: Option<BranchAndFinancialInstitutionIdentification>,

    #[serde(rename = "Cdtr")]
    pub creditor: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "CdtrAcct", skip_serializing_if = "Option::is_none")]
    pub creditor_account: Option<CashAccount>,

    #[serde(rename = "CdtrAgt")]
    pub creditor_agent: BranchAndFinancialInstitutionIdentification,

    #[serde(rename = "InstrForNxtAgt", skip_serializing_if = "Option::is_none")]
    pub instruction_for_next_agent: Option<Vec<InstructionForNextAgent>>,

    #[serde(rename = "Purp", skip_serializing_if = "Option::is_none")]
    pub purpose: Option<Purpose>,

    #[serde(rename = "RgltryRptg", skip_serializing_if = "Option::is_none")]
    pub regulatory_reporting: Option<Vec<RegulatoryReporting>>,

    #[serde(rename = "RmtInf", skip_serializing_if = "Option::is_none")]
    pub remittance_information: Option<RemittanceInformation>,
}

/// Settlement Time Indication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettlementTimeIndication {
    #[serde(rename = "DbtDtTm", skip_serializing_if = "Option::is_none")]
    pub debit_date_time: Option<String>,

    #[serde(rename = "CdtDtTm", skip_serializing_if = "Option::is_none")]
    pub credit_date_time: Option<String>,
}

/// Settlement Time Request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettlementTimeRequest {
    #[serde(rename = "CLSTm", skip_serializing_if = "Option::is_none")]
    pub close_time: Option<String>,

    #[serde(rename = "TillTm", skip_serializing_if = "Option::is_none")]
    pub till_time: Option<String>,

    #[serde(rename = "FrTm", skip_serializing_if = "Option::is_none")]
    pub from_time: Option<String>,

    #[serde(rename = "RjctTm", skip_serializing_if = "Option::is_none")]
    pub reject_time: Option<String>,
}

/// Charges Information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargesInformation {
    #[serde(rename = "Amt")]
    pub amount: ActiveOrHistoricCurrencyAndAmount,

    #[serde(rename = "Agt")]
    pub agent: BranchAndFinancialInstitutionIdentification,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_parse_minimal_pacs009() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.10">
  <FICdtTrf>
    <GrpHdr>
      <MsgId>MSGID-20260210-001</MsgId>
      <CreDtTm>2026-02-10T09:30:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <TtlIntrBkSttlmAmt Ccy="EUR">1000.00</TtlIntrBkSttlmAmt>
      <IntrBkSttlmDt>2026-02-10</IntrBkSttlmDt>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <InstrId>INSTR-20260210-001</InstrId>
        <EndToEndId>E2E-20260210-001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="EUR">1000.00</IntrBkSttlmAmt>
      <IntrBkSttlmDt>2026-02-10</IntrBkSttlmDt>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
      <Cdtr>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </Cdtr>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </CdtrAgt>
    </CdtTrfTxInf>
  </FICdtTrf>
</Document>"#;

        let result: Result<Document, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok(), "Failed to parse XML: {:?}", result.err());

        let doc = result.unwrap();
        let msg = &doc.fi_credit_transfer;

        assert_eq!(msg.group_header.message_id, "MSGID-20260210-001");
        assert_eq!(msg.group_header.number_of_transactions, "1");
        assert_eq!(msg.credit_transfer_transaction_information.len(), 1);

        let tx = &msg.credit_transfer_transaction_information[0];
        assert_eq!(tx.payment_id.end_to_end_id, "E2E-20260210-001");
        assert_eq!(tx.interbank_settlement_amount.currency, "EUR");
        assert_eq!(
            tx.interbank_settlement_amount.value,
            Decimal::from_str("1000.00").unwrap()
        );
    }

    #[test]
    fn test_parse_pacs009_with_settlement_time() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.10">
  <FICdtTrf>
    <GrpHdr>
      <MsgId>MSG-001</MsgId>
      <CreDtTm>2026-02-10T10:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>CHASUS33</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E-001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="USD">50000.00</IntrBkSttlmAmt>
      <SttlmTmIndctn>
        <DbtDtTm>2026-02-10T10:00:00Z</DbtDtTm>
      </SttlmTmIndctn>
      <InstgAgt>
        <FinInstnId>
          <BICFI>DEUTDEFF</BICFI>
        </FinInstnId>
      </InstgAgt>
      <InstdAgt>
        <FinInstnId>
          <BICFI>CHASUS33</BICFI>
        </FinInstnId>
      </InstdAgt>
      <Cdtr>
        <FinInstnId>
          <BICFI>CHASUS33</BICFI>
        </FinInstnId>
      </Cdtr>
      <CdtrAgt>
        <FinInstnId>
          <BICFI>CHASUS33</BICFI>
        </FinInstnId>
      </CdtrAgt>
    </CdtTrfTxInf>
  </FICdtTrf>
</Document>"#;

        let result: Result<Document, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok(), "Failed to parse XML: {:?}", result.err());

        let doc = result.unwrap();
        let tx = &doc.fi_credit_transfer.credit_transfer_transaction_information[0];

        assert!(tx.settlement_time_indication.is_some());
        let sttlm_time = tx.settlement_time_indication.as_ref().unwrap();
        assert_eq!(
            sttlm_time.debit_date_time.as_ref().unwrap(),
            "2026-02-10T10:00:00Z"
        );
    }
}
