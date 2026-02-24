use super::{App, InputMode};
use crate::config::Bookmark;
use std::time::{SystemTime, UNIX_EPOCH};

impl App {
    /// Toggle bookmark on the currently selected news article.
    /// If already bookmarked, removes it. Otherwise, adds it.
    pub fn toggle_news_bookmark(&mut self) {
        let (headline, source, url, published_at) = {
            let items = self.get_filtered_news();
            let item = match items.get(self.news_selected) {
                Some(i) => *i,
                None => return,
            };
            (
                item.title.clone(),
                item.publisher.clone(),
                item.url.clone(),
                item.published_at,
            )
        };

        if self.config.is_bookmarked(&headline, url.as_deref()) {
            self.config
                .bookmarks
                .retain(|b| !(b.headline == headline && b.url == url));
            let _ = self.config.save();
            self.status_message = Some("Bookmark removed".to_string());
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let bookmark = Bookmark {
                id: format!("bm_{}", now),
                headline,
                source,
                url,
                published_at,
                bookmarked_at: now as i64,
                read: false,
            };
            self.config.add_bookmark(bookmark);
            let _ = self.config.save();
            self.status_message = Some("Article bookmarked".to_string());
        }
    }

    /// Remove the currently selected bookmark and adjust the selection index.
    pub fn remove_selected_bookmark(&mut self) {
        let filtered = self.get_filtered_bookmarks();
        if let Some(b) = filtered.get(self.bookmark_selected) {
            let id = b.id.clone();
            self.config.bookmarks.retain(|b| b.id != id);
            let _ = self.config.save();
            let len = self.config.bookmarks.len();
            if self.bookmark_selected >= len && len > 0 {
                self.bookmark_selected = len - 1;
            }
            self.status_message = Some("Bookmark removed".to_string());
        }
    }

    /// Enter the clear-all confirmation mode.
    pub fn start_clear_bookmarks(&mut self) {
        if !self.config.bookmarks.is_empty() {
            self.input_mode = InputMode::BookmarkClearConfirm;
        }
    }

    /// Confirm clearing all bookmarks.
    pub fn confirm_clear_bookmarks(&mut self) {
        self.config.clear_bookmarks();
        let _ = self.config.save();
        self.bookmark_selected = 0;
        *self.bookmark_table_state.offset_mut() = 0;
        self.input_mode = InputMode::Normal;
        self.status_message = Some("All bookmarks cleared".to_string());
    }

    /// Cancel the clear-all confirmation.
    pub fn cancel_clear_bookmarks(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Open bookmark detail view and mark the bookmark as read.
    pub fn open_bookmark_detail(&mut self) {
        let filtered = self.get_filtered_bookmarks();
        if let Some(b) = filtered.get(self.bookmark_selected) {
            let id = b.id.clone();
            // Mark as read in the actual bookmarks list
            if let Some(b) = self.config.bookmarks.iter_mut().find(|b| b.id == id) {
                b.read = true;
            }
            let _ = self.config.save();
            self.input_mode = InputMode::BookmarkDetail;
            self.bookmark_detail_scroll = 0;
        }
    }

    /// Toggle read/unread on the currently selected bookmark.
    pub fn toggle_selected_bookmark_read(&mut self) {
        let filtered = self.get_filtered_bookmarks();
        if let Some(b) = filtered.get(self.bookmark_selected) {
            let id = b.id.clone();
            if let Some(b) = self.config.bookmarks.iter_mut().find(|b| b.id == id) {
                b.read = !b.read;
            }
            let _ = self.config.save();
        }
    }
}
