#![allow(dead_code)]

use idx_cli::api::{NewsItem, StockQuote};
use idx_cli::app::App;
use idx_cli::config::{Config, Holding};

pub fn make_quote(symbol: &str, price: f64, change: f64, change_pct: f64) -> StockQuote {
    StockQuote {
        symbol: symbol.to_string(),
        short_name: format!("{} Corp", symbol),
        price,
        change,
        change_percent: change_pct,
        open: price - 10.0,
        high: price + 20.0,
        low: price - 20.0,
        volume: 1_000_000,
        prev_close: price - change,
        long_name: None,
        sector: None,
        industry: None,
        market_cap: None,
        trailing_pe: None,
        dividend_yield: None,
        fifty_two_week_high: None,
        fifty_two_week_low: None,
        beta: None,
        average_volume: None,
    }
}

pub fn make_news_item(title: &str, publisher: &str, ts: i64) -> NewsItem {
    NewsItem {
        title: title.to_string(),
        publisher: publisher.to_string(),
        published_at: ts,
        url: None,
        summary: None,
    }
}

pub fn make_holding(symbol: &str, lots: u32, avg_price: f64) -> Holding {
    Holding {
        symbol: symbol.to_string(),
        lots,
        avg_price,
    }
}

// Creates a default App instance for testing (no file I/O).
pub fn test_app() -> App {
    App::test_new(Config::test_config())
}
