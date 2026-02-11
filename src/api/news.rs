use crate::api::yahoo::NewsItem;
use anyhow::Result;
use reqwest::Client;

pub struct NewsClient {
    client: Client,
}

impl NewsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build RSS client");
        Self { client }
    }

    async fn fetch_feed(&self, url: &str) -> Result<Vec<NewsItem>> {
        let bytes = self
            .client
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (compatible; idx-cli/0.1)",
            )
            .send()
            .await?
            .bytes()
            .await?;

        let feed = feed_rs::parser::parse(&bytes[..])?;

        let items = feed
            .entries
            .into_iter()
            .map(|entry| {
                let title = entry
                    .title
                    .map(|t| t.content)
                    .unwrap_or_else(|| "(no title)".to_string());
                let publisher = feed
                    .title
                    .as_ref()
                    .map(|t| t.content.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let link = entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_default();
                let published_at = entry
                    .published
                    .or(entry.updated)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);

                NewsItem {
                    title,
                    publisher,
                    link,
                    published_at,
                }
            })
            .collect();

        Ok(items)
    }

    pub async fn fetch_all(&self, urls: &[String]) -> Result<Vec<NewsItem>> {
        let futures: Vec<_> = urls.iter().map(|url| self.fetch_feed(url)).collect();
        let results = futures::future::join_all(futures).await;
        let mut all_items: Vec<NewsItem> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        Ok(all_items)
    }
}

impl Default for NewsClient {
    fn default() -> Self {
        Self::new()
    }
}
