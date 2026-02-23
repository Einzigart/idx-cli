use super::sort::{compare_news_column, compare_portfolio_column, compare_watchlist_column};
use super::{App, InputMode, SortDirection};
use crate::api::{NewsItem, StockQuote};

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
        *self.watchlist_table_state.offset_mut() = 0;
        *self.portfolio_table_state.offset_mut() = 0;
        *self.news_table_state.offset_mut() = 0;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{make_news_item, make_quote};
    use crate::config::Holding;

    fn test_app() -> App {
        crate::app::tests::test_app()
    }

    // --- get_filtered_watchlist ---

    #[test]
    fn test_filtered_watchlist_no_filter() {
        let app = test_app();
        let filtered = app.get_filtered_watchlist();
        // Default watchlist: BBCA, BBRI, TLKM, ASII
        assert_eq!(filtered.len(), 4);
        assert_eq!(*filtered[0].0, "BBCA");
        assert_eq!(*filtered[3].0, "ASII");
    }

    #[test]
    fn test_filtered_watchlist_with_search() {
        let mut app = test_app();
        app.search_active = true;
        app.search_query = "BB".to_string();
        let filtered = app.get_filtered_watchlist();
        assert_eq!(filtered.len(), 2);
        let symbols: Vec<&str> = filtered.iter().map(|(s, _)| s.as_str()).collect();
        assert!(symbols.contains(&"BBCA"));
        assert!(symbols.contains(&"BBRI"));
    }

    #[test]
    fn test_filtered_watchlist_with_sort() {
        let mut app = test_app();
        app.quotes
            .insert("BBCA".to_string(), make_quote("BBCA", 9000.0, 50.0, 0.6));
        app.quotes
            .insert("BBRI".to_string(), make_quote("BBRI", 5000.0, 30.0, 0.6));
        app.quotes
            .insert("TLKM".to_string(), make_quote("TLKM", 3000.0, 10.0, 0.3));
        app.quotes
            .insert("ASII".to_string(), make_quote("ASII", 7000.0, -20.0, -0.3));
        app.watchlist_sort_column = Some(2); // sort by price
        let filtered = app.get_filtered_watchlist();
        // Ascending: TLKM(3000) < BBRI(5000) < ASII(7000) < BBCA(9000)
        assert_eq!(*filtered[0].0, "TLKM");
        assert_eq!(*filtered[3].0, "BBCA");
    }

    #[test]
    fn test_filtered_watchlist_empty() {
        let mut app = test_app();
        app.config.watchlists[0].symbols.clear();
        let filtered = app.get_filtered_watchlist();
        assert!(filtered.is_empty());
    }

    // --- get_filtered_portfolio ---

    #[test]
    fn test_filtered_portfolio_no_filter() {
        let mut app = test_app();
        app.config.portfolios[0].holdings.push(Holding {
            symbol: "BBCA".to_string(),
            lots: 10,
            avg_price: 8000.0,
        });
        app.config.portfolios[0].holdings.push(Holding {
            symbol: "TLKM".to_string(),
            lots: 20,
            avg_price: 3000.0,
        });
        let filtered = app.get_filtered_portfolio();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filtered_portfolio_with_search() {
        let mut app = test_app();
        app.config.portfolios[0].holdings.push(Holding {
            symbol: "BBCA".to_string(),
            lots: 10,
            avg_price: 8000.0,
        });
        app.config.portfolios[0].holdings.push(Holding {
            symbol: "TLKM".to_string(),
            lots: 20,
            avg_price: 3000.0,
        });
        app.search_active = true;
        app.search_query = "BB".to_string();
        let filtered = app.get_filtered_portfolio();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.symbol, "BBCA");
    }

    // --- get_filtered_news ---

    #[test]
    fn test_filtered_news_search_by_title() {
        let mut app = test_app();
        app.news_items
            .push(make_news_item("BBCA naik tajam", "CNBC", 1000));
        app.news_items
            .push(make_news_item("IHSG melemah", "Tempo", 2000));
        app.search_active = true;
        app.search_query = "BBCA".to_string();
        let filtered = app.get_filtered_news();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "BBCA naik tajam");
    }

    #[test]
    fn test_filtered_news_search_by_publisher() {
        let mut app = test_app();
        app.news_items
            .push(make_news_item("Saham naik", "CNBC Indonesia", 1000));
        app.news_items
            .push(make_news_item("Saham turun", "Tempo", 2000));
        app.search_active = true;
        app.search_query = "CNBC".to_string();
        let filtered = app.get_filtered_news();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].publisher, "CNBC Indonesia");
    }

    // --- selected_*_symbol ---

    #[test]
    fn test_selected_watchlist_symbol() {
        let mut app = test_app();
        app.selected_index = 1;
        assert_eq!(app.selected_watchlist_symbol(), Some("BBRI".to_string()));
    }

    #[test]
    fn test_selected_watchlist_symbol_empty() {
        let mut app = test_app();
        app.config.watchlists[0].symbols.clear();
        assert_eq!(app.selected_watchlist_symbol(), None);
    }

    #[test]
    fn test_selected_portfolio_symbol() {
        let mut app = test_app();
        app.config.portfolios[0].holdings.push(Holding {
            symbol: "BBCA".to_string(),
            lots: 10,
            avg_price: 8000.0,
        });
        app.portfolio_selected = 0;
        assert_eq!(app.selected_portfolio_symbol(), Some("BBCA".to_string()));
    }
}
