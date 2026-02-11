// Integration tests for pacs.008 XML parsing

use paymsg_iso20022::pacs008::*;
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::str::FromStr;

fn get_test_data_path() -> PathBuf {
    // Navigate from crates/paymsg-iso20022 to the sibling paymsg-specs repo
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let crate_dir = PathBuf::from(manifest_dir);

    // Go up 3 levels to parent of workspace root, then to sibling paymsg-specs
    crate_dir
        .parent()
        .expect("parent 1")
        .parent()
        .expect("parent 2")
        .parent()
        .expect("parent 3")
        .join("paymsg-specs")
        .join("testdata")
        .join("mx")
        .join("pacs.008")
}

#[test]
fn test_parse_minimal_valid() {
    let test_file = get_test_data_path().join("minimal_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);
    assert!(result.is_ok(), "Failed to parse minimal_valid.xml: {:?}", result.err());

    let doc = result.unwrap();
    let msg = &doc.fi_to_fi_customer_credit_transfer;

    // Validate Group Header
    assert_eq!(msg.group_header.message_id, "MSGID-20260210-001");
    assert_eq!(msg.group_header.creation_date_time, "2026-02-10T14:30:00");
    assert_eq!(msg.group_header.number_of_transactions, "1");

    // Validate Instructing Agent
    assert_eq!(
        msg.group_header.instructing_agent.financial_institution_id.bic.as_ref().unwrap(),
        "DEUTDEFFXXX"
    );

    // Validate Instructed Agent
    assert_eq!(
        msg.group_header.instructed_agent.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPPXXX"
    );

    // Validate there's exactly one transaction
    assert_eq!(msg.credit_transfer_transaction_information.len(), 1);

    let tx = &msg.credit_transfer_transaction_information[0];

    // Validate Payment ID
    assert_eq!(tx.payment_id.instruction_id.as_ref().unwrap(), "INSTR-20260210-001");
    assert_eq!(tx.payment_id.end_to_end_id, "E2E-20260210-001");

    // Validate Interbank Settlement Amount
    assert_eq!(tx.interbank_settlement_amount.currency, "EUR");
    assert_eq!(tx.interbank_settlement_amount.value, Decimal::from_str("1000.00").unwrap());

    // Validate Settlement Date
    assert_eq!(tx.interbank_settlement_date.as_ref().unwrap(), "2026-02-11");

    // Validate Charge Bearer
    assert_eq!(tx.charge_bearer, "SHAR");

    // Validate Debtor
    assert_eq!(tx.debtor.name.as_ref().unwrap(), "ABC Corporation");

    // Validate Debtor Account
    assert_eq!(
        tx.debtor_account.id.iban.as_ref().unwrap(),
        "DE89370400440532013000"
    );

    // Validate Debtor Agent
    assert_eq!(
        tx.debtor_agent.financial_institution_id.bic.as_ref().unwrap(),
        "DEUTDEFFXXX"
    );

    // Validate Creditor Agent
    assert_eq!(
        tx.creditor_agent.financial_institution_id.bic.as_ref().unwrap(),
        "BNPAFRPPXXX"
    );

    // Validate Creditor
    assert_eq!(tx.creditor.name.as_ref().unwrap(), "XYZ Services");

    // Validate Creditor Account
    assert_eq!(
        tx.creditor_account.id.iban.as_ref().unwrap(),
        "FR1420041010050500013M02606"
    );
}

#[test]
fn test_parse_full_valid() {
    let test_file = get_test_data_path().join("full_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);
    assert!(result.is_ok(), "Failed to parse full_valid.xml: {:?}", result.err());

    let doc = result.unwrap();
    let msg = &doc.fi_to_fi_customer_credit_transfer;

    // Validate Group Header
    assert_eq!(msg.group_header.message_id, "MSGID-20260210-002");
    assert_eq!(msg.group_header.creation_date_time, "2026-02-10T15:45:30");
    assert_eq!(msg.group_header.number_of_transactions, "1");

    // Validate Total Interbank Settlement Amount
    let total_amt = msg.group_header.total_interbank_settlement_amount.as_ref().unwrap();
    assert_eq!(total_amt.currency, "USD");
    assert_eq!(total_amt.value, Decimal::from_str("25000.00").unwrap());

    // Validate Settlement Information
    let sttlm_inf = msg.group_header.settlement_information.as_ref().unwrap();
    assert_eq!(sttlm_inf.settlement_method, "CLRG");
    assert_eq!(
        sttlm_inf.clearing_system.as_ref().unwrap().proprietary.as_ref().unwrap(),
        "FEDWIRE"
    );

    let tx = &msg.credit_transfer_transaction_information[0];

    // Validate UETR
    assert_eq!(
        tx.payment_id.uetr.as_ref().unwrap(),
        "8a562c9d-3e41-4a9c-a614-2b7e8f9c1a5d"
    );

    // Validate Payment Type Information
    let pmt_tp = tx.payment_type_information.as_ref().unwrap();
    assert_eq!(pmt_tp.instruction_priority.as_ref().unwrap(), "NORM");
    assert_eq!(pmt_tp.service_level.as_ref().unwrap().proprietary.as_ref().unwrap(), "G001");
    assert_eq!(pmt_tp.local_instrument.as_ref().unwrap().proprietary.as_ref().unwrap(), "WIRE");
    assert_eq!(pmt_tp.category_purpose.as_ref().unwrap().code.as_ref().unwrap(), "SUPP");

    // Validate Instructed Amount (different from settlement amount)
    let instd_amt = tx.instructed_amount.as_ref().unwrap();
    assert_eq!(instd_amt.currency, "USD");
    assert_eq!(instd_amt.value, Decimal::from_str("25100.00").unwrap());

    // Validate Exchange Rate
    assert_eq!(tx.exchange_rate.as_ref().unwrap(), "1.004");

    // Validate Charge Bearer
    assert_eq!(tx.charge_bearer, "DEBT");

    // Validate Charges Information
    let charges = tx.charges_information.as_ref().unwrap();
    assert_eq!(charges.len(), 2);
    assert_eq!(charges[0].amount.value, Decimal::from_str("25.00").unwrap());
    assert_eq!(charges[1].amount.value, Decimal::from_str("75.00").unwrap());

    // Validate Intermediary Agent
    let intrmry_agt = tx.intermediary_agent_1.as_ref().unwrap();
    assert_eq!(
        intrmry_agt.financial_institution_id.bic.as_ref().unwrap(),
        "CITIUS33XXX"
    );
    assert_eq!(
        intrmry_agt.financial_institution_id.name.as_ref().unwrap(),
        "Citibank NA"
    );

    // Validate Debtor with postal address
    assert_eq!(tx.debtor.name.as_ref().unwrap(), "TechCorp Industries Inc");
    let dbtr_addr = tx.debtor.postal_address.as_ref().unwrap();
    assert_eq!(dbtr_addr.street_name.as_ref().unwrap(), "500 Market Street");
    assert_eq!(dbtr_addr.building_number.as_ref().unwrap(), "Suite 2500");
    assert_eq!(dbtr_addr.post_code.as_ref().unwrap(), "94105");
    assert_eq!(dbtr_addr.town_name.as_ref().unwrap(), "San Francisco");
    assert_eq!(dbtr_addr.country_subdivision.as_ref().unwrap(), "CA");
    assert_eq!(dbtr_addr.country.as_ref().unwrap(), "US");

    // Validate Debtor Organization ID
    let dbtr_id = tx.debtor.id.as_ref().unwrap();
    let org_id = dbtr_id.organization_id.as_ref().unwrap();
    let other_ids = org_id.other.as_ref().unwrap();
    assert_eq!(other_ids[0].id, "12-3456789");
    assert_eq!(
        other_ids[0].scheme_name.as_ref().unwrap().proprietary.as_ref().unwrap(),
        "TXID"
    );

    // Validate Debtor Account with account type
    let dbtr_acct = &tx.debtor_account;
    assert_eq!(dbtr_acct.id.other.as_ref().unwrap().id, "1234567890");
    assert_eq!(
        dbtr_acct.account_type.as_ref().unwrap().proprietary.as_ref().unwrap(),
        "CHECKING"
    );
    assert_eq!(dbtr_acct.currency.as_ref().unwrap(), "USD");

    // Validate Creditor with postal address
    assert_eq!(tx.creditor.name.as_ref().unwrap(), "Global Manufacturing LLC");
    let cdtr_addr = tx.creditor.postal_address.as_ref().unwrap();
    assert_eq!(cdtr_addr.street_name.as_ref().unwrap(), "1000 Industrial Boulevard");
    assert_eq!(cdtr_addr.post_code.as_ref().unwrap(), "10001");
    assert_eq!(cdtr_addr.town_name.as_ref().unwrap(), "New York");
    assert_eq!(cdtr_addr.country.as_ref().unwrap(), "US");

    // Validate Instructions for Creditor Agent
    let instr_cdtr_agt = tx.instruction_for_creditor_agent.as_ref().unwrap();
    assert_eq!(instr_cdtr_agt[0].code.as_ref().unwrap(), "PHOB");
    assert!(instr_cdtr_agt[0].instruction_information.as_ref().unwrap().contains("Phone beneficiary"));

    // Validate Instructions for Next Agent
    let instr_nxt_agt = tx.instruction_for_next_agent.as_ref().unwrap();
    assert!(instr_nxt_agt[0].instruction_information.as_ref().unwrap().contains("Priority payment"));

    // Validate Purpose
    assert_eq!(tx.purpose.as_ref().unwrap().code.as_ref().unwrap(), "SUPP");

    // Validate Regulatory Reporting
    let reg_rptg = tx.regulatory_reporting.as_ref().unwrap();
    assert_eq!(reg_rptg.len(), 1);
    let dtls = reg_rptg[0].details.as_ref().unwrap();
    assert_eq!(dtls[0].code.as_ref().unwrap(), "999");
    assert_eq!(dtls[0].information.as_ref().unwrap(), "Export payment - machinery");

    // Validate Remittance Information
    let rmt_inf = tx.remittance_information.as_ref().unwrap();

    // Unstructured remittance
    let ustrd = rmt_inf.unstructured.as_ref().unwrap();
    assert_eq!(ustrd.len(), 1);
    assert!(ustrd[0].contains("Payment for invoice INV-2026-001"));

    // Structured remittance
    let strd = rmt_inf.structured.as_ref().unwrap();
    assert_eq!(strd.len(), 1);
    let rfrd_doc = strd[0].referred_document_information.as_ref().unwrap();
    assert_eq!(rfrd_doc[0].document_type.as_ref().unwrap().code_or_proprietary.code.as_ref().unwrap(), "CINV");
    assert_eq!(rfrd_doc[0].number.as_ref().unwrap(), "INV-2026-001");
    assert_eq!(rfrd_doc[0].related_date.as_ref().unwrap(), "2026-01-15");
}

#[test]
fn test_parse_sepa_credit() {
    let test_file = get_test_data_path().join("sepa_credit.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);
    assert!(result.is_ok(), "Failed to parse sepa_credit.xml: {:?}", result.err());

    let doc = result.unwrap();
    let msg = &doc.fi_to_fi_customer_credit_transfer;
    let tx = &msg.credit_transfer_transaction_information[0];

    // SEPA payments use EUR
    assert_eq!(tx.interbank_settlement_amount.currency, "EUR");

    // SEPA requires IBAN for both accounts
    assert!(tx.debtor_account.id.iban.is_some());
    assert!(tx.creditor_account.id.iban.is_some());
}

#[test]
fn test_parse_usd_wire() {
    let test_file = get_test_data_path().join("usd_wire.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);
    assert!(result.is_ok(), "Failed to parse usd_wire.xml: {:?}", result.err());

    let doc = result.unwrap();
    let msg = &doc.fi_to_fi_customer_credit_transfer;
    let tx = &msg.credit_transfer_transaction_information[0];

    // USD wires
    assert_eq!(tx.interbank_settlement_amount.currency, "USD");
}

#[test]
fn test_parse_multi_transaction() {
    let test_file = get_test_data_path().join("multi_transaction.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);
    assert!(result.is_ok(), "Failed to parse multi_transaction.xml: {:?}", result.err());

    let doc = result.unwrap();
    let msg = &doc.fi_to_fi_customer_credit_transfer;

    // Should have multiple transactions
    assert!(msg.credit_transfer_transaction_information.len() > 1);

    // Validate number of transactions matches
    let nb_txs: usize = msg.group_header.number_of_transactions.parse().unwrap();
    assert_eq!(msg.credit_transfer_transaction_information.len(), nb_txs);
}

#[test]
fn test_parse_invalid_missing_element_should_fail() {
    let test_file = get_test_data_path().join("invalid_missing_element.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let result: Result<Document, _> = quick_xml::de::from_str(&xml);

    // This should fail because mandatory elements are missing
    assert!(result.is_err(), "Expected parsing to fail for invalid_missing_element.xml");
}

#[test]
fn test_roundtrip_serialization() {
    let test_file = get_test_data_path().join("minimal_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    // Parse
    let doc: Document = quick_xml::de::from_str(&xml).expect("Failed to parse");

    // Serialize
    let serialized = paymsg_iso20022::serialize_pacs008(&doc).expect("Failed to serialize");

    // Parse again
    let doc2: Document = quick_xml::de::from_str(&serialized).expect("Failed to re-parse");

    // Compare key fields (full comparison would be complex due to whitespace/formatting)
    assert_eq!(
        doc.fi_to_fi_customer_credit_transfer.group_header.message_id,
        doc2.fi_to_fi_customer_credit_transfer.group_header.message_id
    );

    let tx1 = &doc.fi_to_fi_customer_credit_transfer.credit_transfer_transaction_information[0];
    let tx2 = &doc2.fi_to_fi_customer_credit_transfer.credit_transfer_transaction_information[0];

    assert_eq!(tx1.payment_id.end_to_end_id, tx2.payment_id.end_to_end_id);
    assert_eq!(tx1.interbank_settlement_amount.currency, tx2.interbank_settlement_amount.currency);
    assert_eq!(tx1.interbank_settlement_amount.value, tx2.interbank_settlement_amount.value);
}
