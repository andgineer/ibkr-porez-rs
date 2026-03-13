use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use rust_decimal_macros::dec;

use ibkr_porez::ibkr_flex::parse_flex_report;
use ibkr_porez::models::TransactionType;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/resources/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap()
}

// ---------------------------------------------------------------------------
// complex_flex.xml
// ---------------------------------------------------------------------------

#[test]
fn test_parse_complex_flex_all_transactions() {
    let xml = fixture("complex_flex.xml");
    let txns = parse_flex_report(&xml).unwrap();
    assert_eq!(txns.len(), 7); // 4 trades + 3 cash transactions
}

#[test]
fn test_parse_complex_flex_trades() {
    let xml = fixture("complex_flex.xml");
    let txns = parse_flex_report(&xml).unwrap();
    let trades: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Trade)
        .collect();
    assert_eq!(trades.len(), 4);

    let aapl = &trades[0];
    assert_eq!(aapl.transaction_id, "XML_AAPL_BUY_1");
    assert_eq!(aapl.symbol, "AAPL");
    assert_eq!(aapl.quantity, dec!(10));
    assert_eq!(aapl.price, dec!(150.0));
    assert_eq!(aapl.amount, dec!(300.0));
    assert_eq!(aapl.date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    assert_eq!(
        aapl.open_date,
        Some(NaiveDate::from_ymd_opt(2022, 6, 1).unwrap())
    );
    assert_eq!(aapl.open_price, Some(dec!(120.0)));
    assert_eq!(aapl.description, "Apple Inc.");

    let tsla = &trades[1];
    assert_eq!(tsla.transaction_id, "XML_TSLA_SELL_1");
    assert_eq!(tsla.symbol, "TSLA");
    assert_eq!(tsla.quantity, dec!(-5));
    assert_eq!(tsla.price, dec!(200.0));
    assert_eq!(tsla.amount, dec!(-100.0));

    let goog1 = &trades[2];
    assert_eq!(goog1.transaction_id, "XML_GOOG_SPLIT_1");
    assert_eq!(goog1.quantity, dec!(50));
    assert_eq!(goog1.price, dec!(100.0));
    assert!(goog1.open_date.is_none());

    let goog2 = &trades[3];
    assert_eq!(goog2.transaction_id, "XML_GOOG_SPLIT_2");
    assert_eq!(goog2.quantity, dec!(50));
    assert_eq!(goog2.price, dec!(100.1));
}

#[test]
fn test_parse_complex_flex_cash_transactions() {
    let xml = fixture("complex_flex.xml");
    let txns = parse_flex_report(&xml).unwrap();

    let dividends: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Dividend)
        .collect();
    assert_eq!(dividends.len(), 1);
    let div = &dividends[0];
    assert_eq!(div.transaction_id, "XML_DIV_KO");
    assert_eq!(div.symbol, "KO");
    assert_eq!(div.amount, dec!(50.0));
    assert_eq!(div.date, NaiveDate::from_ymd_opt(2023, 3, 15).unwrap());

    let wht: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::WithholdingTax)
        .collect();
    assert_eq!(wht.len(), 1);
    assert_eq!(wht[0].transaction_id, "XML_TAX_KO");
    assert_eq!(wht[0].amount, dec!(-7.5));

    let interest: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Interest)
        .collect();
    assert_eq!(interest.len(), 1);
    assert_eq!(interest[0].transaction_id, "XML_INTEREST_MAR");
    assert_eq!(interest[0].amount, dec!(-2.5));
    assert_eq!(
        interest[0].date,
        NaiveDate::from_ymd_opt(2023, 3, 31).unwrap()
    );
}

// ---------------------------------------------------------------------------
// fifo_scenarios.xml
// ---------------------------------------------------------------------------

#[test]
fn test_parse_fifo_scenarios() {
    let xml = fixture("fifo_scenarios.xml");
    let txns = parse_flex_report(&xml).unwrap();
    let trades: Vec<_> = txns
        .iter()
        .filter(|t| t.r#type == TransactionType::Trade)
        .collect();
    assert_eq!(trades.len(), 6);

    // AAPL scenario: buy 10@100, buy 10@110, sell 15@120
    assert_eq!(trades[0].transaction_id, "BUY_AAPL_1");
    assert_eq!(trades[0].quantity, dec!(10));
    assert_eq!(trades[0].price, dec!(100.0));

    assert_eq!(trades[1].transaction_id, "BUY_AAPL_2");
    assert_eq!(trades[1].quantity, dec!(10));
    assert_eq!(trades[1].price, dec!(110.0));

    assert_eq!(trades[2].transaction_id, "SELL_AAPL_1");
    assert_eq!(trades[2].quantity, dec!(-15));
    assert_eq!(trades[2].price, dec!(120.0));

    // MSFT scenario: buy 20@200, sell 5@210, sell 10@220
    assert_eq!(trades[3].transaction_id, "BUY_MSFT_1");
    assert_eq!(trades[3].quantity, dec!(20));

    assert_eq!(trades[4].transaction_id, "SELL_MSFT_1");
    assert_eq!(trades[4].quantity, dec!(-5));
    assert_eq!(trades[4].price, dec!(210.0));

    assert_eq!(trades[5].transaction_id, "SELL_MSFT_2");
    assert_eq!(trades[5].quantity, dec!(-10));
    assert_eq!(trades[5].price, dec!(220.0));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_parse_empty_xml() {
    let xml = r"<FlexQueryResponse>
        <FlexStatements>
            <FlexStatement>
                <Trades />
                <CashTransactions />
            </FlexStatement>
        </FlexStatements>
    </FlexQueryResponse>";
    let txns = parse_flex_report(xml).unwrap();
    assert!(txns.is_empty());
}

#[test]
fn test_parse_incomplete_trade_skipped() {
    let xml = r#"<FlexQueryResponse>
        <FlexStatements>
            <FlexStatement>
                <Trades>
                    <Trade symbol="AAPL" currency="USD" />
                </Trades>
            </FlexStatement>
        </FlexStatements>
    </FlexQueryResponse>"#;
    let txns = parse_flex_report(xml).unwrap();
    assert!(txns.is_empty(), "incomplete trade should be skipped");
}

#[test]
fn test_parse_unknown_currency_skipped() {
    let xml = r#"<FlexQueryResponse>
        <FlexStatements>
            <FlexStatement>
                <Trades>
                    <Trade symbol="AAPL" currency="CHF" quantity="10"
                           tradePrice="100" tradeDate="20230101" tradeID="T1" />
                </Trades>
            </FlexStatement>
        </FlexStatements>
    </FlexQueryResponse>"#;
    let txns = parse_flex_report(xml).unwrap();
    assert!(txns.is_empty(), "unknown currency should be skipped");
}

#[test]
fn test_parse_unknown_cash_type_skipped() {
    let xml = r#"<FlexQueryResponse>
        <FlexStatements>
            <FlexStatement>
                <CashTransactions>
                    <CashTransaction type="Commission Adjustments" symbol="AAPL"
                        currency="USD" amount="5" dateTime="20230101" transactionID="T1" />
                </CashTransactions>
            </FlexStatement>
        </FlexStatements>
    </FlexQueryResponse>"#;
    let txns = parse_flex_report(xml).unwrap();
    assert!(txns.is_empty());
}

#[test]
fn test_parse_error_response() {
    let xml = r"<FlexStatementResponse>
        <Status>Fail</Status>
        <ErrorCode>1234</ErrorCode>
        <ErrorMessage>Token expired</ErrorMessage>
    </FlexStatementResponse>";
    let err = parse_flex_report(xml).unwrap_err();
    assert!(err.to_string().contains("1234"));
    assert!(err.to_string().contains("Token expired"));
}

#[test]
fn test_payment_in_lieu_is_dividend() {
    let xml = r#"<FlexQueryResponse>
        <FlexStatements>
            <FlexStatement>
                <CashTransactions>
                    <CashTransaction type="Payment In Lieu Of Dividends" symbol="XYZ"
                        currency="USD" amount="25" dateTime="20230601" transactionID="PIL1"
                        description="PIL dividend" />
                </CashTransactions>
            </FlexStatement>
        </FlexStatements>
    </FlexQueryResponse>"#;
    let txns = parse_flex_report(xml).unwrap();
    assert_eq!(txns.len(), 1);
    assert_eq!(txns[0].r#type, TransactionType::Dividend);
}
