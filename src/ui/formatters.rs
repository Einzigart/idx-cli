pub fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        let int_part = price as u64;
        int_part
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(",")
    } else {
        format!("{:.2}", price)
    }
}

pub fn format_change(change: f64) -> String {
    if change >= 0.0 {
        format!("+{:.0}", change)
    } else {
        format!("{:.0}", change)
    }
}

pub fn format_compact(value: f64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000_000_000.0 {
        format!("{:.2}T", abs / 1_000_000_000_000.0)
    } else if abs >= 1_000_000_000.0 {
        format!("{:.2}B", abs / 1_000_000_000.0)
    } else if abs >= 1_000_000.0 {
        format!("{:.2}M", abs / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.2}K", abs / 1_000.0)
    } else {
        format!("{:.0}", abs)
    }
}

pub fn format_pl(pl: f64) -> String {
    let prefix = if pl >= 0.0 { "+" } else { "-" };
    format!("{}{}", prefix, format_compact(pl))
}

pub fn format_volume(volume: u64) -> String {
    format_compact(volume as f64)
}

pub fn format_value(value: f64) -> String {
    format_compact(value)
}

pub fn format_market_cap(cap: u64) -> String {
    format_compact(cap as f64)
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let end = s
            .char_indices()
            .nth(max_len - 3)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}...", &s[..end])
    } else {
        s.to_string()
    }
}

pub fn format_relative_time(unix_ts: i64) -> String {
    if unix_ts <= 0 {
        return String::new();
    }
    let elapsed_secs = chrono::Utc::now().timestamp() - unix_ts;
    if elapsed_secs < 0 {
        return "just now".to_string();
    }
    let mins = elapsed_secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    match () {
        _ if days > 0 => format!("{}d ago", days),
        _ if hours > 0 => format!("{}h ago", hours),
        _ if mins > 0 => format!("{}m ago", mins),
        _ => "just now".to_string(),
    }
}
