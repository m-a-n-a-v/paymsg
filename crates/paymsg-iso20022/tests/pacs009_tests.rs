use paymsg_iso20022::{parse_pacs009, serialize_pacs009};
use std::path::PathBuf;

fn get_testdata_path() -> PathBuf {
    // When running tests, CARGO_MANIFEST_DIR points to crate directory (crates/paymsg-iso20022)
    // We need to go up 3 levels to reach the parent of the workspace, then to sibling paymsg-specs
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("paymsg-specs")
        .join("testdata")
        .join("mx")
        .join("pacs.009")
}

#[test]
fn test_parse_minimal_valid() {
    let testdata_path = get_testdata_path();
    let xml_path = testdata_path.join("minimal_valid.xml");

    assert!(
        xml_path.exists(),
        "Test data file not found: {:?}",
        xml_path
    );

    let xml = std::fs::read_to_string(&xml_path)
        .expect("Failed to read minimal_valid.xml");

    let result = parse_pacs009(&xml);
    assert!(
        result.is_ok(),
        "Failed to parse minimal_valid.xml: {:?}",
        result.err()
    );

    let doc = result.unwrap();
    let msg = &doc.fi_credit_transfer;

    // Validate group header
    assert_eq!(msg.group_header.message_id, "MSGID-20260210-001");
    assert_eq!(msg.group_header.creation_date_time, "2026-02-10T09:30:00Z");
    assert_eq!(msg.group_header.number_of_transactions, "1");

    // Validate total amount
    assert!(msg.group_header.total_interbank_settlement_amount.is_some());
    let total_amt = msg.group_header.total_interbank_settlement_amount.as_ref().unwrap();
    assert_eq!(total_amt.currency, "EUR");
    assert_eq!(total_amt.value.to_string(), "1000.00");

    // Validate settlement date
    assert_eq!(
        msg.group_header.interbank_settlement_date.as_ref().unwrap(),
        "2026-02-10"
    );

    // Validate agents at group level
    assert_eq!(
        msg.group_header.instructing_agent.financial_institution_id.bic.as_ref().unwrap(),
        "DEUTDEFF"
    );
    assert_eq!(
        msg.group_header.instructed_agent.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPP"
    );

    // Validate transaction information
    assert_eq!(msg.credit_transfer_transaction_information.len(), 1);
    let tx = &msg.credit_transfer_transaction_information[0];

    // Payment ID
    assert_eq!(tx.payment_id.instruction_id.as_ref().unwrap(), "INSTR-20260210-001");
    assert_eq!(tx.payment_id.end_to_end_id, "E2E-20260210-001");

    // Settlement amount
    assert_eq!(tx.interbank_settlement_amount.currency, "EUR");
    assert_eq!(tx.interbank_settlement_amount.value.to_string(), "1000.00");

    // Settlement date
    assert_eq!(tx.interbank_settlement_date.as_ref().unwrap(), "2026-02-10");

    // Agents at transaction level
    assert_eq!(
        tx.instructing_agent.financial_institution_id.bic.as_ref().unwrap(),
        "DEUTDEFF"
    );
    assert_eq!(
        tx.instructed_agent.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPP"
    );

    // Creditor (FI)
    assert_eq!(
        tx.creditor.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPP"
    );

    // Creditor agent
    assert_eq!(
        tx.creditor_agent.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPP"
    );
}

#[test]
fn test_parse_fi_transfer() {
    let testdata_path = get_testdata_path();
    let xml_path = testdata_path.join("fi_transfer.xml");

    assert!(
        xml_path.exists(),
        "Test data file not found: {:?}",
        xml_path
    );

    let xml = std::fs::read_to_string(&xml_path)
        .expect("Failed to read fi_transfer.xml");

    let result = parse_pacs009(&xml);
    assert!(
        result.is_ok(),
        "Failed to parse fi_transfer.xml: {:?}",
        result.err()
    );

    let doc = result.unwrap();
    let msg = &doc.fi_credit_transfer;

    // Validate message ID
    assert_eq!(msg.group_header.message_id, "FI20260210DEUTDEFF001");

    // Validate settlement information
    assert!(msg.group_header.settlement_information.is_some());
    let sttlm_inf = msg.group_header.settlement_information.as_ref().unwrap();
    assert_eq!(sttlm_inf.settlement_method, "CLRG");
    assert!(sttlm_inf.clearing_system.is_some());
    let clr_sys = sttlm_inf.clearing_system.as_ref().unwrap();
    assert_eq!(clr_sys.code.as_ref().unwrap(), "FED");

    // Validate total amount
    let total_amt = msg.group_header.total_interbank_settlement_amount.as_ref().unwrap();
    assert_eq!(total_amt.currency, "USD");
    assert_eq!(total_amt.value.to_string(), "250000.00");

    // Validate transaction details
    let tx = &msg.credit_transfer_transaction_information[0];

    // UETR
    assert_eq!(
        tx.payment_id.uetr.as_ref().unwrap(),
        "f47ac10b-58cc-4372-a567-0e02b2c3d479"
    );

    // Payment type information
    assert!(tx.payment_type_information.is_some());
    let pmt_tp = tx.payment_type_information.as_ref().unwrap();
    assert_eq!(pmt_tp.instruction_priority.as_ref().unwrap(), "HIGH");
    assert!(pmt_tp.service_level.is_some());
    assert_eq!(
        pmt_tp.service_level.as_ref().unwrap().proprietary.as_ref().unwrap(),
        "G001"
    );

    // Settlement time indication
    assert!(tx.settlement_time_indication.is_some());
    let sttlm_time = tx.settlement_time_indication.as_ref().unwrap();
    assert_eq!(
        sttlm_time.debit_date_time.as_ref().unwrap(),
        "2026-02-10T10:00:00Z"
    );

    // Intermediary agent
    assert!(tx.intermediary_agent_1.is_some());
    assert_eq!(
        tx.intermediary_agent_1.as_ref().unwrap().financial_institution_id.bic.as_ref().unwrap(),
        "BOFAUS3N"
    );

    // Creditor with full details
    let cdtr = &tx.creditor.financial_institution_id;
    assert_eq!(cdtr.bic.as_ref().unwrap(), "CHASUS33");
    assert_eq!(cdtr.name.as_ref().unwrap(), "JPMorgan Chase Bank N.A.");
    assert!(cdtr.postal_address.is_some());

    let addr = cdtr.postal_address.as_ref().unwrap();
    assert_eq!(addr.street_name.as_ref().unwrap(), "Park Avenue");
    assert_eq!(addr.building_number.as_ref().unwrap(), "383");
    assert_eq!(addr.post_code.as_ref().unwrap(), "10179");
    assert_eq!(addr.town_name.as_ref().unwrap(), "New York");
    assert_eq!(addr.country_subdivision.as_ref().unwrap(), "NY");
    assert_eq!(addr.country.as_ref().unwrap(), "US");

    // Creditor account
    assert!(tx.creditor_account.is_some());
    let cdtr_acct = tx.creditor_account.as_ref().unwrap();
    assert_eq!(
        cdtr_acct.id.other.as_ref().unwrap().id,
        "021000021-12345678"
    );

    // Instructions for next agent
    assert!(tx.instruction_for_next_agent.is_some());
    let instrs = tx.instruction_for_next_agent.as_ref().unwrap();
    assert_eq!(instrs.len(), 1);
    assert_eq!(instrs[0].code.as_ref().unwrap(), "PHOB");
    assert!(instrs[0].instruction_information.as_ref().unwrap().contains("Priority high-value"));

    // Remittance information
    assert!(tx.remittance_information.is_some());
    let rmt_inf = tx.remittance_information.as_ref().unwrap();
    assert!(rmt_inf.unstructured.is_some());
    let ustrd = rmt_inf.unstructured.as_ref().unwrap();
    assert_eq!(ustrd[0], "Interbank settlement for cross-border payment processing");
}

#[test]
fn test_roundtrip_serialization() {
    let testdata_path = get_testdata_path();
    let xml_path = testdata_path.join("minimal_valid.xml");

    let original_xml = std::fs::read_to_string(&xml_path)
        .expect("Failed to read minimal_valid.xml");

    // Parse
    let doc = parse_pacs009(&original_xml).expect("Failed to parse");

    // Serialize
    let serialized = serialize_pacs009(&doc).expect("Failed to serialize");

    // Re-parse
    let reparsed = parse_pacs009(&serialized).expect("Failed to re-parse");

    // Compare key fields
    assert_eq!(doc.fi_credit_transfer.group_header.message_id, reparsed.fi_credit_transfer.group_header.message_id);
    assert_eq!(doc.fi_credit_transfer.group_header.number_of_transactions, reparsed.fi_credit_transfer.group_header.number_of_transactions);
    assert_eq!(
        doc.fi_credit_transfer.credit_transfer_transaction_information.len(),
        reparsed.fi_credit_transfer.credit_transfer_transaction_information.len()
    );

    let tx_orig = &doc.fi_credit_transfer.credit_transfer_transaction_information[0];
    let tx_reparsed = &reparsed.fi_credit_transfer.credit_transfer_transaction_information[0];

    assert_eq!(tx_orig.payment_id.end_to_end_id, tx_reparsed.payment_id.end_to_end_id);
    assert_eq!(tx_orig.interbank_settlement_amount.currency, tx_reparsed.interbank_settlement_amount.currency);
    assert_eq!(tx_orig.interbank_settlement_amount.value, tx_reparsed.interbank_settlement_amount.value);
}

#[test]
fn test_parse_with_optional_fields() {
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
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E-001</EndToEndId>
        <TxId>TXN-001</TxId>
        <UETR>550e8400-e29b-41d4-a716-446655440000</UETR>
      </PmtId>
      <IntrBkSttlmAmt Ccy="EUR">5000.00</IntrBkSttlmAmt>
      <InstdAmt Ccy="USD">5500.00</InstdAmt>
      <XchgRate>1.10</XchgRate>
      <ChrgBr>SHAR</ChrgBr>
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

    let result = parse_pacs009(xml);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let doc = result.unwrap();
    let tx = &doc.fi_credit_transfer.credit_transfer_transaction_information[0];

    // Validate optional UETR
    assert_eq!(
        tx.payment_id.uetr.as_ref().unwrap(),
        "550e8400-e29b-41d4-a716-446655440000"
    );

    // Validate optional TxId
    assert_eq!(tx.payment_id.transaction_id.as_ref().unwrap(), "TXN-001");

    // Validate optional instructed amount
    assert!(tx.instructed_amount.is_some());
    let instd_amt = tx.instructed_amount.as_ref().unwrap();
    assert_eq!(instd_amt.currency, "USD");
    assert_eq!(instd_amt.value.to_string(), "5500.00");

    // Validate optional exchange rate
    assert_eq!(tx.exchange_rate.as_ref().unwrap(), "1.10");

    // Validate optional charge bearer
    assert_eq!(tx.charge_bearer.as_ref().unwrap(), "SHAR");
}

#[test]
fn test_parse_invalid_missing_element_should_fail() {
    // Missing mandatory InstgAgt element
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.009.001.10">
  <FICdtTrf>
    <GrpHdr>
      <MsgId>MSG-001</MsgId>
      <CreDtTm>2026-02-10T10:00:00Z</CreDtTm>
      <NbOfTxs>1</NbOfTxs>
      <InstdAgt>
        <FinInstnId>
          <BICFI>BNPAFRPP</BICFI>
        </FinInstnId>
      </InstdAgt>
    </GrpHdr>
    <CdtTrfTxInf>
      <PmtId>
        <EndToEndId>E2E-001</EndToEndId>
      </PmtId>
      <IntrBkSttlmAmt Ccy="EUR">1000.00</IntrBkSttlmAmt>
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

    let result = parse_pacs009(xml);
    assert!(result.is_err(), "Should fail due to missing mandatory element");
}
