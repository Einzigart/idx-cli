use crate::api::StockQuote;
use crate::app::{App, ExportFormat, ExportScope, InputMode, ViewMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Sparkline, Table, TableState},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Table
            Constraint::Length(3), // Footer/Input
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);

    match app.view_mode {
        ViewMode::Watchlist => draw_watchlist(frame, chunks[1], app),
        ViewMode::Portfolio => draw_portfolio(frame, chunks[1], app),
    }

    draw_footer(frame, chunks[2], app);

    // Render popup on top if in StockDetail mode
    if app.input_mode == InputMode::StockDetail {
        draw_stock_detail(frame, app);
    }

    // Render help modal if in Help mode
    if app.input_mode == InputMode::Help {
        draw_help(frame, app);
    }

    // Render export menu if in ExportMenu mode
    if app.input_mode == InputMode::ExportMenu {
        draw_export_menu(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let current_time = chrono::Local::now().format("%H:%M:%S").to_string();

    let status = if app.loading {
        "[Loading...]".to_string()
    } else {
        format!("[{}]", current_time)
    };

    let (view_indicator, view_color) = match app.view_mode {
        ViewMode::Watchlist => (app.watchlist_indicator(), Color::Yellow),
        ViewMode::Portfolio => ("Portfolio".to_string(), Color::Magenta),
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

fn draw_watchlist(frame: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["Symbol", "Name", "Price", "Change", "Change %", "Open", "High", "Low", "Volume", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let watchlist = app.get_filtered_watchlist();
    let rows: Vec<Row> = watchlist
        .iter()
        .enumerate()
        .map(|(i, (symbol, quote)): (usize, &(&String, Option<&StockQuote>))| {
            let is_selected = i == app.selected_index;

            if let Some(q) = quote {
                let (change_color, selected_change_color) = if q.change >= 0.0 {
                    (Color::Green, Color::LightGreen)
                } else {
                    (Color::Red, Color::LightRed)
                };

                // Calculate value (price * volume)
                let value = q.price * q.volume as f64;

                let cells = if is_selected {
                    // Selected row: bright text on blue background
                    vec![
                        Cell::from(q.symbol.clone()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Cell::from(truncate_str(&q.short_name, 20)).style(Style::default().fg(Color::White)),
                        Cell::from(format_price(q.price)).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Cell::from(format_change(q.change)).style(Style::default().fg(selected_change_color).add_modifier(Modifier::BOLD)),
                        Cell::from(format!("{:+.2}%", q.change_percent)).style(Style::default().fg(selected_change_color).add_modifier(Modifier::BOLD)),
                        Cell::from(format_price(q.open)).style(Style::default().fg(Color::White)),
                        Cell::from(format_price(q.high)).style(Style::default().fg(Color::White)),
                        Cell::from(format_price(q.low)).style(Style::default().fg(Color::White)),
                        Cell::from(format_volume(q.volume)).style(Style::default().fg(Color::White)),
                        Cell::from(format_value(value)).style(Style::default().fg(Color::Cyan)),
                    ]
                } else {
                    // Normal row
                    vec![
                        Cell::from(q.symbol.clone()),
                        Cell::from(truncate_str(&q.short_name, 20)),
                        Cell::from(format_price(q.price)),
                        Cell::from(format_change(q.change)).style(Style::default().fg(change_color)),
                        Cell::from(format!("{:+.2}%", q.change_percent)).style(Style::default().fg(change_color)),
                        Cell::from(format_price(q.open)),
                        Cell::from(format_price(q.high)),
                        Cell::from(format_price(q.low)),
                        Cell::from(format_volume(q.volume)),
                        Cell::from(format_value(value)),
                    ]
                };

                let row_style = if is_selected {
                    Style::default().bg(Color::Rgb(40, 80, 120))
                } else {
                    Style::default()
                };

                Row::new(cells).style(row_style)
            } else {
                let style = if is_selected {
                    Style::default().bg(Color::Rgb(40, 80, 120)).fg(Color::White)
                } else {
                    Style::default()
                };

                let cells = vec![
                    Cell::from(symbol.as_str()),
                    Cell::from("Loading..."),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                    Cell::from("-"),
                ];
                Row::new(cells).style(style)
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // Symbol
            Constraint::Length(22), // Name
            Constraint::Length(10), // Price
            Constraint::Length(10), // Change
            Constraint::Length(10), // Change %
            Constraint::Length(10), // Open
            Constraint::Length(10), // High
            Constraint::Length(10), // Low
            Constraint::Length(12), // Volume
            Constraint::Length(14), // Value
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Watchlist "));

    let mut state = TableState::default();
    state.select(Some(app.selected_index));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_portfolio(frame: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["Symbol", "Lots", "Shares", "Avg Price", "Curr Price", "Value", "Cost", "P/L", "P/L %"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    // Calculate totals
    let mut total_value = 0.0;
    let mut total_cost = 0.0;

    let filtered = app.get_filtered_portfolio();
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_orig_idx, holding))| {
            let is_selected = i == app.portfolio_selected;
            let quote = app.quotes.get(&holding.symbol);

            let curr_price = quote.map(|q| q.price).unwrap_or(0.0);
            let shares = holding.shares();
            let value = curr_price * shares as f64;
            let cost = holding.cost_basis();
            let pl = value - cost;
            let pl_percent = if cost > 0.0 { (pl / cost) * 100.0 } else { 0.0 };

            total_value += value;
            total_cost += cost;

            let pl_color = if pl >= 0.0 { Color::Green } else { Color::Red };
            let selected_pl_color = if pl >= 0.0 { Color::LightGreen } else { Color::LightRed };

            let cells = if is_selected {
                vec![
                    Cell::from(holding.symbol.clone()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Cell::from(format!("{}", holding.lots)).style(Style::default().fg(Color::White)),
                    Cell::from(format!("{}", shares)).style(Style::default().fg(Color::White)),
                    Cell::from(format_price(holding.avg_price)).style(Style::default().fg(Color::White)),
                    Cell::from(format_price(curr_price)).style(Style::default().fg(Color::White)),
                    Cell::from(format_value(value)).style(Style::default().fg(Color::White)),
                    Cell::from(format_value(cost)).style(Style::default().fg(Color::White)),
                    Cell::from(format_pl(pl)).style(Style::default().fg(selected_pl_color).add_modifier(Modifier::BOLD)),
                    Cell::from(format!("{:+.2}%", pl_percent)).style(Style::default().fg(selected_pl_color).add_modifier(Modifier::BOLD)),
                ]
            } else {
                vec![
                    Cell::from(holding.symbol.clone()),
                    Cell::from(format!("{}", holding.lots)),
                    Cell::from(format!("{}", shares)),
                    Cell::from(format_price(holding.avg_price)),
                    Cell::from(format_price(curr_price)),
                    Cell::from(format_value(value)),
                    Cell::from(format_value(cost)),
                    Cell::from(format_pl(pl)).style(Style::default().fg(pl_color)),
                    Cell::from(format!("{:+.2}%", pl_percent)).style(Style::default().fg(pl_color)),
                ]
            };

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(80, 40, 80))
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style)
        })
        .collect();

    // Calculate total P/L
    let total_pl = total_value - total_cost;
    let total_pl_percent = if total_cost > 0.0 { (total_pl / total_cost) * 100.0 } else { 0.0 };
    let total_pl_color = if total_pl >= 0.0 { Color::Green } else { Color::Red };

    let title = format!(
        " Portfolio | Value: {} | P/L: {} ({:+.2}%) ",
        format_value(total_value),
        format_pl(total_pl),
        total_pl_percent
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // Symbol
            Constraint::Length(6),  // Lots
            Constraint::Length(8),  // Shares
            Constraint::Length(10), // Avg Price
            Constraint::Length(10), // Curr Price
            Constraint::Length(12), // Value
            Constraint::Length(12), // Cost
            Constraint::Length(12), // P/L
            Constraint::Length(10), // P/L %
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(total_pl_color))
    );

    let mut state = TableState::default();
    state.select(Some(app.portfolio_selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let content = match app.input_mode {
        InputMode::Normal => {
            let help = match app.view_mode {
                ViewMode::Watchlist => " [p] Portfolio | [a] Add | [d] Del | [r] Refresh | [q] Quit | [Enter] Detail | [↑↓] Nav | [←→] WL ",
                ViewMode::Portfolio => " [p] Watchlist | [a] Add | [d] Del | [r] Refresh | [q] Quit | [Enter] Detail | [↑↓] Nav ",
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
        InputMode::Adding => {
            Line::from(vec![
                Span::raw(" Add stock: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Cyan)),
                Span::styled("█", Style::default().fg(Color::Cyan)),
                Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
            ])
        }
        InputMode::WatchlistAdd => {
            Line::from(vec![
                Span::raw(" New watchlist name: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Green)),
                Span::styled("█", Style::default().fg(Color::Green)),
                Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
            ])
        }
        InputMode::WatchlistRename => {
            Line::from(vec![
                Span::raw(" Rename watchlist: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Yellow)),
                Span::styled("█", Style::default().fg(Color::Yellow)),
                Span::raw(" | [Enter] Confirm | [Esc] Cancel"),
            ])
        }
        InputMode::PortfolioAddSymbol => {
            Line::from(vec![
                Span::raw(" Symbol: "),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Magenta)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
                Span::raw(" | [Enter] Next | [Esc] Cancel"),
            ])
        }
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
        InputMode::StockDetail => {
            Line::from(Span::styled(
                " [Enter/Esc] Close detail view ",
                Style::default().fg(Color::DarkGray),
            ))
        }
        InputMode::Help => {
            Line::from(Span::styled(
                " [?/Enter/Esc] Close help ",
                Style::default().fg(Color::DarkGray),
            ))
        }
        InputMode::Search => {
            Line::from(vec![
                Span::raw(" Search: /"),
                Span::styled(&app.input_buffer, Style::default().fg(Color::Cyan)),
                Span::styled("█", Style::default().fg(Color::Cyan)),
                Span::raw(" | [Enter] Apply | [Esc] Cancel"),
            ])
        }
        InputMode::ExportMenu => {
            Line::from(Span::styled(
                " [↑↓/jk] Navigate | [←→/hl] Toggle | [Enter] Confirm | [Esc] Cancel ",
                Style::default().fg(Color::DarkGray),
            ))
        }
    };

    let footer = Paragraph::new(content).block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        // Format with thousand separators manually
        let int_part = price as u64;
        let formatted = int_part
            .to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(",");
        formatted
    } else {
        format!("{:.2}", price)
    }
}

fn format_change(change: f64) -> String {
    if change >= 0.0 {
        format!("+{:.0}", change)
    } else {
        format!("{:.0}", change)
    }
}

fn format_pl(pl: f64) -> String {
    let abs_pl = pl.abs();
    let formatted = if abs_pl >= 1_000_000_000.0 {
        format!("{:.2}B", abs_pl / 1_000_000_000.0)
    } else if abs_pl >= 1_000_000.0 {
        format!("{:.2}M", abs_pl / 1_000_000.0)
    } else if abs_pl >= 1_000.0 {
        format!("{:.2}K", abs_pl / 1_000.0)
    } else {
        format!("{:.0}", abs_pl)
    };
    if pl >= 0.0 {
        format!("+{}", formatted)
    } else {
        format!("-{}", formatted)
    }
}

fn format_volume(volume: u64) -> String {
    if volume >= 1_000_000_000 {
        format!("{:.2}B", volume as f64 / 1_000_000_000.0)
    } else if volume >= 1_000_000 {
        format!("{:.2}M", volume as f64 / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.2}K", volume as f64 / 1_000.0)
    } else {
        format!("{}", volume)
    }
}

fn format_value(value: f64) -> String {
    if value >= 1_000_000_000_000.0 {
        format!("{:.2}T", value / 1_000_000_000_000.0)
    } else if value >= 1_000_000_000.0 {
        format!("{:.2}B", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

fn draw_stock_detail(frame: &mut Frame, app: &App) {
    let area = centered_rect(55, 80, frame.area());

    // Clear the area first
    frame.render_widget(Clear, area);

    let quote = match app.get_detail_quote() {
        Some(q) => q,
        None => return,
    };

    // Create outer block
    let title = format!(" {} - Stock Detail ", quote.symbol);
    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split inner area: content on top, sparkline at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(20),    // Main content
            Constraint::Length(5),  // Sparkline chart
        ])
        .split(inner_area);

    // Calculate additional metrics
    let day_range = quote.high - quote.low;
    let day_range_percent = if day_range > 0.0 {
        ((quote.price - quote.low) / day_range) * 100.0
    } else {
        50.0
    };

    let gap_percent = if quote.prev_close > 0.0 {
        ((quote.open - quote.prev_close) / quote.prev_close) * 100.0
    } else {
        0.0
    };

    let value = quote.price * quote.volume as f64;

    // 52-week position
    let week52_position = match (quote.fifty_two_week_high, quote.fifty_two_week_low) {
        (Some(high), Some(low)) if high > low => {
            Some(((quote.price - low) / (high - low)) * 100.0)
        }
        _ => None,
    };

    let change_color = if quote.change >= 0.0 { Color::Green } else { Color::Red };
    let gap_color = if gap_percent >= 0.0 { Color::Green } else { Color::Red };

    let mut content = vec![
        // Header: Full company name
        Line::from(Span::styled(
            quote.long_name.as_deref().unwrap_or(&quote.short_name),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
    ];

    // Classification section
    let sector = quote.sector.as_deref().unwrap_or("N/A");
    let industry = quote.industry.as_deref().unwrap_or("N/A");
    content.push(Line::from(Span::styled(
        format!("{} | {}", sector, industry),
        Style::default().fg(Color::DarkGray),
    )));

    content.push(Line::from(""));

    // Price section
    content.push(Line::from(vec![
        Span::styled("─── Price ", Style::default().fg(Color::Yellow)),
        Span::styled("───────────────────────────", Style::default().fg(Color::DarkGray)),
    ]));
    content.push(Line::from(vec![
        Span::raw("Current:        "),
        Span::styled(
            format_price(quote.price),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]));
    content.push(Line::from(vec![
        Span::raw("Change:         "),
        Span::styled(
            format!("{} ({:+.2}%)", format_change(quote.change), quote.change_percent),
            Style::default().fg(change_color).add_modifier(Modifier::BOLD),
        ),
    ]));
    content.push(Line::from(vec![
        Span::raw("Open:           "),
        Span::raw(format_price(quote.open)),
        Span::raw("  Prev Close: "),
        Span::raw(format_price(quote.prev_close)),
    ]));
    content.push(Line::from(vec![
        Span::raw("Gap:            "),
        Span::styled(format!("{:+.2}%", gap_percent), Style::default().fg(gap_color)),
    ]));

    content.push(Line::from(""));

    // Day Range section
    content.push(Line::from(vec![
        Span::styled("─── Day Range ", Style::default().fg(Color::Yellow)),
        Span::styled("───────────────────────", Style::default().fg(Color::DarkGray)),
    ]));
    content.push(Line::from(vec![
        Span::raw("High:           "),
        Span::styled(format_price(quote.high), Style::default().fg(Color::Green)),
        Span::raw("  Low: "),
        Span::styled(format_price(quote.low), Style::default().fg(Color::Red)),
    ]));
    content.push(Line::from(vec![
        Span::raw("Position:       "),
        Span::raw(format!("{:.1}% from low", day_range_percent)),
    ]));

    // 52-Week Range section
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("─── 52-Week Range ", Style::default().fg(Color::Yellow)),
        Span::styled("───────────────────", Style::default().fg(Color::DarkGray)),
    ]));
    let w52_high = quote.fifty_two_week_high.map(|v| format_price(v)).unwrap_or_else(|| "N/A".to_string());
    let w52_low = quote.fifty_two_week_low.map(|v| format_price(v)).unwrap_or_else(|| "N/A".to_string());
    content.push(Line::from(vec![
        Span::raw("52W High:       "),
        Span::styled(w52_high, Style::default().fg(Color::Green)),
        Span::raw("  52W Low: "),
        Span::styled(w52_low, Style::default().fg(Color::Red)),
    ]));
    if let Some(pos) = week52_position {
        content.push(Line::from(vec![
            Span::raw("Position:       "),
            Span::raw(format!("{:.1}% from 52W low", pos)),
        ]));
    }

    // Fundamentals section
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("─── Fundamentals ", Style::default().fg(Color::Yellow)),
        Span::styled("────────────────────", Style::default().fg(Color::DarkGray)),
    ]));
    let market_cap_str = quote.market_cap.map(|v| format_market_cap(v)).unwrap_or_else(|| "N/A".to_string());
    let pe_str = quote.trailing_pe.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string());
    let div_yield_str = quote.dividend_yield.map(|v| format!("{:.2}%", v * 100.0)).unwrap_or_else(|| "N/A".to_string());
    content.push(Line::from(vec![
        Span::raw("Market Cap:     "),
        Span::styled(market_cap_str, Style::default().fg(Color::Cyan)),
    ]));
    content.push(Line::from(vec![
        Span::raw("P/E Ratio:      "),
        Span::raw(pe_str),
        Span::raw("  Div Yield: "),
        Span::raw(div_yield_str),
    ]));

    // Risk & Liquidity section
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("─── Risk & Liquidity ", Style::default().fg(Color::Yellow)),
        Span::styled("────────────────", Style::default().fg(Color::DarkGray)),
    ]));
    let beta_str = quote.beta.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string());
    let avg_vol_str = quote.average_volume.map(|v| format_volume(v)).unwrap_or_else(|| "N/A".to_string());
    content.push(Line::from(vec![
        Span::raw("Beta:           "),
        Span::raw(beta_str),
    ]));
    content.push(Line::from(vec![
        Span::raw("Volume:         "),
        Span::raw(format_volume(quote.volume)),
        Span::raw("  Avg Vol: "),
        Span::raw(avg_vol_str),
    ]));
    content.push(Line::from(vec![
        Span::raw("Value:          "),
        Span::styled(format_value(value), Style::default().fg(Color::Cyan)),
    ]));

    // Footer
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "[Enter/Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));

    let main_content = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(main_content, chunks[0]);

    // Render sparkline chart with Y-axis labels
    if let Some(ref chart) = app.detail_chart {
        // Split chart area: Y-axis labels on left, sparkline on right
        let chart_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10), // Y-axis labels
                Constraint::Min(20),    // Sparkline
            ])
            .split(chunks[1]);

        // Convert f64 closes to u64 for sparkline (normalize to 0-100 range)
        let min = chart.low;
        let max = chart.high;
        let range = max - min;
        let data: Vec<u64> = if range > 0.0 {
            chart.closes.iter()
                .map(|&v| ((v - min) / range * 100.0) as u64)
                .collect()
        } else {
            chart.closes.iter().map(|_| 50u64).collect()
        };

        // Y-axis labels (high at top, low at bottom)
        let y_axis_content = vec![
            Line::from(Span::styled(format_price(max), Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(format_price(min), Style::default().fg(Color::Red))),
        ];
        let y_axis = Paragraph::new(y_axis_content)
            .alignment(Alignment::Right)
            .block(Block::default().title(" 3M ").borders(Borders::TOP));
        frame.render_widget(y_axis, chart_chunks[0]);

        // Sparkline
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::TOP))
            .data(&data)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(sparkline, chart_chunks[1]);
    } else if app.chart_loading {
        let loading = Paragraph::new("Loading chart...")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(loading, chunks[1]);
    } else {
        let no_data = Paragraph::new("Chart data unavailable")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(no_data, chunks[1]);
    }
}

fn draw_export_menu(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 30, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" Export Data ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

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

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Format:  ", row_style(sel == 0)),
            Span::styled(
                format!("< {} >", format_str),
                row_style(sel == 0).fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled("              ", row_style(sel == 0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Scope:   ", row_style(sel == 1)),
            Span::styled(
                format!("< {} >", scope_str),
                row_style(sel == 1).fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ),
            Span::styled("            ", row_style(sel == 1)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "        [ Export ]        ",
                if sel == 2 {
                    Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  [←→] Toggle  [Enter] Confirm",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let menu = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(menu, inner_area);
}

fn draw_help(frame: &mut Frame, _app: &App) {
    let area = centered_rect(50, 70, frame.area());
    frame.render_widget(Clear, area);

    let outer_block = Block::default()
        .title(" Help - Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let section = |title: &str| -> Line {
        Line::from(vec![
            Span::styled(format!("─── {} ", title), Style::default().fg(Color::Yellow)),
            Span::styled("───────────────────────────", Style::default().fg(Color::DarkGray)),
        ])
    };

    let binding = |key: &str, desc: &str| -> Line {
        Line::from(vec![
            Span::styled(format!("  {:12}", key), Style::default().fg(Color::Cyan)),
            Span::raw(desc.to_string()),
        ])
    };

    let mut content = vec![
        section("General"),
        binding("q", "Quit"),
        binding("p", "Toggle Watchlist / Portfolio"),
        binding("r", "Refresh quotes"),
        binding("?", "Show this help"),
        binding("Enter", "Stock detail popup"),
        binding("j / ↓", "Move selection down"),
        binding("k / ↑", "Move selection up"),
        Line::from(""),
        section("Watchlist"),
        binding("a", "Add stock symbol"),
        binding("d", "Delete selected stock"),
        binding("h / ←", "Previous watchlist"),
        binding("l / →", "Next watchlist"),
        binding("n", "New watchlist"),
        binding("R", "Rename watchlist"),
        binding("D", "Delete watchlist"),
        Line::from(""),
        section("Portfolio"),
        binding("a", "Add holding"),
        binding("d", "Delete selected holding"),
    ];

    // Add search/export hints if those features exist
    content.push(Line::from(""));
    content.push(section("Other"));
    content.push(binding("/", "Search / filter symbols"));
    content.push(binding("e", "Export data (CSV/JSON)"));

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "  [?/Enter/Esc] Close",
        Style::default().fg(Color::DarkGray),
    )));

    let help_content = Paragraph::new(content).alignment(Alignment::Left);
    frame.render_widget(help_content, inner_area);
}

fn format_market_cap(cap: u64) -> String {
    if cap >= 1_000_000_000_000 {
        format!("{:.2}T", cap as f64 / 1_000_000_000_000.0)
    } else if cap >= 1_000_000_000 {
        format!("{:.2}B", cap as f64 / 1_000_000_000.0)
    } else if cap >= 1_000_000 {
        format!("{:.2}M", cap as f64 / 1_000_000.0)
    } else {
        format!("{}", cap)
    }
}
