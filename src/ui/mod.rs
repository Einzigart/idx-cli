mod detail;
mod formatters;
mod modals;
mod news;
mod tables;

pub(crate) use news::NEWS_SORTABLE_COLUMNS;
pub(crate) use tables::{PORTFOLIO_SORTABLE_COLUMNS, WATCHLIST_SORTABLE_COLUMNS};

use crate::app::{App, InputMode, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);

    match app.view_mode {
        ViewMode::Watchlist => tables::draw_watchlist(frame, chunks[1], app),
        ViewMode::Portfolio => tables::draw_portfolio(frame, chunks[1], app),
        ViewMode::News => news::draw_news(frame, chunks[1], app),
    }

    draw_footer(frame, chunks[2], app);

    if app.input_mode == InputMode::StockDetail {
        modals::draw_stock_detail(frame, app);
    }
    if app.input_mode == InputMode::Help {
        modals::draw_help(frame);
    }
    if app.input_mode == InputMode::ExportMenu {
        modals::draw_export_menu(frame, app);
    }
    if app.input_mode == InputMode::PortfolioChart {
        modals::draw_portfolio_chart(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let current_time = chrono::Local::now().format("%H:%M:%S").to_string();
    let status = if app.loading {
        "[Loading...]".to_string()
    } else {
        format!("[{}]", current_time)
    };

    let (view_indicator, view_color) = match app.view_mode {
        ViewMode::Watchlist => (app.watchlist_indicator(), Color::Yellow),
        ViewMode::Portfolio => ("Portfolio".to_string(), Color::Magenta),
        ViewMode::News => ("News".to_string(), Color::Blue),
    };

    let filter_span = if app.search_active {
        Span::styled(
            format!(" (filtered: {})", app.search_query),
            Style::default().fg(Color::Cyan),
        )
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" IDX Stock Tracker ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(view_indicator, Style::default().fg(view_color).add_modifier(Modifier::BOLD)),
        filter_span,
        Span::styled(" ", Style::default()),
        Span::styled(status, Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let content = match app.input_mode {
        InputMode::Normal => {
            let help = match app.view_mode {
                ViewMode::Watchlist => " [a] Add [d] Del [e] Export [r] Refresh [s] Sort [p] Portfolio [Enter] Detail [↑↓] Nav [←→] WL [?] Help ",
                ViewMode::Portfolio => " [a] Add [e] Edit [d] Del [r] Refresh [s] Sort [c] Chart [p] News [Enter] Detail [↑↓] Nav [?] Help ",
                ViewMode::News => " [r] Refresh [s] Sort [/] Search [p] Watchlist [↑↓] Nav [?] Help ",
            };
            if let Some(msg) = &app.status_message {
                Line::from(vec![
                    Span::styled(msg, Style::default().fg(Color::Yellow)),
                    Span::raw(" | "),
                    Span::styled(help, Style::default().fg(Color::DarkGray)),
                ])
            } else {
                Line::from(Span::styled(help, Style::default().fg(Color::DarkGray)))
            }
        }
        InputMode::Adding => Line::from(vec![
            Span::raw(" Add stock: "),
            Span::styled(&app.input_buffer, Style::default().fg(Color::Cyan)),
            Span::styled("█", Style::default().fg(Color::Cyan)),
            Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
        ]),
        InputMode::WatchlistAdd => Line::from(vec![
            Span::raw(" New watchlist name: "),
            Span::styled(&app.input_buffer, Style::default().fg(Color::Green)),
            Span::styled("█", Style::default().fg(Color::Green)),
            Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
        ]),
        InputMode::WatchlistRename => Line::from(vec![
            Span::raw(" Rename watchlist: "),
            Span::styled(&app.input_buffer, Style::default().fg(Color::Yellow)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
        ]),
        InputMode::PortfolioAddSymbol => Line::from(vec![
            Span::raw(" Symbol: "),
            Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
            Span::styled("█", Style::default().fg(Color::Magenta)),
            Span::raw(" | [Enter] Next | [Esc] Cancel"),
        ]),
        InputMode::PortfolioAddLots => {
            let symbol = app.pending_symbol.as_deref().unwrap_or("");
            Line::from(vec![
                Span::styled(format!("{} ", symbol), Style::default().fg(Color::Green)),
                Span::raw("Lots: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
                Span::raw(" | [Enter] Next | [Esc] Cancel"),
            ])
        }
        InputMode::PortfolioAddPrice => {
            let symbol = app.pending_symbol.as_deref().unwrap_or("");
            let lots = app.pending_lots.unwrap_or(0);
            Line::from(vec![
                Span::styled(format!("{} {}lot ", symbol, lots), Style::default().fg(Color::Green)),
                Span::raw("Avg Price: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
                Span::raw(" | [Enter] Add | [Esc] Cancel"),
            ])
        }
        InputMode::PortfolioEditLots => {
            let symbol = app.pending_edit_symbol.as_deref().unwrap_or("");
            Line::from(vec![
                Span::raw(format!(" Edit {} Lots: ", symbol)),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
                Span::raw(" | [Enter] Next | [Esc] Cancel"),
            ])
        }
        InputMode::PortfolioEditPrice => {
            let symbol = app.pending_edit_symbol.as_deref().unwrap_or("");
            let lots = app.pending_lots.unwrap_or(0);
            Line::from(vec![
                Span::styled(format!(" Edit {} {}lot ", symbol, lots), Style::default().fg(Color::Green)),
                Span::raw("Avg Price: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
                Span::raw(" | [Enter] Save | [Esc] Cancel"),
            ])
        }
        InputMode::StockDetail => Line::from(Span::styled(
            " [Enter/Esc] Close detail view ",
            Style::default().fg(Color::DarkGray),
        )),
        InputMode::Help => Line::from(Span::styled(
            " [?/Enter/Esc] Close help ",
            Style::default().fg(Color::DarkGray),
        )),
        InputMode::Search => Line::from(vec![
            Span::raw(" Search: /"),
            Span::styled(&app.input_buffer, Style::default().fg(Color::Cyan)),
            Span::styled("█", Style::default().fg(Color::Cyan)),
            Span::raw(" | [Enter] Apply | [Esc] Cancel"),
        ]),
        InputMode::ExportMenu => Line::from(Span::styled(
            " [↑↓/jk] Navigate | [←→/hl] Toggle | [Enter] Confirm | [Esc] Cancel ",
            Style::default().fg(Color::DarkGray),
        )),
        InputMode::PortfolioChart => Line::from(Span::styled(
            " [c/Enter/Esc] Close allocation chart ",
            Style::default().fg(Color::DarkGray),
        )),
    };

    let footer = Paragraph::new(content).block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}
