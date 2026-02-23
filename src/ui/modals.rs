use super::centered_rect;
use super::formatters::format_value;
use crate::app::{App, ExportFormat, ExportScope};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

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
            Span::styled(
                format!("< {} >", format_str),
                row_style(sel == 0)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("              ", row_style(sel == 0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Scope:   ", row_style(sel == 1)),
            Span::styled(
                format!("< {} >", scope_str),
                row_style(sel == 1)
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("            ", row_style(sel == 1)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "        [ Export ]        ",
            if sel == 2 {
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            },
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "  [←→] Toggle  [Enter] Confirm",
            Style::default().fg(Color::DarkGray),
        )),
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
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::Blue,
        Color::Red,
        Color::LightCyan,
        Color::LightGreen,
    ];
    let bar_max_width = inner_area.width.saturating_sub(24) as usize;

    let mut content = vec![
        Line::from(vec![
            Span::raw("  Total Value: "),
            Span::styled(
                format_value(total_value),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    for (i, (symbol, value, pct)) in allocations.iter().enumerate() {
        let color = bar_colors[i % bar_colors.len()];
        let filled = ((pct / 100.0) * bar_max_width as f64).round() as usize;
        let empty = bar_max_width.saturating_sub(filled);
        content.push(Line::from(vec![
            Span::styled(
                format!("  {:6} ", symbol),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {:5.1}% ", pct)),
            Span::styled(format_value(*value), Style::default().fg(Color::DarkGray)),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "  [c/Enter/Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));

    let chart = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(chart, inner_area);
}

fn help_section(title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("─── {} ", title),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            "───────────────────────────",
            Style::default().fg(Color::DarkGray),
        ),
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
        help_binding("h / ←", "Previous portfolio"),
        help_binding("l / →", "Next portfolio"),
        help_binding("n", "New portfolio"),
        help_binding("R", "Rename portfolio"),
        help_binding("D", "Delete portfolio"),
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
        Line::from(Span::styled(
            "  [?/Enter/Esc] Close",
            Style::default().fg(Color::DarkGray),
        )),
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

    frame.render_widget(
        Paragraph::new(help_content()).alignment(Alignment::Left),
        inner_area,
    );
}
