use crate::app::{App, ExportFormat, ExportScope};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use super::centered_rect;
use super::formatters::*;

pub use super::detail::draw_stock_detail;

fn export_menu_content(app: &App) -> Vec<Line<'static>> {
    let sel = app.export_menu_selection;
    let format_str = match app.export_format {
        ExportFormat::Csv => "CSV",
        ExportFormat::Json => "JSON",
    };
    let scope_str = match app.export_scope {
        ExportScope::Watchlist => "Watchlist",
        ExportScope::Portfolio => "Portfolio",
    };
    let row_style = |selected: bool| -> Style {
        if selected {
            Style::default().bg(Color::Rgb(40, 80, 40)).fg(Color::White)
        } else {
            Style::default()
        }
    };
    vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Format:  ", row_style(sel == 0)),
            Span::styled(format!("< {} >", format_str), row_style(sel == 0).fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("              ", row_style(sel == 0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Scope:   ", row_style(sel == 1)),
            Span::styled(format!("< {} >", scope_str), row_style(sel == 1).fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("            ", row_style(sel == 1)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "        [ Export ]        ",
            if sel == 2 {
                Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            },
        )]),
        Line::from(""),
        Line::from(Span::styled("  [←→] Toggle  [Enter] Confirm", Style::default().fg(Color::DarkGray))),
    ]
}

pub fn draw_export_menu(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" Export Data ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let menu = Paragraph::new(export_menu_content(app)).alignment(Alignment::Left);
    frame.render_widget(menu, inner_area);
}

pub fn draw_portfolio_chart(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" Portfolio Allocation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let allocations = app.portfolio_allocation();
    let total_value: f64 = allocations.iter().map(|(_, v, _)| v).sum();
    let bar_colors = [
        Color::Cyan, Color::Green, Color::Yellow, Color::Magenta,
        Color::Blue, Color::Red, Color::LightCyan, Color::LightGreen,
    ];
    let bar_max_width = inner_area.width.saturating_sub(24) as usize;

    let mut content = vec![
        Line::from(vec![
            Span::raw("  Total Value: "),
            Span::styled(format_value(total_value), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    for (i, (symbol, value, pct)) in allocations.iter().enumerate() {
        let color = bar_colors[i % bar_colors.len()];
        let filled = ((pct / 100.0) * bar_max_width as f64).round() as usize;
        let empty = bar_max_width.saturating_sub(filled);
        content.push(Line::from(vec![
            Span::styled(format!("  {:6} ", symbol), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {:5.1}% ", pct)),
            Span::styled(format_value(*value), Style::default().fg(Color::DarkGray)),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled("  [c/Enter/Esc] Close", Style::default().fg(Color::DarkGray))));

    let chart = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(chart, inner_area);
}

fn help_section(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("─── {} ", title), Style::default().fg(Color::Yellow)),
        Span::styled("───────────────────────────", Style::default().fg(Color::DarkGray)),
    ])
}

fn help_binding(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:12}", key), Style::default().fg(Color::Cyan)),
        Span::raw(desc.to_string()),
    ])
}

fn help_content() -> Vec<Line<'static>> {
    vec![
        help_section("General"),
        help_binding("q", "Quit"),
        help_binding("p", "Cycle Watchlist / Portfolio / News"),
        help_binding("r", "Refresh quotes"),
        help_binding("?", "Show this help"),
        help_binding("Enter", "Stock detail popup"),
        help_binding("↓", "Move selection down"),
        help_binding("↑", "Move selection up"),
        Line::from(""),
        help_section("Watchlist"),
        help_binding("a", "Add stock symbol"),
        help_binding("d", "Delete selected stock"),
        help_binding("e", "Export data (CSV/JSON)"),
        help_binding("h / ←", "Previous watchlist"),
        help_binding("l / →", "Next watchlist"),
        help_binding("n", "New watchlist"),
        help_binding("R", "Rename watchlist"),
        help_binding("D", "Delete watchlist"),
        Line::from(""),
        help_section("Portfolio"),
        help_binding("a", "Add holding (step-by-step)"),
        help_binding("e", "Edit selected holding"),
        help_binding("d", "Delete selected holding"),
        help_binding("c", "Portfolio allocation chart"),
        Line::from(""),
        help_section("News"),
        help_binding("r", "Refresh news feeds"),
        help_binding("Enter", "Open article preview"),
        help_binding("o", "Open article in browser (in preview)"),
        Line::from(""),
        help_section("Other"),
        help_binding("s", "Cycle sort column"),
        help_binding("S", "Toggle sort direction"),
        help_binding("/", "Search / filter symbols"),
        help_binding("e", "Export data (CSV/JSON)"),
        Line::from(""),
        Line::from(Span::styled("  [?/Enter/Esc] Close", Style::default().fg(Color::DarkGray))),
    ]
}

pub fn draw_help(frame: &mut Frame) {
    let area = centered_rect(50, 70, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" Help - Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    frame.render_widget(Paragraph::new(help_content()).alignment(Alignment::Left), inner_area);
}

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
            b'<' => { in_tag = true; i += 1; }
            b'>' => { in_tag = false; i += 1; }
            _ if in_tag => { i += 1; }
            b'&' => {
                let rest = &bytes[i..];
                if rest.starts_with(b"&amp;")       { out.push('&'); i += 5; }
                else if rest.starts_with(b"&lt;")   { out.push('<'); i += 4; }
                else if rest.starts_with(b"&gt;")   { out.push('>'); i += 4; }
                else if rest.starts_with(b"&nbsp;") { out.push(' '); i += 6; }
                else if rest.starts_with(b"&quot;") { out.push('"'); i += 6; }
                else if rest.starts_with(b"&#39;")  { out.push('\''); i += 5; }
                else { out.push('&'); i += 1; }
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

pub fn draw_news_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 55, frame.area());
    frame.render_widget(Clear, area);

    let items = app.get_filtered_news();
    let item = match items.get(app.news_selected) {
        Some(i) => *i,
        None => return,
    };

    let relative = format_relative_time(item.published_at);
    let full_dt = chrono::DateTime::from_timestamp(item.published_at, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y  %H:%M").to_string()
        })
        .unwrap_or_default();

    let outer_block = Block::default()
        .title(format!(" {} ", item.publisher))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    use ratatui::layout::{Constraint, Direction, Layout};
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
    for line in word_wrap(&item.title, inner_width) {
        all_lines.push(Line::from(Span::styled(
            line,
            Style::default().add_modifier(Modifier::BOLD),
        )));
    }

    // Metadata: full datetime + relative
    all_lines.push(Line::from(vec![
        Span::styled(full_dt, Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  ({})", relative), Style::default().fg(Color::DarkGray)),
    ]));

    // URL truncated to fit width
    if let Some(u) = &item.url {
        let display = if u.len() > inner_width {
            format!("{}…", &u[..inner_width.saturating_sub(1)])
        } else {
            u.clone()
        };
        all_lines.push(Line::from(Span::styled(display, Style::default().fg(Color::Blue))));
    }

    all_lines.push(Line::from(Span::styled(
        "─".repeat(inner_width),
        Style::default().fg(Color::DarkGray),
    )));

    // Body: RSS summary with HTML stripped
    let body_clean = strip_tag_content(item.summary.as_deref().unwrap_or(""));

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
    let scroll = app.news_detail_scroll.min(max_scroll);

    let visible: Vec<Line> = all_lines.into_iter().skip(scroll).take(body_height).collect();
    frame.render_widget(Paragraph::new(visible), body_area);

    let footer_line = if item.url.is_some() {
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
