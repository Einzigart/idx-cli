pub fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        let rounded = (price * 100.0).round() / 100.0;
        let int_part = rounded as u64;
        let formatted_int: String = int_part
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(",");
        let frac = rounded - int_part as f64;
        if frac.abs() >= 0.005 {
            format!("{}.{:02}", formatted_int, (frac * 100.0).round() as u64)
        } else {
            formatted_int
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price_rounding_bug() {
        // Issue #1: frac >= 0.995 caused three-digit fractional part
        assert_eq!(format_price(1500.999), "1,501");
        assert_eq!(format_price(1500.995), "1,501");
    }

    #[test]
    fn test_format_price_normal_cases() {
        assert_eq!(format_price(1500.50), "1,500.50");
        assert_eq!(format_price(1500.0), "1,500");
        assert_eq!(format_price(1000.01), "1,000.01");
        assert_eq!(format_price(999.99), "999.99");
        assert_eq!(format_price(50.0), "50.00");
    }

    // --- format_change ---

    #[test]
    fn test_format_change_positive() {
        assert_eq!(format_change(125.7), "+126");
    }

    #[test]
    fn test_format_change_negative() {
        assert_eq!(format_change(-42.3), "-42");
    }

    #[test]
    fn test_format_change_zero() {
        assert_eq!(format_change(0.0), "+0");
    }

    #[test]
    fn test_format_change_small_negative() {
        assert_eq!(format_change(-0.4), "-0");
    }

    // --- format_compact ---

    #[test]
    fn test_format_compact_trillions() {
        assert_eq!(format_compact(2_500_000_000_000.0), "2.50T");
    }

    #[test]
    fn test_format_compact_billions() {
        assert_eq!(format_compact(1_230_000_000.0), "1.23B");
    }

    #[test]
    fn test_format_compact_millions() {
        assert_eq!(format_compact(45_600_000.0), "45.60M");
    }

    #[test]
    fn test_format_compact_thousands() {
        assert_eq!(format_compact(7_890.0), "7.89K");
    }

    #[test]
    fn test_format_compact_small() {
        assert_eq!(format_compact(999.0), "999");
    }

    #[test]
    fn test_format_compact_zero() {
        assert_eq!(format_compact(0.0), "0");
    }

    #[test]
    fn test_format_compact_boundary_million() {
        assert_eq!(format_compact(1_000_000.0), "1.00M");
    }

    #[test]
    fn test_format_compact_negative_uses_abs() {
        assert_eq!(format_compact(-5_000_000.0), "5.00M");
    }

    // --- format_pl ---

    #[test]
    fn test_format_pl_positive() {
        assert_eq!(format_pl(5_000_000.0), "+5.00M");
    }

    #[test]
    fn test_format_pl_negative() {
        assert_eq!(format_pl(-1_230_000.0), "-1.23M");
    }

    #[test]
    fn test_format_pl_zero() {
        assert_eq!(format_pl(0.0), "+0");
    }

    // --- format_volume ---

    #[test]
    fn test_format_volume_large() {
        assert_eq!(format_volume(123_456_789), "123.46M");
    }

    #[test]
    fn test_format_volume_zero() {
        assert_eq!(format_volume(0), "0");
    }

    // --- truncate_str ---

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate_str("Hello", 10), "Hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate_str("Hello", 5), "Hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate_str("Hello World", 8), "Hello...");
    }

    #[test]
    fn test_truncate_very_short_max() {
        assert_eq!(truncate_str("ABCDEFG", 4), "A...");
    }

    // --- format_relative_time ---

    #[test]
    fn test_relative_time_zero_ts() {
        assert_eq!(format_relative_time(0), "");
    }

    #[test]
    fn test_relative_time_negative_ts() {
        assert_eq!(format_relative_time(-100), "");
    }

    #[test]
    fn test_relative_time_future() {
        let future_ts = chrono::Utc::now().timestamp() + 3600;
        assert_eq!(format_relative_time(future_ts), "just now");
    }

    #[test]
    fn test_relative_time_just_now() {
        let recent_ts = chrono::Utc::now().timestamp() - 30;
        assert_eq!(format_relative_time(recent_ts), "just now");
    }

    #[test]
    fn test_relative_time_minutes() {
        let ts = chrono::Utc::now().timestamp() - (5 * 60);
        assert_eq!(format_relative_time(ts), "5m ago");
    }

    #[test]
    fn test_relative_time_hours() {
        let ts = chrono::Utc::now().timestamp() - (3 * 3600);
        assert_eq!(format_relative_time(ts), "3h ago");
    }

    #[test]
    fn test_relative_time_days() {
        let ts = chrono::Utc::now().timestamp() - (2 * 86400);
        assert_eq!(format_relative_time(ts), "2d ago");
    }
}
