use super::SortDirection;
use crate::api::{NewsItem, StockQuote};
use crate::config::{Bookmark, Holding};
use std::cmp::Ordering;
use std::collections::HashMap;

pub fn cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

pub fn compare_watchlist_column(
    col: usize,
    a: &(&String, Option<&StockQuote>),
    b: &(&String, Option<&StockQuote>),
    direction: SortDirection,
) -> Ordering {
    match (a.1, b.1) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(qa), Some(qb)) => {
            let ord = match col {
                0 => qa.symbol.cmp(&qb.symbol),
                1 => qa.short_name.cmp(&qb.short_name),
                2 => cmp_f64(qa.price, qb.price),
                3 => cmp_f64(qa.change, qb.change),
                4 => cmp_f64(qa.change_percent, qb.change_percent),
                5 => cmp_f64(qa.open, qb.open),
                6 => cmp_f64(qa.high, qb.high),
                7 => cmp_f64(qa.low, qb.low),
                8 => qa.volume.cmp(&qb.volume),
                9 => cmp_f64(qa.price * qa.volume as f64, qb.price * qb.volume as f64),
                _ => Ordering::Equal,
            };
            match direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        }
    }
}

pub fn compare_portfolio_column(
    col: usize,
    a: &Holding,
    b: &Holding,
    quotes: &HashMap<String, StockQuote>,
) -> Ordering {
    let price_a = quotes.get(&a.symbol).map(|q| q.price).unwrap_or(0.0);
    let price_b = quotes.get(&b.symbol).map(|q| q.price).unwrap_or(0.0);
    let name_a = quotes
        .get(&a.symbol)
        .map(|q| q.short_name.as_str())
        .unwrap_or("");
    let name_b = quotes
        .get(&b.symbol)
        .map(|q| q.short_name.as_str())
        .unwrap_or("");
    match col {
        0 => a.symbol.cmp(&b.symbol),
        1 => name_a.cmp(name_b),
        2 => a.lots.cmp(&b.lots),
        3 => cmp_f64(a.avg_price, b.avg_price),
        4 => cmp_f64(price_a, price_b),
        5 => cmp_f64(a.pl_metrics(price_a).0, b.pl_metrics(price_b).0),
        6 => cmp_f64(a.cost_basis(), b.cost_basis()),
        7 => cmp_f64(a.pl_metrics(price_a).2, b.pl_metrics(price_b).2),
        8 => cmp_f64(a.pl_metrics(price_a).3, b.pl_metrics(price_b).3),
        _ => Ordering::Equal,
    }
}

pub fn compare_news_column(col: usize, a: &NewsItem, b: &NewsItem) -> Ordering {
    match col {
        0 => a.published_at.cmp(&b.published_at),
        1 => a.publisher.cmp(&b.publisher),
        2 => a.title.cmp(&b.title),
        _ => Ordering::Equal,
    }
}

/// Columns: 0=Bookmarked, 1=Published, 2=Source, 3=Headline
pub fn compare_bookmark_column(col: usize, a: &Bookmark, b: &Bookmark) -> Ordering {
    match col {
        0 => a.bookmarked_at.cmp(&b.bookmarked_at),
        1 => a.published_at.cmp(&b.published_at),
        2 => a.source.cmp(&b.source),
        3 => a.headline.cmp(&b.headline),
        _ => Ordering::Equal,
    }
}
