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

    #[cfg(test)]
    fn test_config() -> Self {
        Self {
            watchlists: vec![Watchlist::default()],
            active_watchlist: 0,
            refresh_interval_secs: 1,
            portfolio: Vec::new(),
            portfolios: default_portfolios(),
            active_portfolio: 0,
            news_sources: Vec::new(),
        }
    }

    /// Migrate old flat `portfolio` field into `portfolios` groups.
    fn migrate_portfolio(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_holding_overflow_returns_false() {
        let mut config = Config::test_config();
        config.add_holding("BBCA", 3_000_000_000, 8000.0);
        // Adding 2 billion more would exceed u32::MAX (4,294,967,295)
        let ok = config.add_holding("BBCA", 2_000_000_000, 9000.0);
        assert!(!ok, "add_holding should return false on u32 overflow");
        // Original holding should be unchanged
        let h = config
            .current_portfolio()
            .holdings
            .iter()
            .find(|h| h.symbol == "BBCA")
            .unwrap();
        assert_eq!(h.lots, 3_000_000_000);
        assert_eq!(h.avg_price, 8000.0);
    }

    #[test]
    fn add_holding_normal_merge() {
        let mut config = Config::test_config();
        config.add_holding("BBCA", 100, 8000.0);
        let ok = config.add_holding("BBCA", 100, 9000.0);
        assert!(ok);
        let h = config
            .current_portfolio()
            .holdings
            .iter()
            .find(|h| h.symbol == "BBCA")
            .unwrap();
        assert_eq!(h.lots, 200);
        // Weighted avg: (100*100*8000 + 100*100*9000) / (200*100) = 8500
        assert!((h.avg_price - 8500.0).abs() < 0.01);
    }

    #[test]
    fn add_holding_new_symbol() {
        let mut config = Config::test_config();
        let ok = config.add_holding("TLKM", 50, 3500.0);
        assert!(ok);
        assert_eq!(config.current_portfolio().holdings.len(), 1);
        assert_eq!(config.current_portfolio().holdings[0].symbol, "TLKM");
        assert_eq!(config.current_portfolio().holdings[0].lots, 50);
    }

    #[test]
    fn migrate_flat_portfolio_to_portfolios() {
        let json = r#"{
            "watchlists": [{"name": "Default", "symbols": ["BBCA"]}],
            "active_watchlist": 0,
            "portfolio": [
                {"symbol": "BBCA", "lots": 100, "avg_price": 8000.0},
                {"symbol": "TLKM", "lots": 50, "avg_price": 3500.0}
            ]
        }"#;
        let mut config: Config = serde_json::from_str(json).unwrap();
        config.migrate_portfolio();
        assert_eq!(config.portfolios.len(), 1);
        assert_eq!(config.portfolios[0].name, "Default");
        assert_eq!(config.portfolios[0].holdings.len(), 2);
        assert_eq!(config.portfolios[0].holdings[0].symbol, "BBCA");
    }

    #[test]
    fn new_format_loads_directly() {
        let json = r#"{
            "watchlists": [{"name": "Default", "symbols": ["BBCA"]}],
            "active_watchlist": 0,
            "portfolios": [
                {"name": "Growth", "holdings": [{"symbol": "BBCA", "lots": 100, "avg_price": 8000.0}]},
                {"name": "Dividend", "holdings": [{"symbol": "TLKM", "lots": 50, "avg_price": 3500.0}]}
            ],
            "active_portfolio": 1
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.portfolios.len(), 2);
        assert_eq!(config.active_portfolio, 1);
        assert_eq!(config.portfolios[1].name, "Dividend");
    }

    #[test]
    fn portfolio_crud_operations() {
        let mut config = Config::test_config();
        assert_eq!(config.portfolios.len(), 1);

        config.add_portfolio("Growth");
        assert_eq!(config.portfolios.len(), 2);
        assert_eq!(config.active_portfolio, 1);
        assert_eq!(config.current_portfolio().name, "Growth");

        config.rename_portfolio("Aggressive Growth");
        assert_eq!(config.current_portfolio().name, "Aggressive Growth");

        config.next_portfolio();
        assert_eq!(config.active_portfolio, 0);
        config.prev_portfolio();
        assert_eq!(config.active_portfolio, 1);

        config.remove_portfolio();
        assert_eq!(config.portfolios.len(), 1);
        assert_eq!(config.active_portfolio, 0);

        config.remove_portfolio();
        assert_eq!(config.portfolios.len(), 1);
    }
}
