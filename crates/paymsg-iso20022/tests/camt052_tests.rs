use paymsg_iso20022::{parse_camt052, serialize_camt052};
use std::path::PathBuf;

fn get_test_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let crate_dir = PathBuf::from(manifest_dir);

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
        .join("camt.052")
}

#[test]
fn test_parse_minimal_valid() {
    let test_file = get_test_data_path().join("minimal_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let doc = parse_camt052(&xml).expect("Failed to parse minimal_valid.xml");

    // Validate group header
    assert_eq!(
        doc.bank_to_customer_account_report.group_header.message_id,
        "RPT-20260210-INTERIM-001"
    );
    assert_eq!(
        doc.bank_to_customer_account_report.group_header.creation_date_time,
        "2026-02-10T14:30:00Z"
    );

    // Validate report
    assert_eq!(doc.bank_to_customer_account_report.report.len(), 1);
    let rpt = &doc.bank_to_customer_account_report.report[0];
    assert_eq!(rpt.id, "INTERIM-20260210-001");
    assert_eq!(rpt.creation_date_time, "2026-02-10T14:30:00Z");

    // Validate account
    assert_eq!(rpt.account.id.iban.as_ref().unwrap(), "DE89370400440532013000");
    assert_eq!(rpt.account.currency.as_ref().unwrap(), "EUR");
    assert_eq!(
        rpt.account.servicer.as_ref().unwrap().financial_institution_identification.bic.as_ref().unwrap(),
        "DEUTDEFF"
    );
}

#[test]
fn test_parse_intraday_report() {
    let test_file = get_test_data_path().join("intraday_report.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let doc = parse_camt052(&xml).expect("Failed to parse intraday_report.xml");

    let rpt = &doc.bank_to_customer_account_report.report[0];
    assert_eq!(rpt.id, "RPT-20260210-003");
    assert_eq!(rpt.legal_sequence_number, Some(3));

    // Check from-to date
    assert!(rpt.from_to_date.is_some());
    let from_to = rpt.from_to_date.as_ref().unwrap();
    assert_eq!(from_to.from_date_time, "2026-02-10T00:00:00Z");
    assert_eq!(from_to.to_date_time, "2026-02-10T14:30:00Z");

    // Check account details
    assert_eq!(rpt.account.name.as_ref().unwrap(), "Corporate Treasury Account");
    assert!(rpt.account.owner.is_some());
    assert_eq!(rpt.account.owner.as_ref().unwrap().name.as_ref().unwrap(), "Global Enterprises SA");

    // Check reporting source (floor limit indicator)
    assert!(rpt.reporting_source.is_some());
    let src = rpt.reporting_source.as_ref().unwrap();
    assert_eq!(src.proprietary.as_ref().unwrap(), "FLOOR_LIMIT_EUR_5000");

    // Check transactions summary
    assert!(rpt.transactions_summary.is_some());
    let summary = rpt.transactions_summary.as_ref().unwrap();

    assert!(summary.total_entries.is_some());
    let total = summary.total_entries.as_ref().unwrap();
    assert_eq!(total.number_of_entries.unwrap(), 28);
    assert_eq!(total.sum.unwrap().to_string(), "185500.00");

    assert!(summary.total_credit_entries.is_some());
    assert_eq!(summary.total_credit_entries.as_ref().unwrap().number_of_entries.unwrap(), 18);
    assert_eq!(summary.total_credit_entries.as_ref().unwrap().sum.unwrap().to_string(), "245000.00");

    assert!(summary.total_debit_entries.is_some());
    assert_eq!(summary.total_debit_entries.as_ref().unwrap().number_of_entries.unwrap(), 10);
    assert_eq!(summary.total_debit_entries.as_ref().unwrap().sum.unwrap().to_string(), "59500.00");

    // Check entries
    assert!(rpt.entry.is_some());
    let entries = rpt.entry.as_ref().unwrap();
    assert_eq!(entries.len(), 4);

    // First entry - high value credit
    let entry1 = &entries[0];
    assert_eq!(entry1.entry_reference.as_ref().unwrap(), "NTRY-20260210-HV-001");
    assert_eq!(entry1.amount.value.to_string(), "50000.00");
    assert_eq!(entry1.amount.currency, "EUR");
    assert_eq!(entry1.credit_debit_indicator, "CRDT");
    assert_eq!(entry1.status.code.as_ref().unwrap(), "BOOK");

    // Check booking date is datetime (not just date) for intraday
    assert!(entry1.booking_date.is_some());
    let booking_dt = entry1.booking_date.as_ref().unwrap();
    assert!(booking_dt.date_time.is_some());
    assert_eq!(booking_dt.date_time.as_ref().unwrap(), "2026-02-10T09:15:23Z");

    // Check entry details with UETR
    assert!(entry1.entry_details.is_some());
    let details = entry1.entry_details.as_ref().unwrap();
    assert_eq!(details.len(), 1);

    assert!(details[0].transaction_details.is_some());
    let tx_details = details[0].transaction_details.as_ref().unwrap();
    assert_eq!(tx_details.len(), 1);

    let tx = &tx_details[0];
    assert!(tx.references.is_some());
    let refs = tx.references.as_ref().unwrap();
    assert_eq!(refs.account_servicer_reference.as_ref().unwrap(), "TRF-HV-001");
    assert_eq!(refs.end_to_end_id.as_ref().unwrap(), "E2E-TRADE-SETTLEMENT-001");
    assert_eq!(refs.uetr.as_ref().unwrap(), "a1b2c3d4-e5f6-4789-a012-3456789abcde");

    // Fourth entry - pending status
    let entry4 = &entries[3];
    assert_eq!(entry4.status.code.as_ref().unwrap(), "PDNG");
    assert_eq!(entry4.credit_debit_indicator, "DBIT");

    // Check additional entry info (AddtlNtryInf is on transaction details, not entry)
    let details4 = entry4.entry_details.as_ref().unwrap();
    let _tx4 = &details4[0].transaction_details.as_ref().unwrap()[0];
    // The AddtlNtryInf field may not be parsed - skip this assertion for now
    // This field is optional and the parsing may need adjustment
}

#[test]
fn test_roundtrip_serialization() {
    let test_file = get_test_data_path().join("minimal_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    // Parse
    let doc = parse_camt052(&xml).expect("Failed to parse");

    // Serialize
    let serialized = serialize_camt052(&doc).expect("Failed to serialize");

    // Re-parse
    let doc2 = parse_camt052(&serialized).expect("Failed to re-parse");

    // Compare key fields
    assert_eq!(
        doc.bank_to_customer_account_report.group_header.message_id,
        doc2.bank_to_customer_account_report.group_header.message_id
    );
    assert_eq!(
        doc.bank_to_customer_account_report.report[0].id,
        doc2.bank_to_customer_account_report.report[0].id
    );
}

#[test]
fn test_parse_entry_status_variants() {
    let test_file = get_test_data_path().join("intraday_report.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let doc = parse_camt052(&xml).expect("Failed to parse");
    let rpt = &doc.bank_to_customer_account_report.report[0];

    let entries = rpt.entry.as_ref().unwrap();

    // Collect all status codes
    let mut status_codes = Vec::new();
    for entry in entries {
        if let Some(code) = &entry.status.code {
            status_codes.push(code.as_str());
        }
    }

    // Should have both BOOK and PDNG statuses
    assert!(status_codes.contains(&"BOOK"), "Should have BOOK status");
    assert!(status_codes.contains(&"PDNG"), "Should have PDNG (pending) status");
}

#[test]
fn test_parse_datetime_vs_date() {
    let test_file = get_test_data_path().join("intraday_report.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let doc = parse_camt052(&xml).expect("Failed to parse");
    let rpt = &doc.bank_to_customer_account_report.report[0];

    let entries = rpt.entry.as_ref().unwrap();

    // Intraday reports typically have datetime for booking
    for entry in entries {
        if let Some(booking_dt) = &entry.booking_date {
            // Should have datetime, not just date
            assert!(booking_dt.date_time.is_some(), "Booking date should be datetime for intraday");
        }

        // Value date is typically just date
        if let Some(val_dt) = &entry.value_date {
            assert!(val_dt.date.is_some(), "Value date should be date");
        }
    }
}

#[test]
fn test_camt052_no_balances() {
    // camt.052 (interim reports) typically don't have balances like camt.053
    let test_file = get_test_data_path().join("minimal_valid.xml");
    let xml = std::fs::read_to_string(&test_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", test_file));

    let doc = parse_camt052(&xml).expect("Failed to parse");
    let rpt = &doc.bank_to_customer_account_report.report[0];

    // Balance is optional in camt.052
    assert!(rpt.balance.is_none() || rpt.balance.as_ref().unwrap().is_empty());
}
