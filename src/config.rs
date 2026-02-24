use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub name: String,
    pub symbols: Vec<String>,
}

impl Default for Watchlist {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            symbols: vec![
                "BBCA".to_string(),
                "BBRI".to_string(),
                "TLKM".to_string(),
                "ASII".to_string(),
            ],
        }
    }
}

/// A holding in the portfolio (1 lot = 100 shares for IDX)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    pub symbol: String,
    pub lots: u32,
    pub avg_price: f64,
}

impl Holding {
    pub fn shares(&self) -> u64 {
        self.lots as u64 * 100
    }

    pub fn cost_basis(&self) -> f64 {
        self.shares() as f64 * self.avg_price
    }

    /// Calculate P/L metrics given the current market price
    pub fn pl_metrics(&self, current_price: f64) -> (f64, f64, f64, f64) {
        let shares = self.shares();
        let value = current_price * shares as f64;
        let cost = self.cost_basis();
        let pl = value - cost;
        let pl_pct = if cost > 0.0 { (pl / cost) * 100.0 } else { 0.0 };
        (value, cost, pl, pl_pct)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    Above,
    Below,
    PercentGain,
    PercentLoss,
}

impl AlertType {
    pub fn label(&self) -> &'static str {
        match self {
            AlertType::Above => "Above",
            AlertType::Below => "Below",
            AlertType::PercentGain => "% Gain",
            AlertType::PercentLoss => "% Loss",
        }
    }

    pub fn next(&self) -> AlertType {
        match self {
            AlertType::Above => AlertType::Below,
            AlertType::Below => AlertType::PercentGain,
            AlertType::PercentGain => AlertType::PercentLoss,
            AlertType::PercentLoss => AlertType::Above,
        }
    }

    pub fn prev(&self) -> AlertType {
        match self {
            AlertType::Above => AlertType::PercentLoss,
            AlertType::Below => AlertType::Above,
            AlertType::PercentGain => AlertType::Below,
            AlertType::PercentLoss => AlertType::PercentGain,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub symbol: String,
    pub alert_type: AlertType,
    pub target_value: f64,
    pub enabled: bool,
    pub last_triggered: Option<u64>,
    pub cooldown_seconds: u32,
}

impl Alert {
    pub fn new(symbol: &str, alert_type: AlertType, target_value: f64) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: format!("{}_{}", ts, symbol),
            symbol: symbol.to_uppercase(),
            alert_type,
            target_value,
            enabled: true,
            last_triggered: None,
            cooldown_seconds: 300,
        }
    }

    pub fn should_trigger(&self, price: f64, change_pct: f64) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(last) = self.last_triggered {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(last) < self.cooldown_seconds as u64 {
                return false;
            }
        }
        match self.alert_type {
            AlertType::Above => price >= self.target_value,
            AlertType::Below => price <= self.target_value,
            AlertType::PercentGain => change_pct >= self.target_value,
            AlertType::PercentLoss => change_pct <= -self.target_value,
        }
    }
}

fn default_alerts() -> Vec<Alert> {
    Vec::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub headline: String,
    pub source: String,
    pub url: Option<String>,
    pub published_at: i64,
    pub bookmarked_at: i64,
    pub read: bool,
}

fn default_bookmarks() -> Vec<Bookmark> {
    Vec::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub name: String,
    pub holdings: Vec<Holding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub watchlists: Vec<Watchlist>,
    #[serde(default)]
    pub active_watchlist: usize,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    /// Old format field — consumed during deserialization for migration
    #[serde(default, skip_serializing)]
    portfolio: Vec<Holding>,
    #[serde(default = "default_portfolios")]
    pub portfolios: Vec<Portfolio>,
    #[serde(default)]
    pub active_portfolio: usize,
    #[serde(default = "default_news_sources")]
    pub news_sources: Vec<String>,
    #[serde(default = "default_alerts")]
    pub alerts: Vec<Alert>,
    #[serde(default = "default_bookmarks")]
    pub bookmarks: Vec<Bookmark>,
}

fn default_refresh_interval() -> u64 {
    1
}

fn default_portfolios() -> Vec<Portfolio> {
    vec![Portfolio {
        name: "Default".to_string(),
        holdings: Vec::new(),
    }]
}

fn default_news_sources() -> Vec<String> {
    vec![
        "https://www.cnbcindonesia.com/market/rss".to_string(),
        "https://www.cnbcindonesia.com/news/rss".to_string(),
        "https://www.idxchannel.com/rss".to_string(),
        "https://rss.tempo.co/bisnis".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watchlists: vec![
                Watchlist {
                    name: "Banking".to_string(),
                    symbols: vec![
                        "BBCA".to_string(),
                        "BBRI".to_string(),
                        "BMRI".to_string(),
                        "BBNI".to_string(),
                    ],
                },
                Watchlist {
                    name: "Tech".to_string(),
                    symbols: vec!["TLKM".to_string(), "GOTO".to_string(), "BUKA".to_string()],
                },
                Watchlist {
                    name: "Mining".to_string(),
                    symbols: vec![
                        "ADRO".to_string(),
                        "ANTM".to_string(),
                        "INCO".to_string(),
                        "PTBA".to_string(),
                    ],
                },
            ],
            active_watchlist: 0,
            refresh_interval_secs: default_refresh_interval(),
            portfolio: Vec::new(),
            portfolios: vec![Portfolio {
                name: "Default".to_string(),
                holdings: Vec::new(),
            }],
            active_portfolio: 0,
            news_sources: default_news_sources(),
            alerts: default_alerts(),
            bookmarks: default_bookmarks(),
        }
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("idx-cli");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        if config.watchlists.is_empty() {
            config.watchlists.push(Watchlist::default());
        }
        if config.active_watchlist >= config.watchlists.len() {
            config.active_watchlist = 0;
        }
        // Migrate old flat portfolio → portfolios
        config.migrate_portfolio();
        if config.migrate_news_sources() {
            let _ = config.save();
        }
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn current_watchlist(&self) -> &Watchlist {
        &self.watchlists[self.active_watchlist]
    }

    pub fn current_watchlist_mut(&mut self) -> &mut Watchlist {
        &mut self.watchlists[self.active_watchlist]
    }

    pub fn next_watchlist(&mut self) {
        if !self.watchlists.is_empty() {
            self.active_watchlist = (self.active_watchlist + 1) % self.watchlists.len();
        }
    }

    pub fn prev_watchlist(&mut self) {
        if !self.watchlists.is_empty() {
            self.active_watchlist = if self.active_watchlist == 0 {
                self.watchlists.len() - 1
            } else {
                self.active_watchlist - 1
            };
        }
    }

    pub fn add_stock(&mut self, symbol: &str) {
        let symbol = symbol.to_uppercase();
        let watchlist = self.current_watchlist_mut();
        if !watchlist.symbols.contains(&symbol) {
            watchlist.symbols.push(symbol);
        }
    }

    pub fn remove_stock(&mut self, symbol: &str) {
        let symbol = symbol.to_uppercase();
        self.current_watchlist_mut()
            .symbols
            .retain(|s| s != &symbol);
    }

    pub fn add_watchlist(&mut self, name: &str) {
        self.watchlists.push(Watchlist {
            name: name.to_string(),
            symbols: Vec::new(),
        });
        self.active_watchlist = self.watchlists.len() - 1;
    }

    pub fn remove_watchlist(&mut self) {
        if self.watchlists.len() > 1 {
            self.watchlists.remove(self.active_watchlist);
            if self.active_watchlist >= self.watchlists.len() {
                self.active_watchlist = self.watchlists.len() - 1;
            }
        }
    }

    pub fn rename_watchlist(&mut self, new_name: &str) {
        self.current_watchlist_mut().name = new_name.to_string();
    }

    pub fn current_portfolio(&self) -> &Portfolio {
        &self.portfolios[self.active_portfolio]
    }

    pub fn current_portfolio_mut(&mut self) -> &mut Portfolio {
        &mut self.portfolios[self.active_portfolio]
    }

    pub fn next_portfolio(&mut self) {
        if !self.portfolios.is_empty() {
            self.active_portfolio = (self.active_portfolio + 1) % self.portfolios.len();
        }
    }

    pub fn prev_portfolio(&mut self) {
        if !self.portfolios.is_empty() {
            self.active_portfolio = if self.active_portfolio == 0 {
                self.portfolios.len() - 1
            } else {
                self.active_portfolio - 1
            };
        }
    }

    pub fn add_portfolio(&mut self, name: &str) {
        self.portfolios.push(Portfolio {
            name: name.to_string(),
            holdings: Vec::new(),
        });
        self.active_portfolio = self.portfolios.len() - 1;
    }

    pub fn remove_portfolio(&mut self) {
        if self.portfolios.len() > 1 {
            self.portfolios.remove(self.active_portfolio);
            if self.active_portfolio >= self.portfolios.len() {
                self.active_portfolio = self.portfolios.len() - 1;
            }
        }
    }

    pub fn rename_portfolio(&mut self, new_name: &str) {
        self.current_portfolio_mut().name = new_name.to_string();
    }

    pub fn alerts_for_symbol(&self, symbol: &str) -> Vec<&Alert> {
        let sym = symbol.to_uppercase();
        self.alerts.iter().filter(|a| a.symbol == sym).collect()
    }

    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    pub fn remove_alert(&mut self, id: &str) {
        self.alerts.retain(|a| a.id != id);
    }

    pub fn toggle_alert(&mut self, id: &str) {
        if let Some(a) = self.alerts.iter_mut().find(|a| a.id == id) {
            a.enabled = !a.enabled;
        }
    }

    pub fn mark_triggered(&mut self, id: &str) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(a) = self.alerts.iter_mut().find(|a| a.id == id) {
            a.last_triggered = Some(now);
        }
    }

    pub fn has_active_alerts(&self, symbol: &str) -> bool {
        let sym = symbol.to_uppercase();
        self.alerts.iter().any(|a| a.symbol == sym && a.enabled)
    }

    /// Check if an article is bookmarked by matching headline and url.
    pub fn is_bookmarked(&self, headline: &str, url: Option<&str>) -> bool {
        self.bookmarks
            .iter()
            .any(|b| b.headline == headline && b.url.as_deref() == url)
    }

    /// Add a bookmark, returning false if duplicate.
    pub fn add_bookmark(&mut self, bookmark: Bookmark) -> bool {
        if self.is_bookmarked(&bookmark.headline, bookmark.url.as_deref()) {
            return false;
        }
        self.bookmarks.push(bookmark);
        true
    }

    /// Remove a bookmark by index.
    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    /// Remove all bookmarks.
    pub fn clear_bookmarks(&mut self) {
        self.bookmarks.clear();
    }

    /// Toggle read/unread status for a bookmark at the given index.
    pub fn toggle_bookmark_read(&mut self, index: usize) {
        if let Some(b) = self.bookmarks.get_mut(index) {
            b.read = !b.read;
        }
    }

    /// Mark a bookmark as read by index.
    pub fn mark_bookmark_read(&mut self, index: usize) {
        if let Some(b) = self.bookmarks.get_mut(index) {
            b.read = true;
        }
    }

    /// Add a new holding or merge into an existing one.
    pub fn add_holding(&mut self, symbol: &str, lots: u32, avg_price: f64) -> bool {
        let symbol = symbol.to_uppercase();
        // Check if holding exists, update it
        if let Some(holding) = self
            .current_portfolio_mut()
            .holdings
            .iter_mut()
            .find(|h| h.symbol == symbol)
        {
            let total_lots = match holding.lots.checked_add(lots) {
                Some(t) => t,
                None => return false,
            };
            let total_cost = holding.cost_basis() + (lots as u64 * 100) as f64 * avg_price;
            holding.avg_price = total_cost / (total_lots as u64 * 100) as f64;
            holding.lots = total_lots;
        } else {
            self.current_portfolio_mut().holdings.push(Holding {
                symbol,
                lots,
                avg_price,
            });
        }
        true
    }

    pub fn remove_holding(&mut self, symbol: &str) {
        let symbol = symbol.to_uppercase();
        self.current_portfolio_mut()
            .holdings
            .retain(|h| h.symbol != symbol);
    }

    pub fn update_holding(&mut self, symbol: &str, lots: u32, avg_price: f64) {
        if let Some(holding) = self
            .current_portfolio_mut()
            .holdings
            .iter_mut()
            .find(|h| h.symbol == symbol)
        {
            holding.lots = lots;
            holding.avg_price = avg_price;
        }
    }

    pub fn portfolio_symbols(&self) -> Vec<String> {
        self.current_portfolio()
            .holdings
            .iter()
            .map(|h| h.symbol.clone())
            .collect()
    }

    pub fn test_config() -> Self {
        Self {
            watchlists: vec![Watchlist::default()],
            active_watchlist: 0,
            refresh_interval_secs: 1,
            portfolio: Vec::new(),
            portfolios: default_portfolios(),
            active_portfolio: 0,
            news_sources: Vec::new(),
            alerts: Vec::new(),
            bookmarks: Vec::new(),
        }
    }

    /// Migrate old flat `portfolio` field into `portfolios` groups.
    pub fn migrate_portfolio(&mut self) {
        if !self.portfolio.is_empty() {
            if self.portfolios.len() == 1 && self.portfolios[0].holdings.is_empty() {
                self.portfolios[0].holdings = std::mem::take(&mut self.portfolio);
            } else {
                self.portfolios.push(Portfolio {
                    name: "Imported".to_string(),
                    holdings: std::mem::take(&mut self.portfolio),
                });
            }
            let _ = self.save();
        }
        if self.portfolios.is_empty() {
            self.portfolios = default_portfolios();
        }
        if self.active_portfolio >= self.portfolios.len() {
            self.active_portfolio = 0;
        }
    }

    /// Replace dead RSS feeds with working alternatives. Returns true if changed.
    fn migrate_news_sources(&mut self) -> bool {
        const DEAD_KONTAN: &str = "https://www.kontan.co.id/rss/investasi";
        self.news_sources.retain(|u| u != DEAD_KONTAN);
        let defaults = default_news_sources();
        let mut changed = false;
        for url in &defaults {
            if !self.news_sources.contains(url) {
                self.news_sources.push(url.clone());
                changed = true;
            }
        }
        changed
    }
}
