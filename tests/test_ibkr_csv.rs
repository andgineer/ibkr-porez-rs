use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use rust_decimal_macros::dec;

use ibkr_porez::ibkr_csv::parse_csv_activity;
use ibkr_porez::models::{Currency, TransactionType};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/resources/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap()
}

// ---------------------------------------------------------------------------
// complex_activity.csv
// ---------------------------------------------------------------------------

#[test]
fn test_parse_csv_all_transactions() {
    let content = fixture("complex_activity.csv");
    let txns = parse_csv_activity(content.as_bytes()).unwrap();
    assert_eq!(txns.len(), 4); // 2 trades + 1 dividend + 1 withholding tax
}

#[test]
fn test_parse_csv_trades() {
    let content = fixture("complex_activity.csv");
    let txns = parse_csv_activity(content.as_bytes()).unwrap();
    let trades: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Trade)
        .collect();
    assert_eq!(trades.len(), 2);

    let aapl = &trades[0];
    assert_eq!(aapl.transaction_id, "csv-XML_AAPL_BUY_1");
    assert_eq!(aapl.symbol, "AAPL");
    assert_eq!(aapl.quantity, dec!(10));
    assert_eq!(aapl.price, dec!(150));
    assert_eq!(aapl.amount, dec!(1500));
    assert_eq!(aapl.currency, Currency::USD);
    assert_eq!(aapl.date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

    let msft = &trades[1];
    assert_eq!(msft.transaction_id, "csv-MSFT_ONLY_CSV");
    assert_eq!(msft.symbol, "MSFT");
    assert_eq!(msft.quantity, dec!(100));
    assert_eq!(msft.price, dec!(300));
    assert_eq!(msft.amount, dec!(30000));
}

#[test]
fn test_parse_csv_dividends() {
    let content = fixture("complex_activity.csv");
    let txns = parse_csv_activity(content.as_bytes()).unwrap();

    let dividends: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Dividend)
        .collect();
    assert_eq!(dividends.len(), 1);
    assert_eq!(dividends[0].transaction_id, "csv-XML_DIV_KO");
    assert_eq!(dividends[0].symbol, "KO");
    assert_eq!(dividends[0].amount, dec!(50));
    assert_eq!(
        dividends[0].date,
        NaiveDate::from_ymd_opt(2023, 3, 15).unwrap()
    );
}

#[test]
fn test_parse_csv_withholding_tax() {
    let content = fixture("complex_activity.csv");
    let txns = parse_csv_activity(content.as_bytes()).unwrap();

    let wht: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::WithholdingTax)
        .collect();
    assert_eq!(wht.len(), 1);
    assert_eq!(wht[0].transaction_id, "csv-XML_TAX_KO");
    assert_eq!(wht[0].amount, dec!(-7.5));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_parse_empty_csv() {
    let txns = parse_csv_activity(b"" as &[u8]).unwrap();
    assert!(txns.is_empty());
}

#[test]
fn test_parse_csv_non_stock_skipped() {
    let csv = r#""Trades","Header","Data","Asset Category","Currency","Symbol","Date/Time","Quantity","T. Price","Proceeds","Transaction ID"
"Trades","Data","Order","Options","USD","AAPL 230120C00150000","2023-01-01","10","5","500","T1"
"#;
    let txns = parse_csv_activity(csv.as_bytes()).unwrap();
    assert!(txns.is_empty(), "non-stock asset should be skipped");
}

#[test]
fn test_parse_csv_fallback_tx_id() {
    let csv = r#""Trades","Header","Data","Asset Category","Currency","Symbol","Date/Time","Quantity","T. Price","Proceeds"
"Trades","Data","Order","Stocks","USD","AAPL","2023-01-01","10","150","1500"
"#;
    let txns = parse_csv_activity(csv.as_bytes()).unwrap();
    assert_eq!(txns.len(), 1);
    assert!(
        txns[0].transaction_id.starts_with("csv-"),
        "should synthesize csv- prefixed ID"
    );
}

#[test]
fn test_parse_csv_unknown_currency_skipped() {
    let csv = r#""Trades","Header","Data","Asset Category","Currency","Symbol","Date/Time","Quantity","T. Price","Proceeds","Transaction ID"
"Trades","Data","Order","Stocks","CHF","NESN","2023-01-01","10","100","1000","T1"
"#;
    let txns = parse_csv_activity(csv.as_bytes()).unwrap();
    assert!(txns.is_empty(), "unknown currency should be skipped");
}

#[test]
fn test_parse_csv_date_with_time() {
    let csv = r#""Trades","Header","Data","Asset Category","Currency","Symbol","Date/Time","Quantity","T. Price","Proceeds","Transaction ID"
"Trades","Data","Order","Stocks","USD","AAPL","2023-01-01, 10:30:00","10","150","1500","T1"
"#;
    let txns = parse_csv_activity(csv.as_bytes()).unwrap();
    assert_eq!(txns.len(), 1);
    assert_eq!(txns[0].date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
}
