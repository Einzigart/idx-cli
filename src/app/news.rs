use super::App;

impl App {
    pub async fn refresh_news(&mut self) {
        self.rss_loading = true;
        let urls = self.config.news_sources.clone();
        match self.news_client.fetch_all(&urls).await {
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
}
