use super::{App, ExportFormat, ExportScope, InputMode, ViewMode};
use anyhow::Result;
use chrono::Local;

impl App {
    pub fn start_export(&mut self) {
        self.input_mode = InputMode::ExportMenu;
        self.export_menu_selection = 0;
        self.export_scope = match self.view_mode {
            ViewMode::Watchlist | ViewMode::News => ExportScope::Watchlist,
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
        if let Some(home) = dirs::home_dir() {
            let downloads = home.join("Downloads");
            if downloads.exists() {
                return Ok(downloads);
            }
            return Ok(home);
        }
        Ok(std::env::current_dir()?)
    }

    fn export_watchlist_csv(&self) -> String {
        let mut csv = String::from("Symbol,Name,Price,Change,Change%,Open,High,Low,Volume\n");
        for (symbol, quote) in self.get_raw_watchlist() {
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
            .get_raw_watchlist()
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
            let curr_price = self
                .quotes
                .get(&holding.symbol)
                .map(|q| q.price)
                .unwrap_or(0.0);
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
                let curr_price = self
                    .quotes
                    .get(&holding.symbol)
                    .map(|q| q.price)
                    .unwrap_or(0.0);
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
