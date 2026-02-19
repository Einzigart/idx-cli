use chrono::Utc;

use super::{App, InputMode, title_contains_ticker};

impl App {
    pub fn has_recent_news(&self, symbol: &str) -> bool {
        let cutoff = Utc::now().timestamp() - 86_400;
        let sym = symbol.to_uppercase();
        self.news_items
            .iter()
            .any(|item| item.published_at >= cutoff && title_contains_ticker(&item.title, &sym))
    }

    /// Set `rss_loading = true` and return the feed URLs.
    pub fn prepare_news_refresh(&mut self) -> Vec<String> {
        self.rss_loading = true;
        self.config.news_sources.clone()
    }

    /// Execute the network fetch for news feeds and clear `rss_loading`.
    pub async fn execute_news_refresh(&mut self, urls: &[String]) {
        match self.news_client.fetch_all(urls).await {
            Ok(items) => {
                self.news_items = items;
                self.news_last_refresh = Some(tokio::time::Instant::now());
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("News error: {}", e));
            }
        }
        self.rss_loading = false;
    }

    /// Open the news detail modal for the currently selected news item.
    pub fn open_news_detail(&mut self) {
        let items = self.get_filtered_news();
        if items.get(self.news_selected).is_none() {
            return;
        }
        self.input_mode = InputMode::NewsDetail;
        self.news_detail_scroll = 0;
    }
}
