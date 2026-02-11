use anyhow::{anyhow, Result};
use reqwest::{cookie::Jar, Client};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

const YAHOO_BASE_URL: &str = "https://finance.yahoo.com";
const YAHOO_QUOTE_URL: &str = "https://query1.finance.yahoo.com/v7/finance/quote";
const YAHOO_CHART_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart";
const YAHOO_SEARCH_URL: &str = "https://query2.finance.yahoo.com/v1/finance/search";

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub short_name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub prev_close: f64,
    // Company classification
    pub long_name: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    // Fundamentals
    pub market_cap: Option<u64>,
    pub trailing_pe: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    // Risk & liquidity
    pub beta: Option<f64>,
    pub average_volume: Option<u64>,
}

/// Historical price data for sparkline chart
#[derive(Debug, Clone)]
pub struct ChartData {
    pub closes: Vec<f64>,
    pub high: f64,
    pub low: f64,
}

// Chart API response structures
#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: ChartResult,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    result: Option<Vec<ChartResultItem>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChartResultItem {
    indicators: ChartIndicators,
}

#[derive(Debug, Deserialize)]
struct ChartIndicators {
    quote: Vec<ChartQuote>,
}

#[derive(Debug, Deserialize)]
struct ChartQuote {
    close: Option<Vec<Option<f64>>>,
}

/// A news article from Yahoo Finance search results or RSS feeds
#[derive(Debug, Clone)]
pub struct NewsItem {
    pub title: String,
    pub publisher: String,
    pub published_at: i64, // Unix timestamp
}

// Search API response structures (for news)
#[derive(Debug, Deserialize)]
struct SearchResponse {
    news: Option<Vec<SearchNewsItem>>,
}

#[derive(Debug, Deserialize)]
struct SearchNewsItem {
    title: Option<String>,
    publisher: Option<String>,
    #[serde(rename = "providerPublishTime", default)]
    provider_publish_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct YahooResponse {
    #[serde(rename = "quoteResponse")]
    quote_response: QuoteResponse,
}

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    result: Vec<QuoteResult>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct QuoteResult {
    symbol: String,
    #[serde(rename = "shortName", default)]
    short_name: Option<String>,
    #[serde(rename = "regularMarketPrice", default)]
    regular_market_price: Option<f64>,
    #[serde(rename = "regularMarketChange", default)]
    regular_market_change: Option<f64>,
    #[serde(rename = "regularMarketChangePercent", default)]
    regular_market_change_percent: Option<f64>,
    #[serde(rename = "regularMarketOpen", default)]
    regular_market_open: Option<f64>,
    #[serde(rename = "regularMarketDayHigh", default)]
    regular_market_day_high: Option<f64>,
    #[serde(rename = "regularMarketDayLow", default)]
    regular_market_day_low: Option<f64>,
    #[serde(rename = "regularMarketVolume", default)]
    regular_market_volume: Option<u64>,
    #[serde(rename = "regularMarketPreviousClose", default)]
    regular_market_previous_close: Option<f64>,
    // Company classification
    #[serde(rename = "longName", default)]
    long_name: Option<String>,
    #[serde(default)]
    sector: Option<String>,
    #[serde(default)]
    industry: Option<String>,
    // Fundamentals
    #[serde(rename = "marketCap", default)]
    market_cap: Option<u64>,
    #[serde(rename = "trailingPE", default)]
    trailing_pe: Option<f64>,
    #[serde(rename = "dividendYield", default)]
    dividend_yield: Option<f64>,
    #[serde(rename = "fiftyTwoWeekHigh", default)]
    fifty_two_week_high: Option<f64>,
    #[serde(rename = "fiftyTwoWeekLow", default)]
    fifty_two_week_low: Option<f64>,
    // Risk & liquidity
    #[serde(default)]
    beta: Option<f64>,
    #[serde(rename = "averageVolume", default)]
    average_volume: Option<u64>,
}

impl From<QuoteResult> for StockQuote {
    fn from(q: QuoteResult) -> Self {
        // Remove .JK suffix for display
        let display_symbol = q.symbol.trim_end_matches(".JK").to_string();

        StockQuote {
            symbol: display_symbol,
            short_name: q.short_name.unwrap_or_else(|| "N/A".to_string()),
            price: q.regular_market_price.unwrap_or(0.0),
            change: q.regular_market_change.unwrap_or(0.0),
            change_percent: q.regular_market_change_percent.unwrap_or(0.0),
            open: q.regular_market_open.unwrap_or(0.0),
            high: q.regular_market_day_high.unwrap_or(0.0),
            low: q.regular_market_day_low.unwrap_or(0.0),
            volume: q.regular_market_volume.unwrap_or(0),
            prev_close: q.regular_market_previous_close.unwrap_or(0.0),
            // Company classification
            long_name: q.long_name,
            sector: q.sector,
            industry: q.industry,
            // Fundamentals
            market_cap: q.market_cap,
            trailing_pe: q.trailing_pe,
            dividend_yield: q.dividend_yield,
            fifty_two_week_high: q.fifty_two_week_high,
            fifty_two_week_low: q.fifty_two_week_low,
            // Risk & liquidity
            beta: q.beta,
            average_volume: q.average_volume,
        }
    }
}

pub struct YahooClient {
    client: Client,
    crumb: Option<String>,
}

impl YahooClient {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(jar)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            crumb: None,
        }
    }

    /// Fetch crumb and cookies from Yahoo Finance
    async fn fetch_crumb(&mut self) -> Result<String> {
        // First, get cookies by visiting the main page
        let response = self
            .client
            .get(YAHOO_BASE_URL)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await?;

        let html = response.text().await?;

        // Extract crumb from the page
        // Look for pattern like "crumb":"XXXXX" in the HTML/JS
        let crumb = Self::extract_crumb(&html)?;
        self.crumb = Some(crumb.clone());
        Ok(crumb)
    }

    fn extract_crumb(html: &str) -> Result<String> {
        // Try multiple patterns to find the crumb

        // Pattern 1: "crumb":"value"
        if let Some(start) = html.find("\"crumb\":\"") {
            let start = start + 9;
            if let Some(end) = html[start..].find('"') {
                let crumb = &html[start..start + end];
                if !crumb.is_empty() {
                    return Ok(crumb.to_string());
                }
            }
        }

        // Pattern 2: "CrsrfToken":"value"
        if let Some(start) = html.find("\"CrumbStore\":{\"crumb\":\"") {
            let start = start + 23;
            if let Some(end) = html[start..].find('"') {
                let crumb = &html[start..start + end];
                if !crumb.is_empty() {
                    return Ok(crumb.to_string());
                }
            }
        }

        Err(anyhow!("Could not extract crumb from Yahoo Finance"))
    }

    /// Convert IDX stock code to Yahoo Finance symbol (add .JK suffix)
    fn to_yahoo_symbol(code: &str) -> String {
        let code = code.to_uppercase();
        if code.ends_with(".JK") {
            code
        } else {
            format!("{}.JK", code)
        }
    }

    /// Fetch quotes for multiple stocks
    pub async fn get_quotes(&mut self, symbols: &[String]) -> Result<HashMap<String, StockQuote>> {
        if symbols.is_empty() {
            return Ok(HashMap::new());
        }

        // Get crumb if we don't have one
        let crumb = match &self.crumb {
            Some(c) => c.clone(),
            None => self.fetch_crumb().await?,
        };

        let yahoo_symbols: Vec<String> = symbols.iter().map(|s| Self::to_yahoo_symbol(s)).collect();
        let symbols_param = yahoo_symbols.join(",");

        let response = self
            .client
            .get(YAHOO_QUOTE_URL)
            .query(&[("symbols", &symbols_param), ("crumb", &crumb)])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/")
            .send()
            .await?;

        // If unauthorized, try refreshing crumb
        if response.status() == 401 {
            self.crumb = None;
            let new_crumb = self.fetch_crumb().await?;

            let response = self
                .client
                .get(YAHOO_QUOTE_URL)
                .query(&[("symbols", &symbols_param), ("crumb", &new_crumb)])
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .header("Accept", "application/json")
                .header("Referer", "https://finance.yahoo.com/")
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Yahoo API error: {}", response.status()));
            }

            let data: YahooResponse = response.json().await?;
            return Self::parse_response(data);
        }

        if !response.status().is_success() {
            return Err(anyhow!("Yahoo API error: {}", response.status()));
        }

        let data: YahooResponse = response.json().await?;
        Self::parse_response(data)
    }

    fn parse_response(data: YahooResponse) -> Result<HashMap<String, StockQuote>> {
        if let Some(err) = data.quote_response.error {
            return Err(anyhow!("Yahoo API error: {:?}", err));
        }

        let mut quotes = HashMap::new();
        for result in data.quote_response.result {
            let quote: StockQuote = result.into();
            quotes.insert(quote.symbol.clone(), quote);
        }

        Ok(quotes)
    }

    /// Fetch historical chart data for sparkline (3 months daily)
    pub async fn get_chart(&self, symbol: &str) -> Result<ChartData> {
        let yahoo_symbol = Self::to_yahoo_symbol(symbol);
        let url = format!("{}/{}", YAHOO_CHART_URL, yahoo_symbol);

        let response = self
            .client
            .get(&url)
            .query(&[("interval", "1d"), ("range", "3mo")])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Chart API error: {}", response.status()));
        }

        let data: ChartResponse = response.json().await?;

        if let Some(err) = data.chart.error {
            return Err(anyhow!("Chart API error: {:?}", err));
        }

        let result = data
            .chart
            .result
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| anyhow!("No chart data found"))?;

        let closes: Vec<f64> = result
            .indicators
            .quote
            .into_iter()
            .next()
            .and_then(|q| q.close)
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect();

        if closes.is_empty() {
            return Err(anyhow!("No price data in chart"));
        }

        let high = closes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let low = closes.iter().cloned().fold(f64::INFINITY, f64::min);

        Ok(ChartData { closes, high, low })
    }

    /// Fetch news articles for a stock symbol via Yahoo search API
    pub async fn get_news(&self, symbol: &str) -> Result<Vec<NewsItem>> {
        let yahoo_symbol = Self::to_yahoo_symbol(symbol);

        let response = self
            .client
            .get(YAHOO_SEARCH_URL)
            .query(&[
                ("q", yahoo_symbol.as_str()),
                ("newsCount", "8"),
                ("quotesCount", "0"),
            ])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .header("Referer", "https://finance.yahoo.com/")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("News API error: {}", response.status()));
        }

        let data: SearchResponse = response.json().await?;

        let items = data
            .news
            .unwrap_or_default()
            .into_iter()
            .filter_map(|n| {
                Some(NewsItem {
                    title: n.title?,
                    publisher: n.publisher.unwrap_or_else(|| "Unknown".to_string()),
                    published_at: n.provider_publish_time.unwrap_or(0),
                })
            })
            .collect();

        Ok(items)
    }
}

impl Default for YahooClient {
    fn default() -> Self {
        Self::new()
    }
}
