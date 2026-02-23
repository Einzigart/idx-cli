use super::{App, InputMode};
use anyhow::Result;
use std::cmp::Ordering;

impl App {
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
                        if self.config.add_holding(symbol, lots, avg_price) {
                            self.config.save()?;
                            self.status_message =
                                Some(format!("Added {} lots of {} @ {}", lots, symbol, avg_price));
                        } else {
                            self.status_message =
                                Some("Total lots would exceed maximum (4,294,967,295)".to_string());
                        }
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

    pub fn start_portfolio_edit(&mut self) {
        if let Some(symbol) = self.selected_portfolio_symbol()
            && let Some(holding) = self
                .config
                .current_portfolio()
                .holdings
                .iter()
                .find(|h| h.symbol == symbol)
        {
            self.pending_edit_symbol = Some(symbol);
            self.input_buffer = holding.lots.to_string();
            self.input_mode = InputMode::PortfolioEditLots;
        }
    }

    pub fn confirm_portfolio_edit_lots(&mut self) {
        if let Ok(lots) = self.input_buffer.trim().parse::<u32>() {
            if lots > 0 {
                self.pending_lots = Some(lots);
                // Pre-fill with current avg_price
                if let Some(ref symbol) = self.pending_edit_symbol {
                    if let Some(holding) = self
                        .config
                        .current_portfolio()
                        .holdings
                        .iter()
                        .find(|h| &h.symbol == symbol)
                    {
                        self.input_buffer = holding.avg_price.to_string();
                    } else {
                        self.input_buffer.clear();
                    }
                }
                self.input_mode = InputMode::PortfolioEditPrice;
            } else {
                self.status_message = Some("Lots must be greater than 0".to_string());
                self.input_buffer.clear();
            }
        } else {
            self.status_message = Some("Invalid number for lots".to_string());
            self.input_buffer.clear();
        }
    }

    pub fn confirm_portfolio_edit_price(&mut self) -> Result<()> {
        if let Ok(avg_price) = self.input_buffer.trim().parse::<f64>() {
            if avg_price > 0.0 {
                match (&self.pending_edit_symbol, self.pending_lots) {
                    (Some(symbol), Some(lots)) => {
                        self.config.update_holding(symbol, lots, avg_price);
                        self.config.save()?;
                        self.status_message = Some(format!(
                            "Updated {} → {} lots @ {}",
                            symbol, lots, avg_price
                        ));
                    }
                    _ => {
                        self.status_message = Some("Missing edit data".to_string());
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
        self.pending_edit_symbol = None;
        self.pending_lots = None;
        Ok(())
    }

    pub fn cancel_portfolio_edit(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.pending_edit_symbol = None;
        self.pending_lots = None;
    }

    pub fn remove_selected_holding(&mut self) -> Result<()> {
        if let Some(symbol) = self.selected_portfolio_symbol() {
            self.config.remove_holding(&symbol);
            self.config.save()?;
            self.quotes.remove(&symbol);
            self.status_message = Some(format!("Removed {}", symbol));
            let len = self.get_filtered_portfolio().len();
            if self.portfolio_selected >= len && self.portfolio_selected > 0 {
                self.portfolio_selected -= 1;
            }
        }
        Ok(())
    }

    pub async fn show_portfolio_detail(&mut self) {
        if let Some(symbol) = self.selected_portfolio_symbol() {
            self.open_detail(&symbol).await;
        }
    }

    pub fn portfolio_indicator(&self) -> String {
        format!(
            "{} ({}/{})",
            self.config.current_portfolio().name,
            self.config.active_portfolio + 1,
            self.config.portfolios.len()
        )
    }

    pub fn next_portfolio(&mut self) {
        self.config.next_portfolio();
        self.portfolio_selected = 0;
        *self.portfolio_table_state.offset_mut() = 0;
        self.quotes.clear();
        self.portfolio_sort_column = None;
    }

    pub fn prev_portfolio(&mut self) {
        self.config.prev_portfolio();
        self.portfolio_selected = 0;
        *self.portfolio_table_state.offset_mut() = 0;
        self.quotes.clear();
        self.portfolio_sort_column = None;
    }

    pub fn start_portfolio_new(&mut self) {
        self.input_mode = InputMode::PortfolioNew;
        self.input_buffer.clear();
    }

    pub fn start_portfolio_rename(&mut self) {
        self.input_mode = InputMode::PortfolioRename;
        self.input_buffer = self.config.current_portfolio().name.clone();
    }

    pub fn confirm_portfolio_new(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let name = self.input_buffer.trim().to_string();
            self.config.add_portfolio(&name);
            self.config.save()?;
            self.quotes.clear();
            self.portfolio_selected = 0;
            *self.portfolio_table_state.offset_mut() = 0;
            self.status_message = Some(format!("Created portfolio '{}'", name));
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    pub fn confirm_portfolio_rename(&mut self) -> Result<()> {
        if !self.input_buffer.is_empty() {
            let new_name = self.input_buffer.trim().to_string();
            let old_name = self.config.current_portfolio().name.clone();
            self.config.rename_portfolio(&new_name);
            self.config.save()?;
            self.status_message = Some(format!("Renamed '{}' to '{}'", old_name, new_name));
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        Ok(())
    }

    pub fn remove_current_portfolio(&mut self) -> Result<()> {
        if self.config.portfolios.len() > 1 {
            let name = self.config.current_portfolio().name.clone();
            self.config.remove_portfolio();
            self.config.save()?;
            self.quotes.clear();
            self.portfolio_selected = 0;
            *self.portfolio_table_state.offset_mut() = 0;
            self.status_message = Some(format!("Removed portfolio '{}'", name));
        } else {
            self.status_message = Some("Cannot remove the last portfolio".to_string());
        }
        Ok(())
    }

    pub fn show_portfolio_chart(&mut self) {
        if !self.config.current_portfolio().holdings.is_empty() {
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
            .current_portfolio()
            .holdings
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
                let pct = if total > 0.0 {
                    (val / total) * 100.0
                } else {
                    0.0
                };
                (sym, val, pct)
            })
            .collect()
    }
}
