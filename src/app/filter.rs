use super::sort::{
    compare_bookmark_column, compare_news_column, compare_portfolio_column,
    compare_watchlist_column,
};
use super::{App, InputMode, SortDirection};
use crate::api::{NewsItem, StockQuote};
use crate::config::Bookmark;

impl App {
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
            self.news_selected = 0;
            *self.watchlist_table_state.offset_mut() = 0;
            *self.portfolio_table_state.offset_mut() = 0;
            *self.news_table_state.offset_mut() = 0;
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
        self.news_selected = 0;
        self.bookmark_selected = 0;
        *self.watchlist_table_state.offset_mut() = 0;
        *self.portfolio_table_state.offset_mut() = 0;
        *self.news_table_state.offset_mut() = 0;
        *self.bookmark_table_state.offset_mut() = 0;
    }

    pub fn get_raw_watchlist(&self) -> Vec<(&String, Option<&StockQuote>)> {
        self.config
            .current_watchlist()
            .symbols
            .iter()
            .map(|symbol| (symbol, self.quotes.get(symbol)))
            .collect()
    }

    pub fn get_filtered_watchlist(&self) -> Vec<(&String, Option<&StockQuote>)> {
        let mut items = self.get_raw_watchlist();
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
        let mut items: Vec<(usize, &crate::config::Holding)> = self
            .config
            .current_portfolio()
            .holdings
            .iter()
            .enumerate()
            .collect();
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

    pub fn selected_watchlist_symbol(&self) -> Option<String> {
        let filtered = self.get_filtered_watchlist();
        filtered.get(self.selected_index).map(|(s, _)| (*s).clone())
    }

    pub fn selected_portfolio_symbol(&self) -> Option<String> {
        let filtered = self.get_filtered_portfolio();
        filtered
            .get(self.portfolio_selected)
            .map(|(_, h)| h.symbol.clone())
    }

    pub fn get_filtered_news(&self) -> Vec<&NewsItem> {
        let mut items: Vec<&NewsItem> = self.news_items.iter().collect();
        if self.search_active {
            items.retain(|item| {
                item.title.to_uppercase().contains(&self.search_query)
                    || item.publisher.to_uppercase().contains(&self.search_query)
            });
        }
        if let Some(col) = self.news_sort_column {
            let dir = self.news_sort_direction;
            items.sort_by(|a, b| {
                let ord = compare_news_column(col, a, b);
                match dir {
                    SortDirection::Ascending => ord,
                    SortDirection::Descending => ord.reverse(),
                }
            });
        }
        items
    }

    pub fn get_filtered_bookmarks(&self) -> Vec<&Bookmark> {
        let mut items: Vec<&Bookmark> = self.config.bookmarks.iter().collect();
        if self.search_active {
            items.retain(|b| {
                b.headline.to_uppercase().contains(&self.search_query)
                    || b.source.to_uppercase().contains(&self.search_query)
            });
        }
        // Default sort: by bookmarked_at descending (most recent first)
        if let Some(col) = self.bookmark_sort_column {
            let dir = self.bookmark_sort_direction;
            items.sort_by(|a, b| {
                let ord = compare_bookmark_column(col, a, b);
                match dir {
                    SortDirection::Ascending => ord,
                    SortDirection::Descending => ord.reverse(),
                }
            });
        } else {
            // No explicit sort column: default to bookmarked_at descending
            items.sort_by(|a, b| b.bookmarked_at.cmp(&a.bookmarked_at));
        }
        items
    }
}
