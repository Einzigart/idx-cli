mod common;

use common::{make_holding, make_news_item, make_quote};
use idx_cli::api::StockQuote;
use idx_cli::app::SortDirection;
use idx_cli::app::sort::*;
use std::cmp::Ordering;
use std::collections::HashMap;

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
    let a = make_holding("BBCA", 10, 8000.0);
    let b = make_holding("BBRI", 10, 9000.0);
    let mut quotes = HashMap::new();
    quotes.insert("BBCA".to_string(), make_quote("BBCA", 9000.0, 0.0, 0.0));
    quotes.insert("BBRI".to_string(), make_quote("BBRI", 8000.0, 0.0, 0.0));
    assert_eq!(
        compare_portfolio_column(8, &a, &b, &quotes),
        Ordering::Greater
    );
}

#[test]
fn test_portfolio_sort_missing_quote_defaults_zero() {
    let a = make_holding("BBCA", 10, 8000.0);
    let b = make_holding("BBRI", 10, 5000.0);
    let quotes = HashMap::new();
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
    let a = make_news_item("Earlier", "CNBC", 1000);
    let b = make_news_item("Later", "CNBC", 2000);
    assert_eq!(compare_news_column(0, &a, &b), Ordering::Less);
}

#[test]
fn test_news_sort_by_publisher() {
    let a = make_news_item("Title", "AAA News", 1000);
    let b = make_news_item("Title", "ZZZ News", 1000);
    assert_eq!(compare_news_column(1, &a, &b), Ordering::Less);
}

#[test]
fn test_news_sort_by_title() {
    let a = make_news_item("Alpha headline", "CNBC", 1000);
    let b = make_news_item("Zeta headline", "CNBC", 1000);
    assert_eq!(compare_news_column(2, &a, &b), Ordering::Less);
}

#[test]
fn test_news_sort_invalid_column() {
    let a = make_news_item("Title", "CNBC", 1000);
    let b = make_news_item("Title", "CNBC", 2000);
    assert_eq!(compare_news_column(99, &a, &b), Ordering::Equal);
}
