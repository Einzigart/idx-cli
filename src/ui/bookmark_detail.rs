use super::centered_rect;
use super::formatters::*;
use super::news_detail::word_wrap;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn draw_bookmark_detail(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 55, frame.area());
    frame.render_widget(Clear, area);

    let (headline, source, published_at, bookmarked_at, url, read, summary) = {
        let filtered = app.get_filtered_bookmarks();
        let bookmark = match filtered.get(app.bookmark_selected) {
            Some(b) => *b,
            None => return,
        };
        // Look up summary from live news items if still available
        let summary = app
            .news_items
            .iter()
            .find(|item| item.title == bookmark.headline && item.url == bookmark.url)
            .and_then(|item| item.summary.clone());
        (
            bookmark.headline.clone(),
            bookmark.source.clone(),
            bookmark.published_at,
            bookmark.bookmarked_at,
            bookmark.url.clone(),
            bookmark.read,
            summary,
        )
    };

    let pub_relative = format_relative_time(published_at);
    let pub_full = chrono::DateTime::from_timestamp(published_at, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y  %H:%M")
                .to_string()
        })
        .unwrap_or_default();

    let bm_full = chrono::DateTime::from_timestamp(bookmarked_at, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y  %H:%M")
                .to_string()
        })
        .unwrap_or_default();

    let outer_block = Block::default()
        .title(format!(" {} ", source))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner_area);
    let body_area = chunks[0];
    let footer_area = chunks[1];

    let inner_width = body_area.width as usize;
    let body_height = body_area.height as usize;

    let mut all_lines: Vec<Line> = Vec::new();

    // Title (bold, wrapped)
    for line in word_wrap(&headline, inner_width) {
        all_lines.push(Line::from(Span::styled(
            line,
            Style::default().add_modifier(Modifier::BOLD),
        )));
    }

    // Published date
    all_lines.push(Line::from(vec![
        Span::styled("Published: ", Style::default().fg(Color::DarkGray)),
        Span::styled(pub_full, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("  ({})", pub_relative),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // Bookmarked date
    all_lines.push(Line::from(vec![
        Span::styled("Bookmarked: ", Style::default().fg(Color::DarkGray)),
        Span::styled(bm_full, Style::default().fg(Color::DarkGray)),
    ]));

    // Read status
    let read_label = if read { "Read" } else { "Unread" };
    let read_color = if read { Color::DarkGray } else { Color::Yellow };
    all_lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            read_label,
            Style::default().fg(read_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // URL
    if let Some(u) = &url {
        let display = if u.chars().count() > inner_width {
            let end = u
                .char_indices()
                .nth(inner_width.saturating_sub(1))
                .map(|(i, _)| i)
                .unwrap_or(u.len());
            format!("{}…", &u[..end])
        } else {
            u.clone()
        };
        all_lines.push(Line::from(Span::styled(
            display,
            Style::default().fg(Color::Blue),
        )));
    }

    all_lines.push(Line::from(Span::styled(
        "─".repeat(inner_width),
        Style::default().fg(Color::DarkGray),
    )));

    // Summary from live news if available
    if let Some(body) = &summary {
        for line in word_wrap(body, inner_width) {
            all_lines.push(Line::from(line));
        }
    } else {
        all_lines.push(Line::from(Span::styled(
            "Summary not available (article no longer in feed).",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let max_scroll = all_lines.len().saturating_sub(body_height);
    app.bookmark_detail_scroll = app.bookmark_detail_scroll.min(max_scroll);

    let visible: Vec<Line> = all_lines
        .into_iter()
        .skip(app.bookmark_detail_scroll)
        .take(body_height)
        .collect();
    frame.render_widget(Paragraph::new(visible), body_area);

    let footer_line = Line::from(vec![
        Span::styled("[o] ", Style::default().fg(Color::Cyan)),
        Span::styled("browser  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[m] ", Style::default().fg(Color::Cyan)),
        Span::styled("toggle read  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
        Span::styled("scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
        Span::styled("close", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(footer_line), footer_area);
}
