mod alerts;
mod export;
mod filter;
mod news;
mod portfolio;
mod sort;
mod watchlist;

use crate::api::{ChartData, NewsClient, NewsItem, StockQuote, YahooClient};
use crate::config::{AlertType, Config};
use crate::ui::{NEWS_SORTABLE_COLUMNS, PORTFOLIO_SORTABLE_COLUMNS, WATCHLIST_SORTABLE_COLUMNS};
use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use tokio::time::Instant;

/// Check if a headline contains a ticker as a whole word, not as a substring.
/// e.g. "DEWA" matches "Saham DEWA Naik" and "Darma (DEWA)" but not "Dewan Pengawas".
pub fn title_contains_ticker(title: &str, ticker: &str) -> bool {
    if ticker.is_empty() {
        return false;
    }
    let title_upper = title.to_uppercase();
    let mut start = 0;
    while let Some(pos) = title_upper[start..].find(ticker) {
        let abs_pos = start + pos;
        let end_pos = abs_pos + ticker.len();

        let before_ok = abs_pos == 0 || !title_upper.as_bytes()[abs_pos - 1].is_ascii_alphabetic();
        let after_ok =
            end_pos >= title_upper.len() || !title_upper.as_bytes()[end_pos].is_ascii_alphabetic();

        if before_ok && after_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Adding,
    WatchlistAdd,
    WatchlistRename,
    StockDetail,
    PortfolioAddSymbol,
    PortfolioAddLots,
    PortfolioAddPrice,
    Help,
    Search,
    ExportMenu,
    PortfolioChart,
    PortfolioEditLots,
    PortfolioEditPrice,
    NewsDetail,
    PortfolioNew,
    PortfolioRename,
    AlertList,
    AlertAddType,
    AlertAddValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Watchlist,
    Portfolio,
    News,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    Csv,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportScope {
    #[default]
    Watchlist,
    Portfolio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&mut self) {
        *self = match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}

pub struct App {
    pub config: Config,
    pub quotes: HashMap<String, StockQuote>,
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub status_message: Option<String>,
    pub loading: bool,
    pub detail_symbol: Option<String>,
    pub detail_chart: Option<ChartData>,
    pub detail_news: Option<Vec<NewsItem>>,
    pub chart_loading: bool,
    pub news_loading: bool,
    pub view_mode: ViewMode,
    pub portfolio_selected: usize,
    pub search_query: String,
    pub search_active: bool,
    pub export_format: ExportFormat,
    pub export_scope: ExportScope,
    pub export_menu_selection: usize,
    pub pending_symbol: Option<String>,
    pub pending_lots: Option<u32>,
    pub pending_edit_symbol: Option<String>,
    pub alert_symbol: Option<String>,
    pub alert_list_selected: usize,
    pub pending_alert_type: AlertType,
    pub watchlist_sort_column: Option<usize>,
    pub watchlist_sort_direction: SortDirection,
    pub portfolio_sort_column: Option<usize>,
    pub portfolio_sort_direction: SortDirection,
    pub news_items: Vec<NewsItem>,
    pub news_selected: usize,
    pub news_last_refresh: Option<Instant>,
    pub rss_loading: bool,
    pub news_sort_column: Option<usize>,
    pub news_sort_direction: SortDirection,
    pub news_detail_scroll: usize,
    pub watchlist_table_state: TableState,
    pub portfolio_table_state: TableState,
    pub news_table_state: TableState,
    pub table_viewport_height: usize,
    news_client: NewsClient,
    client: YahooClient,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self {
            config,
            quotes: HashMap::new(),
            selected_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            loading: false,
            detail_symbol: None,
            detail_chart: None,
            detail_news: None,
            chart_loading: false,
            news_loading: false,
            view_mode: ViewMode::Watchlist,
            portfolio_selected: 0,
            search_query: String::new(),
            search_active: false,
            export_format: ExportFormat::default(),
            export_scope: ExportScope::default(),
            export_menu_selection: 0,
            pending_symbol: None,
            pending_lots: None,
            pending_edit_symbol: None,
            alert_symbol: None,
            alert_list_selected: 0,
            pending_alert_type: AlertType::Above,
            watchlist_sort_column: None,
            watchlist_sort_direction: SortDirection::Ascending,
            portfolio_sort_column: None,
            portfolio_sort_direction: SortDirection::Ascending,
            news_items: Vec::new(),
            news_selected: 0,
            news_last_refresh: None,
            rss_loading: false,
            news_sort_column: None,
            news_sort_direction: SortDirection::Ascending,
            news_detail_scroll: 0,
            watchlist_table_state: TableState::default(),
            portfolio_table_state: TableState::default(),
            news_table_state: TableState::default(),
            table_viewport_height: 0,
            news_client: NewsClient::new(),
            client: YahooClient::new(),
        })
    }

    /// Collect symbols for the current view. Returns `None` for News view.
    /// Always includes `^JKSE` so the IHSG index is available.
    pub fn refresh_symbols(&self) -> Option<Vec<String>> {
        let mut symbols: Vec<String> = match self.view_mode {
            ViewMode::Watchlist => self.config.current_watchlist().symbols.clone(),
            ViewMode::Portfolio => self.config.portfolio_symbols(),
            ViewMode::News => return None,
        };
        if symbols.is_empty() {
            return Some(vec!["^JKSE".to_string()]);
        }
        if !symbols.contains(&"^JKSE".to_string()) {
            symbols.push("^JKSE".to_string());
        }
        Some(symbols)
    }

    /// Collect symbols and set `loading = true`. Returns `None` for News view
    /// or empty watchlists (no network call needed).
    pub fn prepare_refresh(&mut self) -> Option<Vec<String>> {
        let symbols = self.refresh_symbols();
        if symbols.is_some() {
            self.loading = true;
        }
        symbols
    }

    /// Execute the network fetch for the given symbols and clear `loading`.
    pub async fn execute_refresh(&mut self, symbols: &[String]) -> Result<()> {
        match self.client.get_quotes(symbols).await {
            Ok(quotes) => {
                self.quotes = quotes;
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
        self.loading = false;
        Ok(())
    }

    pub fn move_up(&mut self) {
        let vh = self.table_viewport_height;
        match self.view_mode {
            ViewMode::Watchlist => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                let sel = self.selected_index;
                let state = &mut self.watchlist_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
            ViewMode::Portfolio => {
                if self.portfolio_selected > 0 {
                    self.portfolio_selected -= 1;
                }
                let sel = self.portfolio_selected;
                let state = &mut self.portfolio_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
            ViewMode::News => {
                if self.news_selected > 0 {
                    self.news_selected -= 1;
                }
                let sel = self.news_selected;
                let state = &mut self.news_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        let vh = self.table_viewport_height;
        match self.view_mode {
            ViewMode::Watchlist => {
                let len = self.get_filtered_watchlist().len();
                if len > 0 && self.selected_index < len - 1 {
                    self.selected_index += 1;
                }
                let sel = self.selected_index;
                let state = &mut self.watchlist_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
            ViewMode::Portfolio => {
                let len = self.get_filtered_portfolio().len();
                if len > 0 && self.portfolio_selected < len - 1 {
                    self.portfolio_selected += 1;
                }
                let sel = self.portfolio_selected;
                let state = &mut self.portfolio_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
            ViewMode::News => {
                let len = self.get_filtered_news().len();
                if len > 0 && self.news_selected < len - 1 {
                    self.news_selected += 1;
                }
                let sel = self.news_selected;
                let state = &mut self.news_table_state;
                state.select(Some(sel));
                let off = state.offset();
                if sel < off {
                    *state.offset_mut() = sel;
                } else if vh > 0 && sel >= off + vh {
                    *state.offset_mut() = sel + 1 - vh;
                }
            }
        }
    }

    pub fn cycle_sort_column(&mut self) {
        let num_columns = match self.view_mode {
            ViewMode::Watchlist => WATCHLIST_SORTABLE_COLUMNS,
            ViewMode::Portfolio => PORTFOLIO_SORTABLE_COLUMNS,
            ViewMode::News => NEWS_SORTABLE_COLUMNS,
        };
        let (col, selected) = match self.view_mode {
            ViewMode::Watchlist => (&mut self.watchlist_sort_column, &mut self.selected_index),
            ViewMode::Portfolio => (
                &mut self.portfolio_sort_column,
                &mut self.portfolio_selected,
            ),
            ViewMode::News => (&mut self.news_sort_column, &mut self.news_selected),
        };
        *col = match *col {
            None => Some(0),
            Some(i) if i + 1 >= num_columns => None,
            Some(i) => Some(i + 1),
        };
        *selected = 0;
        self.reset_current_table_offset();
    }

    pub fn toggle_sort_direction(&mut self) {
        let (dir, selected) = match self.view_mode {
            ViewMode::Watchlist => (&mut self.watchlist_sort_direction, &mut self.selected_index),
            ViewMode::Portfolio => (
                &mut self.portfolio_sort_direction,
                &mut self.portfolio_selected,
            ),
            ViewMode::News => (&mut self.news_sort_direction, &mut self.news_selected),
        };
        dir.toggle();
        *selected = 0;
        self.reset_current_table_offset();
    }

    fn reset_current_table_offset(&mut self) {
        let state = match self.view_mode {
            ViewMode::Watchlist => &mut self.watchlist_table_state,
            ViewMode::Portfolio => &mut self.portfolio_table_state,
            ViewMode::News => &mut self.news_table_state,
        };
        *state.offset_mut() = 0;
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Watchlist => ViewMode::Portfolio,
            ViewMode::Portfolio => ViewMode::News,
            ViewMode::News => ViewMode::Watchlist,
        };
        if self.view_mode != ViewMode::News {
            self.quotes.clear();
        }
        self.clear_filter();
    }

    pub fn close_stock_detail(&mut self) {
        self.detail_symbol = None;
        self.detail_chart = None;
        self.detail_news = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn get_detail_quote(&self) -> Option<&StockQuote> {
        self.detail_symbol.as_ref().and_then(|s| self.quotes.get(s))
    }

    pub fn get_ihsg_quote(&self) -> Option<&StockQuote> {
        self.quotes.get("IHSG")
    }

    async fn open_detail(&mut self, symbol: &str) {
        self.detail_symbol = Some(symbol.to_string());
        self.detail_chart = None;
        self.detail_news = None;
        self.chart_loading = true;
        self.news_loading = true;
        self.input_mode = InputMode::StockDetail;

        // Ensure RSS news is loaded before filtering
        if self.news_items.is_empty() {
            let urls = self.prepare_news_refresh();
            self.execute_news_refresh(&urls).await;
        }

        // Filter RSS headlines matching this stock's ticker or company name
        self.detail_news = Some(self.get_detail_news(symbol));
        self.news_loading = false;

        if let Ok(chart) = self.client.get_chart(symbol).await {
            self.detail_chart = Some(chart);
        }
        self.chart_loading = false;
    }

    /// Filter RSS news items relevant to a specific stock by ticker match
    fn get_detail_news(&self, symbol: &str) -> Vec<NewsItem> {
        let sym_upper = symbol.to_uppercase();
        self.news_items
            .iter()
            .filter(|item| title_contains_ticker(&item.title, &sym_upper))
            .take(8)
            .cloned()
            .collect()
    }

    pub fn show_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn make_quote(symbol: &str, price: f64, change: f64, change_pct: f64) -> StockQuote {
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

    pub(crate) fn make_news_item(title: &str, publisher: &str, ts: i64) -> NewsItem {
        NewsItem {
            title: title.to_string(),
            publisher: publisher.to_string(),
            published_at: ts,
            url: None,
            summary: None,
        }
    }

    pub(crate) fn test_app() -> App {
        App {
            config: Config::test_config(),
            quotes: HashMap::new(),
            selected_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            loading: false,
            detail_symbol: None,
            detail_chart: None,
            detail_news: None,
            chart_loading: false,
            news_loading: false,
            view_mode: ViewMode::Watchlist,
            portfolio_selected: 0,
            search_query: String::new(),
            search_active: false,
            export_format: ExportFormat::default(),
            export_scope: ExportScope::default(),
            export_menu_selection: 0,
            pending_symbol: None,
            pending_lots: None,
            pending_edit_symbol: None,
            alert_symbol: None,
            alert_list_selected: 0,
            pending_alert_type: AlertType::Above,
            watchlist_sort_column: None,
            watchlist_sort_direction: SortDirection::Ascending,
            portfolio_sort_column: None,
            portfolio_sort_direction: SortDirection::Ascending,
            news_items: Vec::new(),
            news_selected: 0,
            news_last_refresh: None,
            rss_loading: false,
            news_sort_column: None,
            news_sort_direction: SortDirection::Ascending,
            news_detail_scroll: 0,
            watchlist_table_state: TableState::default(),
            portfolio_table_state: TableState::default(),
            news_table_state: TableState::default(),
            table_viewport_height: 20,
            news_client: NewsClient::new(),
            client: YahooClient::new(),
        }
    }

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
        // Empty ticker should not match anything
        assert!(!title_contains_ticker("Saham Naik", ""));
    }

    // --- move_up / move_down ---

    #[test]
    fn test_move_down_increments() {
        let mut app = test_app();
        // Default watchlist has 4 symbols: BBCA, BBRI, TLKM, ASII
        assert_eq!(app.selected_index, 0);
        app.move_down();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_move_down_clamps_at_bottom() {
        let mut app = test_app();
        app.selected_index = 3; // last item (4 symbols, index 0-3)
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
        // Add holdings so there's something to navigate
        app.config.portfolios[0]
            .holdings
            .push(crate::config::Holding {
                symbol: "BBCA".to_string(),
                lots: 10,
                avg_price: 8000.0,
            });
        app.config.portfolios[0]
            .holdings
            .push(crate::config::Holding {
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
        app.watchlist_sort_column = Some(9); // last watchlist column (10 total, 0-9)
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
        app.portfolio_sort_column = Some(8); // last portfolio column (9 total, 0-8)
        app.cycle_sort_column();
        assert_eq!(app.portfolio_sort_column, None);
    }

    #[test]
    fn test_cycle_sort_column_news_view() {
        let mut app = test_app();
        app.view_mode = ViewMode::News;
        app.news_sort_column = Some(2); // last news column (3 total, 0-2)
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
        // Watchlist → Portfolio: clears quotes
        app.toggle_view();
        assert!(app.quotes.is_empty());

        app.quotes
            .insert("BBRI".to_string(), make_quote("BBRI", 5000.0, 30.0, 0.6));
        // Portfolio → News: does NOT clear quotes
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
        assert_eq!(app.search_query, "BBCA"); // uppercased
        assert_eq!(app.selected_index, 0); // reset
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
}
