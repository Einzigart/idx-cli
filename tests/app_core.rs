mod common;

use common::{make_news_item, make_quote, test_app};
use idx_cli::api::ChartData;
use idx_cli::app::{InputMode, SortDirection, ViewMode, title_contains_ticker};
use idx_cli::config::Holding;

// --- title_contains_ticker ---

#[test]
fn test_ticker_exact_word() {
    assert!(title_contains_ticker("Saham DEWA Naik", "DEWA"));
}

#[test]
fn test_ticker_at_start() {
    assert!(title_contains_ticker("BBCA mencatat kenaikan", "BBCA"));
}

#[test]
fn test_ticker_at_end() {
    assert!(title_contains_ticker("Kenaikan saham BBCA", "BBCA"));
}

#[test]
fn test_ticker_in_parens() {
    assert!(title_contains_ticker("Darma (DEWA) laba", "DEWA"));
}

#[test]
fn test_ticker_substring_no_match() {
    assert!(!title_contains_ticker("Dewan Pengawas", "DEWA"));
}

#[test]
fn test_ticker_case_insensitive() {
    assert!(title_contains_ticker("saham dewa naik", "DEWA"));
}

#[test]
fn test_ticker_empty_title() {
    assert!(!title_contains_ticker("", "BBCA"));
}

#[test]
fn test_ticker_empty_ticker() {
    assert!(!title_contains_ticker("Saham Naik", ""));
}

// --- move_up / move_down ---

#[test]
fn test_move_down_increments() {
    let mut app = test_app();
    assert_eq!(app.selected_index, 0);
    app.move_down();
    assert_eq!(app.selected_index, 1);
}

#[test]
fn test_move_down_clamps_at_bottom() {
    let mut app = test_app();
    app.selected_index = 3;
    app.move_down();
    assert_eq!(app.selected_index, 3);
}

#[test]
fn test_move_up_decrements() {
    let mut app = test_app();
    app.selected_index = 2;
    app.move_up();
    assert_eq!(app.selected_index, 1);
}

#[test]
fn test_move_up_clamps_at_zero() {
    let mut app = test_app();
    app.selected_index = 0;
    app.move_up();
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_move_down_empty_list() {
    let mut app = test_app();
    app.config.watchlists[0].symbols.clear();
    app.move_down();
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_move_down_portfolio_view() {
    let mut app = test_app();
    app.view_mode = ViewMode::Portfolio;
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "BBCA".to_string(),
        lots: 10,
        avg_price: 8000.0,
    });
    app.config.portfolios[0].holdings.push(Holding {
        symbol: "BBRI".to_string(),
        lots: 5,
        avg_price: 5000.0,
    });
    assert_eq!(app.portfolio_selected, 0);
    app.move_down();
    assert_eq!(app.portfolio_selected, 1);
}

#[test]
fn test_move_down_news_view() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_items.push(make_news_item("News 1", "CNBC", 1000));
    app.news_items.push(make_news_item("News 2", "CNBC", 2000));
    assert_eq!(app.news_selected, 0);
    app.move_down();
    assert_eq!(app.news_selected, 1);
}

// --- cycle_sort_column ---

#[test]
fn test_cycle_sort_column_from_none() {
    let mut app = test_app();
    assert_eq!(app.watchlist_sort_column, None);
    app.cycle_sort_column();
    assert_eq!(app.watchlist_sort_column, Some(0));
}

#[test]
fn test_cycle_sort_column_increment() {
    let mut app = test_app();
    app.watchlist_sort_column = Some(0);
    app.cycle_sort_column();
    assert_eq!(app.watchlist_sort_column, Some(1));
}

#[test]
fn test_cycle_sort_column_wrap_to_none() {
    let mut app = test_app();
    app.watchlist_sort_column = Some(9);
    app.cycle_sort_column();
    assert_eq!(app.watchlist_sort_column, None);
}

#[test]
fn test_cycle_sort_column_resets_selected() {
    let mut app = test_app();
    app.selected_index = 3;
    app.cycle_sort_column();
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_cycle_sort_column_portfolio_view() {
    let mut app = test_app();
    app.view_mode = ViewMode::Portfolio;
    app.portfolio_sort_column = Some(8);
    app.cycle_sort_column();
    assert_eq!(app.portfolio_sort_column, None);
}

#[test]
fn test_cycle_sort_column_news_view() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_sort_column = Some(2);
    app.cycle_sort_column();
    assert_eq!(app.news_sort_column, None);
}

// --- toggle_sort_direction ---

#[test]
fn test_toggle_sort_ascending_to_descending() {
    let mut app = test_app();
    assert_eq!(app.watchlist_sort_direction, SortDirection::Ascending);
    app.toggle_sort_direction();
    assert_eq!(app.watchlist_sort_direction, SortDirection::Descending);
}

#[test]
fn test_toggle_sort_descending_to_ascending() {
    let mut app = test_app();
    app.watchlist_sort_direction = SortDirection::Descending;
    app.toggle_sort_direction();
    assert_eq!(app.watchlist_sort_direction, SortDirection::Ascending);
}

#[test]
fn test_toggle_sort_resets_selected() {
    let mut app = test_app();
    app.selected_index = 3;
    app.toggle_sort_direction();
    assert_eq!(app.selected_index, 0);
}

// --- toggle_view ---

#[test]
fn test_toggle_view_cycle() {
    let mut app = test_app();
    assert_eq!(app.view_mode, ViewMode::Watchlist);
    app.toggle_view();
    assert_eq!(app.view_mode, ViewMode::Portfolio);
    app.toggle_view();
    assert_eq!(app.view_mode, ViewMode::News);
    app.toggle_view();
    assert_eq!(app.view_mode, ViewMode::Watchlist);
}

#[test]
fn test_toggle_view_clears_filter() {
    let mut app = test_app();
    app.search_active = true;
    app.search_query = "BBCA".to_string();
    app.selected_index = 2;
    app.toggle_view();
    assert!(!app.search_active);
    assert!(app.search_query.is_empty());
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_toggle_view_clears_quotes_except_to_news() {
    let mut app = test_app();
    app.quotes
        .insert("BBCA".to_string(), make_quote("BBCA", 8000.0, 50.0, 0.6));
    // Watchlist -> Portfolio: clears quotes
    app.toggle_view();
    assert!(app.quotes.is_empty());

    app.quotes
        .insert("BBRI".to_string(), make_quote("BBRI", 5000.0, 30.0, 0.6));
    // Portfolio -> News: does NOT clear quotes
    app.toggle_view();
    assert!(!app.quotes.is_empty());
}

// --- cancel_input / show_help / close_help / close_stock_detail ---

#[test]
fn test_cancel_input_resets_mode() {
    let mut app = test_app();
    app.input_mode = InputMode::Adding;
    app.input_buffer = "BBCA".to_string();
    app.cancel_input();
    assert_eq!(app.input_mode, InputMode::Normal);
    assert!(app.input_buffer.is_empty());
}

#[test]
fn test_show_help_sets_mode() {
    let mut app = test_app();
    app.show_help();
    assert_eq!(app.input_mode, InputMode::Help);
}

#[test]
fn test_close_help_resets_mode() {
    let mut app = test_app();
    app.input_mode = InputMode::Help;
    app.close_help();
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_close_stock_detail() {
    let mut app = test_app();
    app.input_mode = InputMode::StockDetail;
    app.detail_symbol = Some("BBCA".to_string());
    app.detail_chart = Some(ChartData {
        closes: vec![100.0],
        high: 110.0,
        low: 90.0,
    });
    app.detail_news = Some(vec![make_news_item("Test", "CNBC", 1000)]);
    app.close_stock_detail();
    assert_eq!(app.input_mode, InputMode::Normal);
    assert!(app.detail_symbol.is_none());
    assert!(app.detail_chart.is_none());
    assert!(app.detail_news.is_none());
}

// --- search flow ---

#[test]
fn test_start_search() {
    let mut app = test_app();
    app.input_buffer = "leftover".to_string();
    app.start_search();
    assert_eq!(app.input_mode, InputMode::Search);
    assert!(app.input_buffer.is_empty());
}

#[test]
fn test_confirm_search_activates_filter() {
    let mut app = test_app();
    app.input_mode = InputMode::Search;
    app.input_buffer = "bbca".to_string();
    app.selected_index = 2;
    app.confirm_search();
    assert!(app.search_active);
    assert_eq!(app.search_query, "BBCA");
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_confirm_search_empty_clears_filter() {
    let mut app = test_app();
    app.search_active = true;
    app.search_query = "OLD".to_string();
    app.input_buffer.clear();
    app.confirm_search();
    assert!(!app.search_active);
    assert!(app.search_query.is_empty());
}

#[test]
fn test_cancel_search() {
    let mut app = test_app();
    app.search_active = true;
    app.search_query = "BBCA".to_string();
    app.input_buffer = "partial".to_string();
    app.cancel_search();
    assert!(!app.search_active);
    assert!(app.search_query.is_empty());
    assert!(app.input_buffer.is_empty());
    assert_eq!(app.input_mode, InputMode::Normal);
}
