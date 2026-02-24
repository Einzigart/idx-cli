mod common;

use common::test_app;
use idx_cli::api::StockQuote;
use idx_cli::app::{InputMode, ViewMode};
use idx_cli::config::{Alert, AlertType};

#[test]
fn check_alerts_fires_when_price_matches() {
    let mut app = test_app();
    let alert = Alert::new("BBCA", AlertType::Above, 8000.0);
    app.config.add_alert(alert);

    let quote = StockQuote {
        symbol: "BBCA".to_string(),
        short_name: "Bank Mandiri".to_string(),
        price: 8001.0,
        change: 100.0,
        change_percent: 1.0,
        open: 7900.0,
        high: 8050.0,
        low: 7900.0,
        volume: 1_000_000,
        prev_close: 7900.0,
        long_name: Some("PT Bank Mandiri".to_string()),
        sector: Some("Financial".to_string()),
        industry: Some("Banking".to_string()),
        market_cap: Some(100_000_000_000),
        trailing_pe: Some(10.0),
        dividend_yield: Some(2.5),
        fifty_two_week_high: Some(9000.0),
        fifty_two_week_low: Some(7000.0),
        beta: Some(1.2),
        average_volume: Some(500_000),
    };
    app.quotes.insert("BBCA".to_string(), quote);

    let triggered = app.check_alerts();
    assert_eq!(triggered.len(), 1);
    assert!(triggered[0].1.contains("crossed above"));
}

#[test]
fn open_alert_modal_returns_to_normal_when_no_symbol() {
    let mut app = test_app();
    app.view_mode = ViewMode::Watchlist;
    app.config.watchlists[0].symbols.clear();
    app.selected_index = 0;
    app.open_alert_modal();
    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.alert_symbol, None);
}
