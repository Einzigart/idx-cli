mod export;
mod filter;
mod news;
mod portfolio;
mod sort;
mod watchlist;

use crate::api::{ChartData, NewsClient, NewsItem, StockQuote, YahooClient};
use crate::config::Config;
use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;
use tokio::time::Instant;

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
    pub last_updated: Option<String>,
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
            last_updated: None,
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
            news_client: NewsClient::new(),
            client: YahooClient::new(),
        })
    }

    pub async fn refresh_quotes(&mut self) -> Result<()> {
        self.loading = true;
        let symbols: Vec<String> = match self.view_mode {
            ViewMode::Watchlist => self.config.current_watchlist().symbols.clone(),
            ViewMode::Portfolio => self.config.portfolio_symbols(),
            ViewMode::News => {
                self.loading = false;
                return Ok(());
            }
        };
        match self.client.get_quotes(&symbols).await {
            Ok(quotes) => {
                self.quotes = quotes;
                self.last_updated = Some(Local::now().format("%H:%M:%S").to_string());
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
        match self.view_mode {
            ViewMode::Watchlist => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            ViewMode::Portfolio => {
                if self.portfolio_selected > 0 {
                    self.portfolio_selected -= 1;
                }
            }
            ViewMode::News => {
                if self.news_selected > 0 {
                    self.news_selected -= 1;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.view_mode {
            ViewMode::Watchlist => {
                let len = self.get_filtered_watchlist().len();
                if len > 0 && self.selected_index < len - 1 {
                    self.selected_index += 1;
                }
            }
            ViewMode::Portfolio => {
                let len = self.get_filtered_portfolio().len();
                if len > 0 && self.portfolio_selected < len - 1 {
                    self.portfolio_selected += 1;
                }
            }
            ViewMode::News => {
                let len = self.get_filtered_news().len();
                if len > 0 && self.news_selected < len - 1 {
                    self.news_selected += 1;
                }
            }
        }
    }

    pub fn cycle_sort_column(&mut self) {
        let num_columns = match self.view_mode {
            ViewMode::Watchlist => 10,
            ViewMode::Portfolio => 9,
            ViewMode::News => 3,
        };
        let (col, selected) = match self.view_mode {
            ViewMode::Watchlist => (&mut self.watchlist_sort_column, &mut self.selected_index),
            ViewMode::Portfolio => (&mut self.portfolio_sort_column, &mut self.portfolio_selected),
            ViewMode::News => (&mut self.news_sort_column, &mut self.news_selected),
        };
        *col = match *col {
            None => Some(0),
            Some(i) if i + 1 >= num_columns => None,
            Some(i) => Some(i + 1),
        };
        *selected = 0;
    }

    pub fn toggle_sort_direction(&mut self) {
        let (dir, selected) = match self.view_mode {
            ViewMode::Watchlist => (&mut self.watchlist_sort_direction, &mut self.selected_index),
            ViewMode::Portfolio => (&mut self.portfolio_sort_direction, &mut self.portfolio_selected),
            ViewMode::News => (&mut self.news_sort_direction, &mut self.news_selected),
        };
        dir.toggle();
        *selected = 0;
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

    async fn open_detail(&mut self, symbol: &str) {
        self.detail_symbol = Some(symbol.to_string());
        self.detail_chart = None;
        self.detail_news = None;
        self.chart_loading = true;
        self.news_loading = true;
        self.input_mode = InputMode::StockDetail;
        if let Ok(chart) = self.client.get_chart(symbol).await {
            self.detail_chart = Some(chart);
        }
        self.chart_loading = false;
        if let Ok(news) = self.client.get_news(symbol).await {
            self.detail_news = Some(news);
        }
        self.news_loading = false;
    }

    pub fn show_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }
}
