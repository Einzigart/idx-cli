use super::centered_rect;
use super::formatters::*;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Wrap text to lines of at most `width` characters, breaking at word boundaries.
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

/// Strip HTML tags (including all attribute text) and decode common entities.
fn strip_tag_content(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => {
                in_tag = true;
                i += 1;
            }
            b'>' => {
                in_tag = false;
                i += 1;
            }
            _ if in_tag => {
                i += 1;
            }
            b'&' => {
                let rest = &bytes[i..];
                if rest.starts_with(b"&amp;") {
                    out.push('&');
                    i += 5;
                } else if rest.starts_with(b"&lt;") {
                    out.push('<');
                    i += 4;
                } else if rest.starts_with(b"&gt;") {
                    out.push('>');
                    i += 4;
                } else if rest.starts_with(b"&nbsp;") {
                    out.push(' ');
                    i += 6;
                } else if rest.starts_with(b"&quot;") {
                    out.push('"');
                    i += 6;
                } else if rest.starts_with(b"&#39;") {
                    out.push('\'');
                    i += 5;
                } else {
                    out.push('&');
                    i += 1;
                }
            }
            b => {
                let ch_len = match b {
                    0x00..=0x7F => 1,
                    0xC0..=0xDF => 2,
                    0xE0..=0xEF => 3,
                    0xF0..=0xF7 => 4,
                    _ => 1,
                };
                let end = (i + ch_len).min(bytes.len());
                if let Ok(s) = std::str::from_utf8(&bytes[i..end]) {
                    out.push_str(s);
                }
                i = end;
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn draw_news_detail(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 55, frame.area());
    frame.render_widget(Clear, area);

    // Extract owned data from the borrowed item so we can later mutate app.
    let (title, publisher, published_at, url, summary) = {
        let items = app.get_filtered_news();
        let item = match items.get(app.news_selected) {
            Some(i) => *i,
            None => return,
        };
        (
            item.title.clone(),
            item.publisher.clone(),
            item.published_at,
            item.url.clone(),
            item.summary.clone(),
        )
    };

    let relative = format_relative_time(published_at);
    let full_dt = chrono::DateTime::from_timestamp(published_at, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y  %H:%M")
                .to_string()
        })
        .unwrap_or_default();

    let outer_block = Block::default()
        .title(format!(" {} ", publisher))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
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
    for line in word_wrap(&title, inner_width) {
        all_lines.push(Line::from(Span::styled(
            line,
            Style::default().add_modifier(Modifier::BOLD),
        )));
    }

    // Metadata: full datetime + relative
    all_lines.push(Line::from(vec![
        Span::styled(full_dt, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("  ({})", relative),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // URL truncated to fit width
    if let Some(u) = &url {
        let display = if u.len() > inner_width {
            format!("{}…", &u[..inner_width.saturating_sub(1)])
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

    // Body: RSS summary with HTML stripped
    let body_clean = strip_tag_content(summary.as_deref().unwrap_or(""));

    if body_clean.is_empty() {
        all_lines.push(Line::from(Span::styled(
            "No summary available.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for line in word_wrap(&body_clean, inner_width) {
            all_lines.push(Line::from(line));
        }
    }

    let max_scroll = all_lines.len().saturating_sub(body_height);
    app.news_detail_scroll = app.news_detail_scroll.min(max_scroll);

    let visible: Vec<Line> = all_lines
        .into_iter()
        .skip(app.news_detail_scroll)
        .take(body_height)
        .collect();
    frame.render_widget(Paragraph::new(visible), body_area);

    let footer_line = if url.is_some() {
        Line::from(vec![
            Span::styled("[o] ", Style::default().fg(Color::Cyan)),
            Span::styled("browser  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
            Span::styled("scroll  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
            Span::styled("close", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
            Span::styled("scroll  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
            Span::styled("close", Style::default().fg(Color::DarkGray)),
        ])
    };
    frame.render_widget(Paragraph::new(footer_line), footer_area);
}
