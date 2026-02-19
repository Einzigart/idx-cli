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
pub struct Config {
    pub watchlists: Vec<Watchlist>,
    #[serde(default)]
    pub active_watchlist: usize,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    #[serde(default)]
    pub portfolio: Vec<Holding>,
    #[serde(default = "default_news_sources")]
    pub news_sources: Vec<String>,
}

fn default_refresh_interval() -> u64 {
    1
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
                    symbols: vec![
                        "TLKM".to_string(),
                        "GOTO".to_string(),
                        "BUKA".to_string(),
                    ],
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
        self.current_watchlist_mut().symbols.retain(|s| s != &symbol);
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

    /// Add a new holding or merge into an existing one.
    /// Returns `false` if merging would overflow the lot count.
    pub fn add_holding(&mut self, symbol: &str, lots: u32, avg_price: f64) -> bool {
        let symbol = symbol.to_uppercase();
        // Check if holding exists, update it
        if let Some(holding) = self.portfolio.iter_mut().find(|h| h.symbol == symbol) {
            let total_lots = match holding.lots.checked_add(lots) {
                Some(t) => t,
                None => return false,
            };
            let total_cost = holding.cost_basis() + (lots as u64 * 100) as f64 * avg_price;
            holding.avg_price = total_cost / (total_lots as u64 * 100) as f64;
            holding.lots = total_lots;
        } else {
            self.portfolio.push(Holding {
                symbol,
                lots,
                avg_price,
            });
        }
        true
    }

    pub fn remove_holding(&mut self, symbol: &str) {
        let symbol = symbol.to_uppercase();
        self.portfolio.retain(|h| h.symbol != symbol);
    }

    pub fn update_holding(&mut self, symbol: &str, lots: u32, avg_price: f64) {
        if let Some(holding) = self.portfolio.iter_mut().find(|h| h.symbol == symbol) {
            holding.lots = lots;
            holding.avg_price = avg_price;
        }
    }

    pub fn portfolio_symbols(&self) -> Vec<String> {
        self.portfolio.iter().map(|h| h.symbol.clone()).collect()
    }

    #[cfg(test)]
    fn test_config() -> Self {
        Self {
            watchlists: vec![Watchlist::default()],
            active_watchlist: 0,
            refresh_interval_secs: 1,
            portfolio: Vec::new(),
            news_sources: Vec::new(),
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
        let h = config.portfolio.iter().find(|h| h.symbol == "BBCA").unwrap();
        assert_eq!(h.lots, 3_000_000_000);
        assert_eq!(h.avg_price, 8000.0);
    }

    #[test]
    fn add_holding_normal_merge() {
        let mut config = Config::test_config();
        config.add_holding("BBCA", 100, 8000.0);
        let ok = config.add_holding("BBCA", 100, 9000.0);
        assert!(ok);
        let h = config.portfolio.iter().find(|h| h.symbol == "BBCA").unwrap();
        assert_eq!(h.lots, 200);
        // Weighted avg: (100*100*8000 + 100*100*9000) / (200*100) = 8500
        assert!((h.avg_price - 8500.0).abs() < 0.01);
    }

    #[test]
    fn add_holding_new_symbol() {
        let mut config = Config::test_config();
        let ok = config.add_holding("TLKM", 50, 3500.0);
        assert!(ok);
        assert_eq!(config.portfolio.len(), 1);
        assert_eq!(config.portfolio[0].symbol, "TLKM");
        assert_eq!(config.portfolio[0].lots, 50);
    }
}
