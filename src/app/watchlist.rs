use super::{App, InputMode};
use anyhow::Result;

impl App {
    pub fn start_adding(&mut self) {
        self.input_mode = InputMode::Adding;
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
        if let Some(symbol) = self.selected_watchlist_symbol() {
            self.config.remove_stock(&symbol);
            self.config.save()?;
            self.quotes.remove(&symbol);
            self.status_message = Some(format!("Removed {}", symbol));
            let len = self.get_filtered_watchlist().len();
            if self.selected_index >= len && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
        Ok(())
    }

    pub fn watchlist_indicator(&self) -> String {
        format!(
            "{} ({}/{})",
            self.config.current_watchlist().name,
            self.config.active_watchlist + 1,
            self.config.watchlists.len()
        )
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
        if let Some(symbol) = self.selected_watchlist_symbol() {
            self.open_detail(&symbol).await;
        }
    }
}
