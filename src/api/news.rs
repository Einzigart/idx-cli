use crate::api::yahoo::NewsItem;
use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// Extract a short publisher name from the feed URL's domain.
/// e.g. "https://www.cnbcindonesia.com/market/rss" → "CNBC Indonesia"
fn publisher_from_url(url: &str) -> String {
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .trim_start_matches("www.")
        .trim_start_matches("rss.")
        .trim_start_matches("feeds.");

    match host {
        "cnbcindonesia.com" => "CNBC Indonesia".to_string(),        "idxchannel.com" => "IDX Channel".to_string(),
        "tempo.co" => "Tempo".to_string(),
        "kontan.co.id" => "Kontan".to_string(),
        "feedburner.com" => "Feedburner".to_string(),
        // Fallback: use domain as-is, dropping TLD
        other => other
            .split('.')
            .next()
            .unwrap_or(other)
            .to_string(),
    }
}

pub struct NewsClient {
    client: Client,
}

impl NewsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
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
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .send()
            .await?
            .bytes()
            .await?;

        let feed = feed_rs::parser::parse(&bytes[..])?;

        let publisher = publisher_from_url(url);

        let items = feed
            .entries
            .into_iter()
            .map(|entry| {
                let title = entry
                    .title
                    .map(|t| t.content)
                    .unwrap_or_else(|| "(no title)".to_string());
                let published_at = entry
                    .published
                    .or(entry.updated)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);
                let url = entry.links.into_iter().next().map(|l| l.href);
                let summary = entry.summary.map(|s| s.content);

                NewsItem {
                    title,
                    publisher: publisher.clone(),
                    published_at,
                    url,
                    summary,
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
