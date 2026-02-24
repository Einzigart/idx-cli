mod common;

use common::{make_news_item, make_quote, test_app};
use idx_cli::config::Holding;

// --- get_filtered_watchlist ---

#[test]
fn test_filtered_watchlist_no_filter() {
    let app = test_app();
    let filtered = app.get_filtered_watchlist();
    assert_eq!(filtered.len(), 4);
    assert_eq!(*filtered[0].0, "BBCA");
    assert_eq!(*filtered[3].0, "ASII");
}

#[test]
fn test_filtered_watchlist_with_search() {
    let mut app = test_app();
    app.search_active = true;
    app.search_query = "BB".to_string();
    let filtered = app.get_filtered_watchlist();
    assert_eq!(filtered.len(), 2);
    let symbols: Vec<&str> = filtered.iter().map(|(s, _)| s.as_str()).collect();
    assert!(symbols.contains(&"BBCA"));
    assert!(symbols.contains(&"BBRI"));
}

#[test]
fn test_filtered_watchlist_with_sort() {
    let mut app = test_app();
    app.quotes
        .insert("BBCA".to_string(), make_quote("BBCA", 9000.0, 50.0, 0.6));
    app.quotes
        .insert("BBRI".to_string(), make_quote("BBRI", 5000.0, 30.0, 0.6));
    app.quotes
        .insert("TLKM".to_string(), make_quote("TLKM", 3000.0, 10.0, 0.3));
    app.quotes
        .insert("ASII".to_string(), make_quote("ASII", 7000.0, -20.0, -0.3));
    app.watchlist_sort_column = Some(2); // sort by price
    let filtered = app.get_filtered_watchlist();
    assert_eq!(*filtered[0].0, "TLKM");
    assert_eq!(*filtered[3].0, "BBCA");
}

#[test]
fn test_filtered_watchlist_empty() {
    let mut app = test_app();
    app.config.watchlists[0].symbols.clear();
    let filtered = app.get_filtered_watchlist();
    assert!(filtered.is_empty());
}

// --- get_filtered_portfolio ---

#[test]
fn test_filtered_portfolio_no_filter() {
    let mut app = test_app();
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "BBCA".to_string(),
        lots: 10,
        avg_price: 8000.0,
    });
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "TLKM".to_string(),
        lots: 20,
        avg_price: 3000.0,
    });
    let filtered = app.get_filtered_portfolio();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_filtered_portfolio_with_search() {
    let mut app = test_app();
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "BBCA".to_string(),
        lots: 10,
        avg_price: 8000.0,
    });
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "TLKM".to_string(),
        lots: 20,
        avg_price: 3000.0,
    });
    app.search_active = true;
    app.search_query = "BB".to_string();
    let filtered = app.get_filtered_portfolio();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.symbol, "BBCA");
}

// --- get_filtered_news ---

#[test]
fn test_filtered_news_search_by_title() {
    let mut app = test_app();
    app.news_items
        .push(make_news_item("BBCA naik tajam", "CNBC", 1000));
    app.news_items
        .push(make_news_item("IHSG melemah", "Tempo", 2000));
    app.search_active = true;
    app.search_query = "BBCA".to_string();
    let filtered = app.get_filtered_news();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "BBCA naik tajam");
}

#[test]
fn test_filtered_news_search_by_publisher() {
    let mut app = test_app();
    app.news_items
        .push(make_news_item("Saham naik", "CNBC Indonesia", 1000));
    app.news_items
        .push(make_news_item("Saham turun", "Tempo", 2000));
    app.search_active = true;
    app.search_query = "CNBC".to_string();
    let filtered = app.get_filtered_news();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].publisher, "CNBC Indonesia");
}

// --- selected_*_symbol ---

#[test]
fn test_selected_watchlist_symbol() {
    let mut app = test_app();
    app.selected_index = 1;
    assert_eq!(app.selected_watchlist_symbol(), Some("BBRI".to_string()));
}

#[test]
fn test_selected_watchlist_symbol_empty() {
    let mut app = test_app();
    app.config.watchlists[0].symbols.clear();
    assert_eq!(app.selected_watchlist_symbol(), None);
}

#[test]
fn test_selected_portfolio_symbol() {
    let mut app = test_app();
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "BBCA".to_string(),
        lots: 10,
        avg_price: 8000.0,
    });
    app.portfolio_selected = 0;
    assert_eq!(app.selected_portfolio_symbol(), Some("BBCA".to_string()));
}
