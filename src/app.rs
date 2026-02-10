use crate::api::{ChartData, StockQuote, YahooClient};
use crate::config::Config;
use anyhow::Result;
use chrono::Local;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Adding,
    WatchlistAdd,
    WatchlistRename,
    StockDetail,
    PortfolioAddSymbol,   // Step 1: Enter symbol
    PortfolioAddLots,     // Step 2: Enter lots
    PortfolioAddPrice,    // Step 3: Enter avg price
    Help,              // Help modal with keybindings
    Search,            // Search/filter symbols
    ExportMenu,        // Export menu for CSV/JSON
    PortfolioChart,    // Portfolio allocation chart
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Watchlist,
    Portfolio,
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
    pub chart_loading: bool,
    pub view_mode: ViewMode,
    pub portfolio_selected: usize,
    pub search_query: String,
    pub search_active: bool,
    pub export_format: ExportFormat,
    pub export_scope: ExportScope,
    pub export_menu_selection: usize, // 0: Format, 1: Scope, 2: Export button
    // Portfolio add workflow state
    pub pending_symbol: Option<String>,
    pub pending_lots: Option<u32>,
    // Sort state
    pub watchlist_sort_column: Option<usize>,
    pub watchlist_sort_direction: SortDirection,
    pub portfolio_sort_column: Option<usize>,
    pub portfolio_sort_direction: SortDirection,
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
            chart_loading: false,
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
            client: YahooClient::new(),
        })
    }

    pub async fn refresh_quotes(&mut self) -> Result<()> {
        self.loading = true;
        let symbols: Vec<String> = match self.view_mode {
            ViewMode::Watchlist => self.config.current_watchlist().symbols.clone(),
            ViewMode::Portfolio => self.config.portfolio_symbols(),
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
        }
    }

    pub fn move_down(&mut self) {
        match self.view_mode {
            ViewMode::Watchlist => {
                let symbols = &self.config.current_watchlist().symbols;
                if !symbols.is_empty() && self.selected_index < symbols.len() - 1 {
                    self.selected_index += 1;
                }
            }
            ViewMode::Portfolio => {
                let len = self.config.portfolio.len();
                if len > 0 && self.portfolio_selected < len - 1 {
                    self.portfolio_selected += 1;
                }
            }
        }
    }

    pub fn next_watchlist(&mut self) {
        self.config.next_watchlist();
        self.selected_index = 0;
        self.quotes.clear();
        self.watchlist_sort_column = None;
    }

    pub fn prev_watchlist(&mut self) {
        self.config.prev_watchlist();
        self.selected_index = 0;
        self.quotes.clear();
        self.watchlist_sort_column = None;
    }

    // Sort methods
    pub fn cycle_sort_column(&mut self) {
        let num_columns = match self.view_mode {
            ViewMode::Watchlist => 10, // WATCHLIST_COLUMNS count
            ViewMode::Portfolio => 9,  // PORTFOLIO_COLUMNS count
        };
        let (col, selected) = match self.view_mode {
            ViewMode::Watchlist => (&mut self.watchlist_sort_column, &mut self.selected_index),
            ViewMode::Portfolio => (&mut self.portfolio_sort_column, &mut self.portfolio_selected),
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
        };
        dir.toggle();
        *selected = 0;
    }

    pub fn start_adding(&mut self) {
        self.input_mode = InputMode::Adding;
        self.input_buffer.clear();
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn confirm_add(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let symbol = self.input_buffer.trim().to_uppercase();
            self.config.add_stock(&symbol);
            self.config.save()?;
            self.status_message = Some(format!("Added {}", symbol));
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    pub fn remove_selected(&mut self) -> Result<()> {
        let symbols = &self.config.current_watchlist().symbols;
        if !symbols.is_empty() {
            let symbol = symbols[self.selected_index].clone();
            self.config.remove_stock(&symbol);
            self.config.save()?;
            self.quotes.remove(&symbol);
            self.status_message = Some(format!("Removed {}", symbol));

            // Adjust selected index
            let new_len = self.config.current_watchlist().symbols.len();
            if self.selected_index >= new_len && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
        Ok(())
    }

    pub fn get_sorted_watchlist(&self) -> Vec<(&String, Option<&StockQuote>)> {
        self.config
            .current_watchlist()
            .symbols
            .iter()
            .map(|symbol| (symbol, self.quotes.get(symbol)))
            .collect()
    }

    pub fn watchlist_indicator(&self) -> String {
        format!(
            "{} ({}/{})",
            self.config.current_watchlist().name,
            self.config.active_watchlist + 1,
            self.config.watchlists.len()
        )
    }

    pub fn start_watchlist_add(&mut self) {
        self.input_mode = InputMode::WatchlistAdd;
        self.input_buffer.clear();
    }

    pub fn start_watchlist_rename(&mut self) {
        self.input_mode = InputMode::WatchlistRename;
        self.input_buffer = self.config.current_watchlist().name.clone();
    }

    pub fn confirm_watchlist_add(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let name = self.input_buffer.trim().to_string();
            self.config.add_watchlist(&name);
            self.config.save()?;
            self.quotes.clear();
            self.selected_index = 0;
            self.status_message = Some(format!("Created watchlist '{}'", name));
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    pub fn confirm_watchlist_rename(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let new_name = self.input_buffer.trim().to_string();
            let old_name = self.config.current_watchlist().name.clone();
            self.config.rename_watchlist(&new_name);
            self.config.save()?;
            self.status_message = Some(format!("Renamed '{}' to '{}'", old_name, new_name));
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    pub fn remove_current_watchlist(&mut self) -> Result<()> {
        if self.config.watchlists.len() > 1 {
            let name = self.config.current_watchlist().name.clone();
            self.config.remove_watchlist();
            self.config.save()?;
            self.quotes.clear();
            self.selected_index = 0;
            self.status_message = Some(format!("Removed watchlist '{}'", name));
        } else {
            self.status_message = Some("Cannot remove the last watchlist".to_string());
        }
        Ok(())
    }

    pub async fn show_stock_detail(&mut self) {
        let symbols = &self.config.current_watchlist().symbols;
        if !symbols.is_empty() && self.selected_index < symbols.len() {
            let symbol = symbols[self.selected_index].clone();
            self.open_detail(&symbol).await;
        }
    }

    pub fn close_stock_detail(&mut self) {
        self.detail_symbol = None;
        self.detail_chart = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn get_detail_quote(&self) -> Option<&StockQuote> {
        self.detail_symbol.as_ref().and_then(|s| self.quotes.get(s))
    }

    // Portfolio methods
    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Watchlist => ViewMode::Portfolio,
            ViewMode::Portfolio => ViewMode::Watchlist,
        };
        self.quotes.clear();
        self.clear_filter();
    }

    pub fn start_portfolio_add(&mut self) {
        self.input_mode = InputMode::PortfolioAddSymbol;
        self.input_buffer.clear();
        self.pending_symbol = None;
        self.pending_lots = None;
    }

    pub fn confirm_portfolio_symbol(&mut self) {
        let symbol = self.input_buffer.trim().to_uppercase();
        if symbol.is_empty() {
            self.status_message = Some("Symbol cannot be empty".to_string());
            self.input_mode = InputMode::Normal;
        } else {
            self.pending_symbol = Some(symbol);
            self.input_mode = InputMode::PortfolioAddLots;
        }
        self.input_buffer.clear();
    }

    pub fn confirm_portfolio_lots(&mut self) {
        if let Ok(lots) = self.input_buffer.trim().parse::<u32>() {
            if lots > 0 {
                self.pending_lots = Some(lots);
                self.input_mode = InputMode::PortfolioAddPrice;
                self.input_buffer.clear();
            } else {
                self.status_message = Some("Lots must be greater than 0".to_string());
                self.input_buffer.clear();
            }
        } else {
            self.status_message = Some("Invalid number for lots".to_string());
            self.input_buffer.clear();
        }
    }

    pub fn confirm_portfolio_price(&mut self) -> Result<()> {
        if let Ok(avg_price) = self.input_buffer.trim().parse::<f64>() {
            if avg_price > 0.0 {
                match (&self.pending_symbol, self.pending_lots) {
                    (Some(symbol), Some(lots)) => {
                        self.config.add_holding(symbol, lots, avg_price);
                        self.config.save()?;
                        self.status_message = Some(format!("Added {} lots of {} @ {}", lots, symbol, avg_price));
                    }
                    _ => {
                        self.status_message = Some("Missing symbol or lots data".to_string());
                    }
                }
            } else {
                self.status_message = Some("Price must be greater than 0".to_string());
            }
        } else {
            self.status_message = Some("Invalid number for price".to_string());
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.pending_symbol = None;
        self.pending_lots = None;
        Ok(())
    }

    pub fn cancel_portfolio_add(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.pending_symbol = None;
        self.pending_lots = None;
    }

    pub fn remove_selected_holding(&mut self) -> Result<()> {
        if !self.config.portfolio.is_empty() {
            let symbol = self.config.portfolio[self.portfolio_selected].symbol.clone();
            self.config.remove_holding(&symbol);
            self.config.save()?;
            self.quotes.remove(&symbol);
            self.status_message = Some(format!("Removed {}", symbol));

            let new_len = self.config.portfolio.len();
            if self.portfolio_selected >= new_len && self.portfolio_selected > 0 {
                self.portfolio_selected -= 1;
            }
        }
        Ok(())
    }

    pub async fn show_portfolio_detail(&mut self) {
        if !self.config.portfolio.is_empty() && self.portfolio_selected < self.config.portfolio.len() {
            let symbol = self.config.portfolio[self.portfolio_selected].symbol.clone();
            self.open_detail(&symbol).await;
        }
    }

    async fn open_detail(&mut self, symbol: &str) {
        self.detail_symbol = Some(symbol.to_string());
        self.detail_chart = None;
        self.chart_loading = true;
        self.input_mode = InputMode::StockDetail;

        if let Ok(chart) = self.client.get_chart(symbol).await {
            self.detail_chart = Some(chart);
        }
        self.chart_loading = false;
    }

    // Help modal methods
    pub fn show_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    // Portfolio chart methods
    pub fn show_portfolio_chart(&mut self) {
        if !self.config.portfolio.is_empty() {
            self.input_mode = InputMode::PortfolioChart;
        }
    }

    pub fn close_portfolio_chart(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Returns (symbol, value, percentage) sorted by value descending.
    pub fn portfolio_allocation(&self) -> Vec<(String, f64, f64)> {
        let mut items: Vec<(String, f64)> = self
            .config
            .portfolio
            .iter()
            .map(|h| {
                let price = self.quotes.get(&h.symbol).map(|q| q.price).unwrap_or(0.0);
                let value = price * h.shares() as f64;
                (h.symbol.clone(), value)
            })
            .collect();

        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        let total: f64 = items.iter().map(|(_, v)| v).sum();
        items
            .into_iter()
            .map(|(sym, val)| {
                let pct = if total > 0.0 { (val / total) * 100.0 } else { 0.0 };
                (sym, val, pct)
            })
            .collect()
    }

    // Search/filter methods
    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.input_buffer.clear();
    }

    pub fn confirm_search(&mut self) {
        if !self.input_buffer.is_empty() {
            self.search_query = self.input_buffer.trim().to_uppercase();
            self.search_active = true;
            self.selected_index = 0;
            self.portfolio_selected = 0;
        } else {
            self.clear_filter();
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn cancel_search(&mut self) {
        self.clear_filter();
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub fn clear_filter(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.selected_index = 0;
        self.portfolio_selected = 0;
    }

    pub fn get_filtered_watchlist(&self) -> Vec<(&String, Option<&StockQuote>)> {
        let mut items = self.get_sorted_watchlist();
        if self.search_active {
            items.retain(|(symbol, _)| symbol.to_uppercase().contains(&self.search_query));
        }
        if let Some(col) = self.watchlist_sort_column {
            let dir = self.watchlist_sort_direction;
            items.sort_by(|a, b| compare_watchlist_column(col, a, b, dir));
        }
        items
    }

    pub fn get_filtered_portfolio(&self) -> Vec<(usize, &crate::config::Holding)> {
        let mut items: Vec<(usize, &crate::config::Holding)> =
            self.config.portfolio.iter().enumerate().collect();
        if self.search_active {
            items.retain(|(_, h)| h.symbol.to_uppercase().contains(&self.search_query));
        }
        if let Some(col) = self.portfolio_sort_column {
            let dir = self.portfolio_sort_direction;
            let quotes = &self.quotes;
            items.sort_by(|a, b| {
                let ord = compare_portfolio_column(col, a.1, b.1, quotes);
                match dir {
                    SortDirection::Ascending => ord,
                    SortDirection::Descending => ord.reverse(),
                }
            });
        }
        items
    }

    // Export menu methods
    pub fn start_export(&mut self) {
        self.input_mode = InputMode::ExportMenu;
        self.export_menu_selection = 0;
        // Default scope to current view
        self.export_scope = match self.view_mode {
            ViewMode::Watchlist => ExportScope::Watchlist,
            ViewMode::Portfolio => ExportScope::Portfolio,
        };
    }

    pub fn cancel_export(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn toggle_export_format(&mut self) {
        self.export_format = match self.export_format {
            ExportFormat::Csv => ExportFormat::Json,
            ExportFormat::Json => ExportFormat::Csv,
        };
    }

    pub fn toggle_export_scope(&mut self) {
        self.export_scope = match self.export_scope {
            ExportScope::Watchlist => ExportScope::Portfolio,
            ExportScope::Portfolio => ExportScope::Watchlist,
        };
    }

    pub fn export_menu_up(&mut self) {
        if self.export_menu_selection > 0 {
            self.export_menu_selection -= 1;
        }
    }

    pub fn export_menu_down(&mut self) {
        if self.export_menu_selection < 2 {
            self.export_menu_selection += 1;
        }
    }

    pub fn confirm_export(&mut self) -> Result<()> {
        if self.export_menu_selection == 2 {
            // Export button selected
            let result = self.perform_export();
            self.input_mode = InputMode::Normal;
            match result {
                Ok(path) => {
                    self.status_message = Some(format!("Exported to {}", path));
                }
                Err(e) => {
                    self.status_message = Some(format!("Export failed: {}", e));
                }
            }
        }
        Ok(())
    }

    fn perform_export(&self) -> Result<String> {
        use std::fs;
        use std::io::Write;

        let dir = self.get_export_dir()?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let scope_str = match self.export_scope {
            ExportScope::Watchlist => "watchlist",
            ExportScope::Portfolio => "portfolio",
        };
        let ext = match self.export_format {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
        };
        let filename = format!("idx_{}_{}.{}", scope_str, timestamp, ext);
        let filepath = dir.join(&filename);

        let content = match (self.export_scope, self.export_format) {
            (ExportScope::Watchlist, ExportFormat::Csv) => self.export_watchlist_csv(),
            (ExportScope::Watchlist, ExportFormat::Json) => self.export_watchlist_json(),
            (ExportScope::Portfolio, ExportFormat::Csv) => self.export_portfolio_csv(),
            (ExportScope::Portfolio, ExportFormat::Json) => self.export_portfolio_json(),
        };

        let mut file = fs::File::create(&filepath)?;
        file.write_all(content.as_bytes())?;

        Ok(filepath.to_string_lossy().to_string())
    }

    fn get_export_dir(&self) -> Result<std::path::PathBuf> {
        // Try ~/Downloads first, fallback to home dir
        if let Some(home) = dirs::home_dir() {
            let downloads = home.join("Downloads");
            if downloads.exists() {
                return Ok(downloads);
            }
            return Ok(home);
        }
        anyhow::bail!("Could not determine export directory")
    }

    fn export_watchlist_csv(&self) -> String {
        let mut csv = String::from("Symbol,Name,Price,Change,Change%,Open,High,Low,Volume\n");
        for (symbol, quote) in self.get_sorted_watchlist() {
            if let Some(q) = quote {
                csv.push_str(&format!(
                    "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{}\n",
                    q.symbol,
                    q.short_name.replace(',', ";"),
                    q.price,
                    q.change,
                    q.change_percent,
                    q.open,
                    q.high,
                    q.low,
                    q.volume
                ));
            } else {
                csv.push_str(&format!("{},Loading...,,,,,,,\n", symbol));
            }
        }
        csv
    }

    fn export_watchlist_json(&self) -> String {
        let data: Vec<serde_json::Value> = self
            .get_sorted_watchlist()
            .iter()
            .map(|(symbol, quote)| {
                if let Some(q) = quote {
                    serde_json::json!({
                        "symbol": q.symbol,
                        "name": q.short_name,
                        "price": q.price,
                        "change": q.change,
                        "change_percent": q.change_percent,
                        "open": q.open,
                        "high": q.high,
                        "low": q.low,
                        "volume": q.volume
                    })
                } else {
                    serde_json::json!({
                        "symbol": symbol,
                        "name": null,
                        "price": null
                    })
                }
            })
            .collect();
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "[]".to_string())
    }

    fn export_portfolio_csv(&self) -> String {
        let mut csv = String::from("Symbol,Lots,Shares,AvgPrice,CurrentPrice,Value,Cost,PL,PL%\n");
        for holding in &self.config.portfolio {
            let curr_price = self.quotes.get(&holding.symbol).map(|q| q.price).unwrap_or(0.0);
            let shares = holding.shares();
            let (value, cost, pl, pl_percent) = holding.pl_metrics(curr_price);

            csv.push_str(&format!(
                "{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
                holding.symbol,
                holding.lots,
                shares,
                holding.avg_price,
                curr_price,
                value,
                cost,
                pl,
                pl_percent
            ));
        }
        csv
    }

    fn export_portfolio_json(&self) -> String {
        let data: Vec<serde_json::Value> = self
            .config
            .portfolio
            .iter()
            .map(|holding| {
                let curr_price = self.quotes.get(&holding.symbol).map(|q| q.price).unwrap_or(0.0);
                let shares = holding.shares();
                let (value, cost, pl, pl_percent) = holding.pl_metrics(curr_price);

                serde_json::json!({
                    "symbol": holding.symbol,
                    "lots": holding.lots,
                    "shares": shares,
                    "avg_price": holding.avg_price,
                    "current_price": curr_price,
                    "value": value,
                    "cost": cost,
                    "pl": pl,
                    "pl_percent": pl_percent
                })
            })
            .collect();
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Helper: compare two f64 values with NaN safety
fn cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Compare two watchlist rows by column index.
/// None quotes (still loading) always sort to the bottom, regardless of direction.
fn compare_watchlist_column(
    col: usize,
    a: &(&String, Option<&StockQuote>),
    b: &(&String, Option<&StockQuote>),
    direction: SortDirection,
) -> Ordering {
    match (a.1, b.1) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater, // always bottom
        (Some(_), None) => Ordering::Less,    // always bottom
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

/// Compare two portfolio rows by column index.
/// Uses quotes map for current-price-dependent fields.
fn compare_portfolio_column(
    col: usize,
    a: &crate::config::Holding,
    b: &crate::config::Holding,
    quotes: &HashMap<String, StockQuote>,
) -> Ordering {
    let price_a = quotes.get(&a.symbol).map(|q| q.price).unwrap_or(0.0);
    let price_b = quotes.get(&b.symbol).map(|q| q.price).unwrap_or(0.0);

    match col {
        0 => a.symbol.cmp(&b.symbol),
        1 => a.lots.cmp(&b.lots),
        2 => a.shares().cmp(&b.shares()),
        3 => cmp_f64(a.avg_price, b.avg_price),
        4 => cmp_f64(price_a, price_b),
        5 => cmp_f64(a.pl_metrics(price_a).0, b.pl_metrics(price_b).0), // value
        6 => cmp_f64(a.cost_basis(), b.cost_basis()),
        7 => cmp_f64(a.pl_metrics(price_a).2, b.pl_metrics(price_b).2), // P/L
        8 => cmp_f64(a.pl_metrics(price_a).3, b.pl_metrics(price_b).3), // P/L%
        _ => Ordering::Equal,
    }
}
