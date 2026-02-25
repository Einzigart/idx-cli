mod common;

use common::{make_news_item, test_app};
use idx_cli::app::{InputMode, NewsTab, ViewMode};
use idx_cli::config::Bookmark;

fn make_bookmark(headline: &str, source: &str, url: Option<&str>) -> Bookmark {
    use std::sync::atomic::{AtomicI64, Ordering};
    static COUNTER: AtomicI64 = AtomicI64::new(3000);
    let ts = COUNTER.fetch_add(1, Ordering::Relaxed);
    Bookmark {
        id: format!("bm_test_{}", ts),
        headline: headline.to_string(),
        source: source.to_string(),
        url: url.map(|u| u.to_string()),
        published_at: 1000,
        bookmarked_at: ts,
        read: false,
    }
}

#[test]
fn toggle_bookmark_adds_and_removes() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_items
        .push(make_news_item("BBCA rises 5%", "CNBC", 1000));
    app.news_selected = 0;

    // Add bookmark
    app.toggle_news_bookmark();
    assert_eq!(app.config.bookmarks.len(), 1);
    assert_eq!(app.config.bookmarks[0].headline, "BBCA rises 5%");

    // Remove bookmark
    app.toggle_news_bookmark();
    assert!(app.config.bookmarks.is_empty());
}

#[test]
fn bookmark_prevents_duplicates() {
    let mut app = test_app();
    let bm = make_bookmark("Test headline", "CNBC", Some("https://example.com"));
    assert!(app.config.add_bookmark(bm));
    assert_eq!(app.config.bookmarks.len(), 1);

    // Duplicate should be rejected
    let bm2 = make_bookmark("Test headline", "CNBC", Some("https://example.com"));
    assert!(!app.config.add_bookmark(bm2));
    assert_eq!(app.config.bookmarks.len(), 1);
}

#[test]
fn remove_selected_bookmark_adjusts_index() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_tab = NewsTab::Bookmarks;
    app.config.bookmarks.push(make_bookmark("First", "A", None));
    app.config
        .bookmarks
        .push(make_bookmark("Second", "B", None));
    app.config.bookmarks.push(make_bookmark("Third", "C", None));
    assert_eq!(app.config.bookmarks.len(), 3);

    // Default sort is bookmarked_at descending, so index 2 = oldest ("First")
    // Select the last visible item and remove it
    app.bookmark_selected = 2;
    app.remove_selected_bookmark();
    assert_eq!(app.config.bookmarks.len(), 2);
    // Index should clamp down since we removed the last item
    assert!(app.bookmark_selected <= 1);
}

#[test]
fn clear_bookmarks_empties_list() {
    let mut app = test_app();
    app.config.bookmarks.push(make_bookmark("A", "X", None));
    app.config.bookmarks.push(make_bookmark("B", "Y", None));

    app.start_clear_bookmarks();
    assert_eq!(app.input_mode, InputMode::BookmarkClearConfirm);

    app.confirm_clear_bookmarks();
    assert!(app.config.bookmarks.is_empty());
    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.bookmark_selected, 0);
}

#[test]
fn toggle_read_flips_status() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_tab = NewsTab::Bookmarks;
    app.config.bookmarks.push(make_bookmark("Test", "S", None));
    assert!(!app.config.bookmarks[0].read);

    app.bookmark_selected = 0;
    app.toggle_selected_bookmark_read();
    assert!(app.config.bookmarks[0].read);

    app.toggle_selected_bookmark_read();
    assert!(!app.config.bookmarks[0].read);
}

#[test]
fn is_bookmarked_checks_headline_and_url() {
    let mut app = test_app();
    app.config
        .bookmarks
        .push(make_bookmark("Headline A", "Source", Some("https://a.com")));

    // Exact match should be found
    assert!(
        app.config
            .is_bookmarked("Headline A", Some("https://a.com"))
    );

    // Same headline, different URL: not a match
    assert!(
        !app.config
            .is_bookmarked("Headline A", Some("https://b.com"))
    );

    // Different headline, same URL: not a match
    assert!(
        !app.config
            .is_bookmarked("Headline B", Some("https://a.com"))
    );

    // Neither matches
    assert!(
        !app.config
            .is_bookmarked("Headline B", Some("https://b.com"))
    );

    // None URL vs Some URL: not a match
    assert!(!app.config.is_bookmarked("Headline A", None));
}

#[test]
fn news_tab_toggle_cycle() {
    let mut app = test_app();
    assert_eq!(app.view_mode, ViewMode::Watchlist);
    app.toggle_view(); // -> Portfolio
    app.toggle_view(); // -> News (Feed tab)
    assert_eq!(app.view_mode, ViewMode::News);
    assert_eq!(app.news_tab, NewsTab::Feed);
    app.toggle_news_tab(); // -> Bookmarks tab
    assert_eq!(app.news_tab, NewsTab::Bookmarks);
    app.toggle_news_tab(); // -> Feed tab
    assert_eq!(app.news_tab, NewsTab::Feed);
    app.toggle_view(); // -> Watchlist
    assert_eq!(app.view_mode, ViewMode::Watchlist);
}

#[test]
fn open_bookmark_detail_marks_read() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_tab = NewsTab::Bookmarks;
    app.config.bookmarks.push(make_bookmark("Test", "S", None));
    assert!(!app.config.bookmarks[0].read);

    app.bookmark_selected = 0;
    app.open_bookmark_detail();
    assert!(app.config.bookmarks[0].read);
    assert_eq!(app.input_mode, InputMode::BookmarkDetail);
}

#[test]
fn cancel_clear_bookmarks_preserves_data() {
    let mut app = test_app();
    app.config.bookmarks.push(make_bookmark("Keep", "S", None));
    app.start_clear_bookmarks();
    assert_eq!(app.input_mode, InputMode::BookmarkClearConfirm);

    app.cancel_clear_bookmarks();
    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.config.bookmarks.len(), 1);
}

#[test]
fn start_clear_bookmarks_no_op_when_empty() {
    let mut app = test_app();
    assert!(app.config.bookmarks.is_empty());
    app.start_clear_bookmarks();
    // Should stay in Normal mode since there's nothing to clear
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn remove_bookmark_clamps_to_filtered_list() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_tab = NewsTab::Bookmarks;
    // Add 3 bookmarks: "Alpha from X", "Beta from Y", "Alpha from Z"
    app.config
        .bookmarks
        .push(make_bookmark("Alpha headline", "X", None));
    app.config
        .bookmarks
        .push(make_bookmark("Beta headline", "Y", None));
    app.config
        .bookmarks
        .push(make_bookmark("Alpha headline again", "Z", None));

    // Search for "ALPHA" — filtered list should have 2 items
    app.search_active = true;
    app.search_query = "ALPHA".to_string();
    assert_eq!(app.get_filtered_bookmarks().len(), 2);

    // Select last filtered item and remove it
    app.bookmark_selected = 1;
    app.remove_selected_bookmark();

    // Selection should clamp to filtered length (now 1 item), not unfiltered (2 items)
    let filtered_len = app.get_filtered_bookmarks().len();
    assert!(
        app.bookmark_selected < filtered_len,
        "bookmark_selected ({}) should be < filtered len ({})",
        app.bookmark_selected,
        filtered_len
    );
}

#[test]
fn confirm_search_resets_bookmark_selected() {
    let mut app = test_app();
    app.view_mode = ViewMode::News;
    app.news_tab = NewsTab::Bookmarks;
    app.config
        .bookmarks
        .push(make_bookmark("News A", "X", None));
    app.config
        .bookmarks
        .push(make_bookmark("News B", "Y", None));

    // Simulate user scrolled down in bookmarks
    app.bookmark_selected = 1;
    *app.bookmark_table_state.offset_mut() = 1;

    // Now search for something
    app.input_mode = InputMode::Search;
    app.input_buffer = "news".to_string();
    app.confirm_search();

    // Both should be reset to 0
    assert_eq!(app.bookmark_selected, 0);
    assert_eq!(app.bookmark_table_state.offset(), 0);
}
