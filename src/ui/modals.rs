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

fn alert_modal_content(app: &crate::app::App) -> Vec<Line<'static>> {
    use std::borrow::Cow;
    let sym = match &app.alert_symbol {
        Some(s) => s.clone(),
        None => return vec![],
    };
    let alerts = app.config.alerts_for_symbol(&sym);
    let count = alerts.len();
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Alerts for {}", sym),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, alert) in alerts.iter().enumerate() {
        let is_sel = i == app.alert_list_selected;
        let enabled_icon = if alert.enabled { "●" } else { "○" };
        let enabled_color = if alert.enabled { Color::Green } else { Color::DarkGray };
        let label = Cow::from(format!(
            "  {} {:8} {:>10.2}  {}",
            enabled_icon,
            alert.alert_type.label(),
            alert.target_value,
            if alert.enabled { "ON " } else { "OFF" },
        ));
        let row_style = if is_sel {
            Style::default().bg(Color::Rgb(40, 40, 80)).fg(Color::White)
        } else {
            Style::default().fg(enabled_color)
        };
        lines.push(Line::from(Span::styled(label, row_style)));
    }

    let add_style = if app.alert_list_selected == count {
        Style::default().bg(Color::Rgb(40, 80, 40)).fg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  [ + Add Alert ]", add_style)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Toggle/Add  [d] Delete  [↑↓/jk] Nav  [Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));
    lines
}

fn alert_add_type_content(app: &crate::app::App) -> Vec<Line<'static>> {
    use crate::config::AlertType;
    use std::borrow::Cow;
    let types = [AlertType::Above, AlertType::Below, AlertType::PercentGain, AlertType::PercentLoss];
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Select alert type:", Style::default().fg(Color::Cyan))),
        Line::from(""),
    ];
    for t in &types {
        let is_sel = &app.pending_alert_type == t;
        let style = if is_sel {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label = Cow::from(format!("  {} {}", if is_sel { ">" } else { " " }, t.label()));
        lines.push(Line::from(Span::styled(label, style)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [h/l or ←→] Next type  [Enter] Confirm  [Esc] Back",
        Style::default().fg(Color::DarkGray),
    )));
    lines
}

fn alert_add_value_content(app: &crate::app::App) -> Vec<Line<'static>> {
    use std::borrow::Cow;
    vec![
        Line::from(""),
        Line::from(Span::styled(
            Cow::from(format!("  Type: {}", app.pending_alert_type.label())),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Target: "),
            Span::styled(Cow::from(app.input_buffer.clone()), Style::default().fg(Color::Yellow)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  [Enter] Add  [Esc] Back",
            Style::default().fg(Color::DarkGray),
        )),
    ]
}

pub fn draw_alert_modal(frame: &mut Frame, app: &crate::app::App) {
    use crate::app::InputMode;
    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::AlertAddType => " Set Alert Type ",
        InputMode::AlertAddValue => " Set Alert Value ",
        _ => " Manage Alerts ",
    };
    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));
    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let content = match app.input_mode {
        InputMode::AlertAddType => alert_add_type_content(app),
        InputMode::AlertAddValue => alert_add_value_content(app),
        _ => alert_modal_content(app),
    };
    frame.render_widget(
        Paragraph::new(content).alignment(Alignment::Left),
        inner_area,
    );
}
