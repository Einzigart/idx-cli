use idx_cli::config::{Alert, AlertType, Bookmark, Config};

fn test_config() -> Config {
    Config::test_config()
}

#[test]
fn add_holding_overflow_returns_false() {
    let mut config = test_config();
    config.add_holding("BBCA", 3_000_000_000, 8000.0);
    // Adding 2 billion more would exceed u32::MAX (4,294,967,295)
    let ok = config.add_holding("BBCA", 2_000_000_000, 9000.0);
    assert!(!ok, "add_holding should return false on u32 overflow");
    // Original holding should be unchanged
    let h = config
        .current_portfolio()
        .holdings
        .iter()
        .find(|h| h.symbol == "BBCA")
        .unwrap();
    assert_eq!(h.lots, 3_000_000_000);
    assert_eq!(h.avg_price, 8000.0);
}

#[test]
fn add_holding_normal_merge() {
    let mut config = test_config();
    config.add_holding("BBCA", 100, 8000.0);
    let ok = config.add_holding("BBCA", 100, 9000.0);
    assert!(ok);
    let h = config
        .current_portfolio()
        .holdings
        .iter()
        .find(|h| h.symbol == "BBCA")
        .unwrap();
    assert_eq!(h.lots, 200);
    // Weighted avg: (100*100*8000 + 100*100*9000) / (200*100) = 8500
    assert!((h.avg_price - 8500.0).abs() < 0.01);
}

#[test]
fn add_holding_new_symbol() {
    let mut config = test_config();
    let ok = config.add_holding("TLKM", 50, 3500.0);
    assert!(ok);
    assert_eq!(config.current_portfolio().holdings.len(), 1);
    assert_eq!(config.current_portfolio().holdings[0].symbol, "TLKM");
    assert_eq!(config.current_portfolio().holdings[0].lots, 50);
}

#[test]
fn migrate_flat_portfolio_to_portfolios() {
    let json = r#"{
        "watchlists": [{"name": "Default", "symbols": ["BBCA"]}],
        "active_watchlist": 0,
        "portfolio": [
            {"symbol": "BBCA", "lots": 100, "avg_price": 8000.0},
            {"symbol": "TLKM", "lots": 50, "avg_price": 3500.0}
        ]
    }"#;
    let mut config: Config = serde_json::from_str(json).unwrap();
    config.migrate_portfolio();
    assert_eq!(config.portfolios.len(), 1);
    assert_eq!(config.portfolios[0].name, "Default");
    assert_eq!(config.portfolios[0].holdings.len(), 2);
    assert_eq!(config.portfolios[0].holdings[0].symbol, "BBCA");
}

#[test]
fn new_format_loads_directly() {
    let json = r#"{
        "watchlists": [{"name": "Default", "symbols": ["BBCA"]}],
        "active_watchlist": 0,
        "portfolios": [
            {"name": "Growth", "holdings": [{"symbol": "BBCA", "lots": 100, "avg_price": 8000.0}]},
            {"name": "Dividend", "holdings": [{"symbol": "TLKM", "lots": 50, "avg_price": 3500.0}]}
        ],
        "active_portfolio": 1
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.portfolios.len(), 2);
    assert_eq!(config.active_portfolio, 1);
    assert_eq!(config.portfolios[1].name, "Dividend");
}

#[test]
fn portfolio_crud_operations() {
    let mut config = test_config();
    assert_eq!(config.portfolios.len(), 1);

    config.add_portfolio("Growth");
    assert_eq!(config.portfolios.len(), 2);
    assert_eq!(config.active_portfolio, 1);
    assert_eq!(config.current_portfolio().name, "Growth");

    config.rename_portfolio("Aggressive Growth");
    assert_eq!(config.current_portfolio().name, "Aggressive Growth");

    config.next_portfolio();
    assert_eq!(config.active_portfolio, 0);
    config.prev_portfolio();
    assert_eq!(config.active_portfolio, 1);

    config.remove_portfolio();
    assert_eq!(config.portfolios.len(), 1);
    assert_eq!(config.active_portfolio, 0);

    config.remove_portfolio();
    assert_eq!(config.portfolios.len(), 1);
}

#[test]
fn alert_above_fires_when_price_meets_threshold() {
    let alert = Alert::new("BBCA", AlertType::Above, 8000.0);
    assert!(alert.should_trigger(8000.0, 0.0));
    assert!(alert.should_trigger(8001.0, 0.0));
    assert!(!alert.should_trigger(7999.0, 0.0));
}

#[test]
fn alert_below_fires_when_price_meets_threshold() {
    let alert = Alert::new("BBCA", AlertType::Below, 8000.0);
    assert!(alert.should_trigger(8000.0, 0.0));
    assert!(alert.should_trigger(7999.0, 0.0));
    assert!(!alert.should_trigger(8001.0, 0.0));
}

#[test]
fn alert_pct_gain_fires_when_change_meets_threshold() {
    let alert = Alert::new("BBCA", AlertType::PercentGain, 5.0);
    assert!(alert.should_trigger(8000.0, 5.0));
    assert!(alert.should_trigger(8000.0, 6.0));
    assert!(!alert.should_trigger(8000.0, 4.0));
}

#[test]
fn alert_pct_loss_fires_when_change_meets_threshold() {
    let alert = Alert::new("BBCA", AlertType::PercentLoss, 5.0);
    assert!(alert.should_trigger(8000.0, -5.0));
    assert!(alert.should_trigger(8000.0, -6.0));
    assert!(!alert.should_trigger(8000.0, -4.0));
}

#[test]
fn alert_disabled_does_not_fire() {
    let mut alert = Alert::new("BBCA", AlertType::Above, 8000.0);
    alert.enabled = false;
    assert!(!alert.should_trigger(8001.0, 0.0));
}

#[test]
fn alert_respects_cooldown() {
    let mut alert = Alert::new("BBCA", AlertType::Above, 8000.0);
    alert.cooldown_seconds = 1000;
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    alert.last_triggered = Some(now);
    assert!(!alert.should_trigger(8001.0, 0.0));
}

#[test]
fn config_add_remove_toggle_alerts() {
    let mut config = test_config();
    let alert = Alert::new("BBCA", AlertType::Above, 8000.0);
    let id = alert.id.clone();
    config.add_alert(alert);
    assert_eq!(config.alerts.len(), 1);
    assert!(config.has_active_alerts("BBCA"));

    config.toggle_alert(&id);
    assert!(!config.has_active_alerts("BBCA"));

    config.toggle_alert(&id);
    assert!(config.has_active_alerts("BBCA"));

    config.remove_alert(&id);
    assert_eq!(config.alerts.len(), 0);
}

#[test]
fn config_alerts_for_symbol_filters_correctly() {
    let mut config = test_config();
    config.add_alert(Alert::new("BBCA", AlertType::Above, 8000.0));
    config.add_alert(Alert::new("BBCA", AlertType::Below, 7000.0));
    config.add_alert(Alert::new("TLKM", AlertType::Above, 3500.0));

    let bbca_alerts = config.alerts_for_symbol("BBCA");
    assert_eq!(bbca_alerts.len(), 2);
    let tlkm_alerts = config.alerts_for_symbol("TLKM");
    assert_eq!(tlkm_alerts.len(), 1);
}

#[test]
fn alert_type_cycles() {
    let mut at = AlertType::Above;
    at = at.next();
    assert_eq!(at, AlertType::Below);
    at = at.next();
    assert_eq!(at, AlertType::PercentGain);
    at = at.next();
    assert_eq!(at, AlertType::PercentLoss);
    at = at.next();
    assert_eq!(at, AlertType::Above);
}

fn make_test_bookmark(headline: &str, url: Option<&str>) -> Bookmark {
    Bookmark {
        id: format!("bm_{}", headline.len()),
        headline: headline.to_string(),
        source: "TestSource".to_string(),
        url: url.map(|u| u.to_string()),
        published_at: 1000,
        bookmarked_at: 2000,
        read: false,
    }
}

#[test]
fn bookmark_add_and_duplicate_prevention() {
    let mut config = test_config();
    let bm = make_test_bookmark("BBCA rises", Some("https://a.com"));
    assert!(config.add_bookmark(bm));
    assert_eq!(config.bookmarks.len(), 1);

    let dup = make_test_bookmark("BBCA rises", Some("https://a.com"));
    assert!(!config.add_bookmark(dup));
    assert_eq!(config.bookmarks.len(), 1);
}

#[test]
fn bookmark_remove_by_index() {
    let mut config = test_config();
    config.add_bookmark(make_test_bookmark("A", None));
    config.add_bookmark(make_test_bookmark("B", None));
    assert_eq!(config.bookmarks.len(), 2);

    config.remove_bookmark(0);
    assert_eq!(config.bookmarks.len(), 1);
    assert_eq!(config.bookmarks[0].headline, "B");
}

#[test]
fn bookmark_clear_all() {
    let mut config = test_config();
    config.add_bookmark(make_test_bookmark("X", None));
    config.add_bookmark(make_test_bookmark("Y", None));
    config.clear_bookmarks();
    assert!(config.bookmarks.is_empty());
}

#[test]
fn bookmark_toggle_and_mark_read() {
    let mut config = test_config();
    config.add_bookmark(make_test_bookmark("Test", None));
    assert!(!config.bookmarks[0].read);

    config.toggle_bookmark_read(0);
    assert!(config.bookmarks[0].read);

    config.toggle_bookmark_read(0);
    assert!(!config.bookmarks[0].read);

    config.mark_bookmark_read(0);
    assert!(config.bookmarks[0].read);

    // mark_bookmark_read is idempotent
    config.mark_bookmark_read(0);
    assert!(config.bookmarks[0].read);
}

#[test]
fn bookmark_is_bookmarked_query() {
    let mut config = test_config();
    config.add_bookmark(make_test_bookmark("Headline", Some("https://url.com")));

    assert!(config.is_bookmarked("Headline", Some("https://url.com")));
    assert!(!config.is_bookmarked("Headline", Some("https://other.com")));
    assert!(!config.is_bookmarked("Other", Some("https://url.com")));
    assert!(!config.is_bookmarked("Headline", None));
}

#[test]
fn bookmark_deserialization_with_default() {
    let json = r#"{
        "watchlists": [{"name": "Default", "symbols": ["BBCA"]}],
        "active_watchlist": 0
    }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert!(config.bookmarks.is_empty());
}
