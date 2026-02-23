use super::SortDirection;
use crate::api::{NewsItem, StockQuote};
use crate::config::Holding;
use std::cmp::Ordering;
use std::collections::HashMap;

pub fn cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

pub fn compare_watchlist_column(
    col: usize,
    a: &(&String, Option<&StockQuote>),
    b: &(&String, Option<&StockQuote>),
    direction: SortDirection,
) -> Ordering {
    match (a.1, b.1) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(qa), Some(qb)) => {
            let ord = match col {
                0 => qa.symbol.cmp(&qb.symbol),
                1 => qa.short_name.cmp(&qb.short_name),
                2 => cmp_f64(qa.price, qb.price),
                3 => cmp_f64(qa.change, qb.change),
                4 => cmp_f64(qa.change_percent, qb.change_percent),
                5 => cmp_f64(qa.open, qb.open),
                6 => cmp_f64(qa.high, qb.high),
                7 => cmp_f64(qa.low, qb.low),
                8 => qa.volume.cmp(&qb.volume),
                9 => cmp_f64(qa.price * qa.volume as f64, qb.price * qb.volume as f64),
                _ => Ordering::Equal,
            };
            match direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        }
    }
}

pub fn compare_portfolio_column(
    col: usize,
    a: &Holding,
    b: &Holding,
    quotes: &HashMap<String, StockQuote>,
) -> Ordering {
    let price_a = quotes.get(&a.symbol).map(|q| q.price).unwrap_or(0.0);
    let price_b = quotes.get(&b.symbol).map(|q| q.price).unwrap_or(0.0);
    let name_a = quotes
        .get(&a.symbol)
        .map(|q| q.short_name.as_str())
        .unwrap_or("");
    let name_b = quotes
        .get(&b.symbol)
        .map(|q| q.short_name.as_str())
        .unwrap_or("");
    match col {
        0 => a.symbol.cmp(&b.symbol),
        1 => name_a.cmp(name_b),
        2 => a.lots.cmp(&b.lots),
        3 => cmp_f64(a.avg_price, b.avg_price),
        4 => cmp_f64(price_a, price_b),
        5 => cmp_f64(a.pl_metrics(price_a).0, b.pl_metrics(price_b).0),
        6 => cmp_f64(a.cost_basis(), b.cost_basis()),
        7 => cmp_f64(a.pl_metrics(price_a).2, b.pl_metrics(price_b).2),
        8 => cmp_f64(a.pl_metrics(price_a).3, b.pl_metrics(price_b).3),
        _ => Ordering::Equal,
    }
}

pub fn compare_news_column(col: usize, a: &NewsItem, b: &NewsItem) -> Ordering {
    match col {
        0 => a.published_at.cmp(&b.published_at),
        1 => a.publisher.cmp(&b.publisher),
        2 => a.title.cmp(&b.title),
        _ => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quote(symbol: &str, price: f64, change: f64, change_pct: f64) -> StockQuote {
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

    fn make_holding(symbol: &str, lots: u32, avg_price: f64) -> Holding {
        Holding {
            symbol: symbol.to_string(),
            lots,
            avg_price,
        }
    }

    fn make_news(title: &str, publisher: &str, ts: i64) -> NewsItem {
        NewsItem {
            title: title.to_string(),
            publisher: publisher.to_string(),
            published_at: ts,
            url: None,
            summary: None,
        }
    }

    // --- cmp_f64 ---

    #[test]
    fn test_cmp_f64_less() {
        assert_eq!(cmp_f64(1.0, 2.0), Ordering::Less);
    }

    #[test]
    fn test_cmp_f64_greater() {
        assert_eq!(cmp_f64(3.0, 1.0), Ordering::Greater);
    }

    #[test]
    fn test_cmp_f64_equal() {
        assert_eq!(cmp_f64(1.5, 1.5), Ordering::Equal);
    }

    #[test]
    fn test_cmp_f64_nan_nan() {
        assert_eq!(cmp_f64(f64::NAN, f64::NAN), Ordering::Equal);
    }

    #[test]
    fn test_cmp_f64_nan_value() {
        assert_eq!(cmp_f64(f64::NAN, 1.0), Ordering::Equal);
    }

    #[test]
    fn test_cmp_f64_value_nan() {
        assert_eq!(cmp_f64(1.0, f64::NAN), Ordering::Equal);
    }

    // --- compare_watchlist_column ---

    #[test]
    fn test_watchlist_sort_by_symbol() {
        let qa = make_quote("AAAA", 1000.0, 10.0, 1.0);
        let qb = make_quote("BBBB", 2000.0, 20.0, 2.0);
        let sa = "AAAA".to_string();
        let sb = "BBBB".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(0, &a, &b, SortDirection::Ascending),
            Ordering::Less
        );
    }

    #[test]
    fn test_watchlist_sort_by_price() {
        let qa = make_quote("BBCA", 8000.0, 50.0, 0.6);
        let qb = make_quote("BBRI", 9000.0, -30.0, -0.3);
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(2, &a, &b, SortDirection::Ascending),
            Ordering::Less
        );
    }

    #[test]
    fn test_watchlist_sort_by_change_percent() {
        let qa = make_quote("BBCA", 8000.0, -200.0, -2.5);
        let qb = make_quote("BBRI", 9000.0, 135.0, 1.5);
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(4, &a, &b, SortDirection::Ascending),
            Ordering::Less
        );
    }

    #[test]
    fn test_watchlist_sort_by_volume() {
        let mut qa = make_quote("BBCA", 8000.0, 50.0, 0.6);
        let mut qb = make_quote("BBRI", 9000.0, 30.0, 0.3);
        qa.volume = 500_000;
        qb.volume = 2_000_000;
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(8, &a, &b, SortDirection::Ascending),
            Ordering::Less
        );
    }

    #[test]
    fn test_watchlist_sort_descending_reverses() {
        let qa = make_quote("BBCA", 8000.0, 50.0, 0.6);
        let qb = make_quote("BBRI", 9000.0, 30.0, 0.3);
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(2, &a, &b, SortDirection::Descending),
            Ordering::Greater
        );
    }

    #[test]
    fn test_watchlist_sort_none_quote_sorts_last() {
        let qb = make_quote("BBRI", 9000.0, 30.0, 0.3);
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a: (&String, Option<&StockQuote>) = (&sa, None);
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(2, &a, &b, SortDirection::Ascending),
            Ordering::Greater
        );
    }

    #[test]
    fn test_watchlist_sort_both_none_equal() {
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a: (&String, Option<&StockQuote>) = (&sa, None);
        let b: (&String, Option<&StockQuote>) = (&sb, None);
        assert_eq!(
            compare_watchlist_column(2, &a, &b, SortDirection::Ascending),
            Ordering::Equal
        );
    }

    #[test]
    fn test_watchlist_sort_invalid_column() {
        let qa = make_quote("BBCA", 8000.0, 50.0, 0.6);
        let qb = make_quote("BBRI", 9000.0, 30.0, 0.3);
        let sa = "BBCA".to_string();
        let sb = "BBRI".to_string();
        let a = (&sa, Some(&qa));
        let b = (&sb, Some(&qb));
        assert_eq!(
            compare_watchlist_column(99, &a, &b, SortDirection::Ascending),
            Ordering::Equal
        );
    }

    // --- compare_portfolio_column ---

    #[test]
    fn test_portfolio_sort_by_symbol() {
        let a = make_holding("AAAA", 10, 1000.0);
        let b = make_holding("ZZZZ", 10, 1000.0);
        let quotes = HashMap::new();
        assert_eq!(compare_portfolio_column(0, &a, &b, &quotes), Ordering::Less);
    }

    #[test]
    fn test_portfolio_sort_by_lots() {
        let a = make_holding("BBCA", 5, 8000.0);
        let b = make_holding("BBRI", 20, 5000.0);
        let quotes = HashMap::new();
        assert_eq!(compare_portfolio_column(2, &a, &b, &quotes), Ordering::Less);
    }

    #[test]
    fn test_portfolio_sort_by_avg_price() {
        let a = make_holding("BBCA", 10, 7500.0);
        let b = make_holding("BBRI", 10, 9200.0);
        let quotes = HashMap::new();
        assert_eq!(compare_portfolio_column(3, &a, &b, &quotes), Ordering::Less);
    }

    #[test]
    fn test_portfolio_sort_by_current_price() {
        let a = make_holding("BBCA", 10, 8000.0);
        let b = make_holding("BBRI", 10, 5000.0);
        let mut quotes = HashMap::new();
        quotes.insert("BBCA".to_string(), make_quote("BBCA", 8500.0, 50.0, 0.6));
        quotes.insert("BBRI".to_string(), make_quote("BBRI", 9500.0, 30.0, 0.3));
        assert_eq!(compare_portfolio_column(4, &a, &b, &quotes), Ordering::Less);
    }

    #[test]
    fn test_portfolio_sort_by_pl_percent() {
        // BBCA: 10 lots @ 8000, current 9000 → profit
        // BBRI: 10 lots @ 9000, current 8000 → loss
        let a = make_holding("BBCA", 10, 8000.0);
        let b = make_holding("BBRI", 10, 9000.0);
        let mut quotes = HashMap::new();
        quotes.insert("BBCA".to_string(), make_quote("BBCA", 9000.0, 0.0, 0.0));
        quotes.insert("BBRI".to_string(), make_quote("BBRI", 8000.0, 0.0, 0.0));
        // BBCA P/L% = +12.5%, BBRI P/L% = -11.1%, so BBRI < BBCA
        assert_eq!(
            compare_portfolio_column(8, &a, &b, &quotes),
            Ordering::Greater
        );
    }

    #[test]
    fn test_portfolio_sort_missing_quote_defaults_zero() {
        let a = make_holding("BBCA", 10, 8000.0);
        let b = make_holding("BBRI", 10, 5000.0);
        let quotes = HashMap::new(); // no quotes → both prices 0.0
        // Col 4 (current_price): both 0.0 → Equal
        assert_eq!(
            compare_portfolio_column(4, &a, &b, &quotes),
            Ordering::Equal
        );
    }

    #[test]
    fn test_portfolio_sort_invalid_column() {
        let a = make_holding("BBCA", 10, 8000.0);
        let b = make_holding("BBRI", 20, 5000.0);
        let quotes = HashMap::new();
        assert_eq!(
            compare_portfolio_column(99, &a, &b, &quotes),
            Ordering::Equal
        );
    }

    // --- compare_news_column ---

    #[test]
    fn test_news_sort_by_time() {
        let a = make_news("Earlier", "CNBC", 1000);
        let b = make_news("Later", "CNBC", 2000);
        assert_eq!(compare_news_column(0, &a, &b), Ordering::Less);
    }

    #[test]
    fn test_news_sort_by_publisher() {
        let a = make_news("Title", "AAA News", 1000);
        let b = make_news("Title", "ZZZ News", 1000);
        assert_eq!(compare_news_column(1, &a, &b), Ordering::Less);
    }

    #[test]
    fn test_news_sort_by_title() {
        let a = make_news("Alpha headline", "CNBC", 1000);
        let b = make_news("Zeta headline", "CNBC", 1000);
        assert_eq!(compare_news_column(2, &a, &b), Ordering::Less);
    }

    #[test]
    fn test_news_sort_invalid_column() {
        let a = make_news("Title", "CNBC", 1000);
        let b = make_news("Title", "CNBC", 2000);
        assert_eq!(compare_news_column(99, &a, &b), Ordering::Equal);
    }
}
