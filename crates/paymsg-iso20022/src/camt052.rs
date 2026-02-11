//! camt.052.001.10 Bank-to-Customer Account Report (Interim/Intraday Report)
//!
//! Equivalent to MT942 (Interim Transaction Report)

use serde::{Deserialize, Serialize};

// Reuse common types from camt053
use crate::camt053::{
    Balance, CashAccount, DateTimePeriod, Entry, GroupHeader as Camt053GroupHeader, Pagination,
    ReportingSource, TransactionsSummary,
};

/// Document root for camt.052
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "BkToCstmrAcctRpt")]
    pub bank_to_customer_account_report: BankToCustomerAccountReport,
}

/// BankToCustomerAccountReport main container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BankToCustomerAccountReport {
    #[serde(rename = "GrpHdr")]
    pub group_header: GroupHeader,

    #[serde(rename = "Rpt")]
    pub report: Vec<AccountReport>,

    #[serde(rename = "AddtlRptInf", skip_serializing_if = "Option::is_none")]
    pub additional_report_info: Option<String>,
}

/// GroupHeader for camt.052
/// Same structure as camt.053 GroupHeader, just creating an alias
pub type GroupHeader = Camt053GroupHeader;

/// Account Report (similar to Statement in camt.053 but for interim reports)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountReport {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "RptPgntn", skip_serializing_if = "Option::is_none")]
    pub report_pagination: Option<Pagination>,

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
    pub interest: Option<Vec<crate::camt053::InterestRecord>>,

    #[serde(rename = "Bal", skip_serializing_if = "Option::is_none")]
    pub balance: Option<Vec<Balance>>,

    #[serde(rename = "TxsSummary", skip_serializing_if = "Option::is_none")]
    pub transactions_summary: Option<TransactionsSummary>,

    #[serde(rename = "Ntry", skip_serializing_if = "Option::is_none")]
    pub entry: Option<Vec<Entry>>,

    #[serde(rename = "AddtlRptInf", skip_serializing_if = "Option::is_none")]
    pub additional_report_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_camt052() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.052.001.10">
  <BkToCstmrAcctRpt>
    <GrpHdr>
      <MsgId>RPT-001</MsgId>
      <CreDtTm>2026-02-10T14:30:00Z</CreDtTm>
    </GrpHdr>
    <Rpt>
      <Id>RPT-20260210</Id>
      <CreDtTm>2026-02-10T14:30:00Z</CreDtTm>
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
    </Rpt>
  </BkToCstmrAcctRpt>
</Document>"#;

        let doc: Document = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(
            doc.bank_to_customer_account_report.group_header.message_id,
            "RPT-001"
        );
        assert_eq!(doc.bank_to_customer_account_report.report.len(), 1);

        let rpt = &doc.bank_to_customer_account_report.report[0];
        assert_eq!(rpt.id, "RPT-20260210");
        assert_eq!(
            rpt.account
                .id
                .iban
                .as_ref()
                .unwrap(),
            "DE89370400440532013000"
        );
    }
}
