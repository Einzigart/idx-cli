use super::formatters::*;
use crate::api::StockQuote;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

pub(super) struct ColumnDef {
    pub name: &'static str,
    pub width: u16,
    pub priority: u8,
}

const WATCHLIST_COLUMNS: &[ColumnDef] = &[
    ColumnDef {
        name: "Symbol",
        width: 8,
        priority: 1,
    },
    ColumnDef {
        name: "Name",
        width: 22,
        priority: 3,
    },
    ColumnDef {
        name: "Price",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "Change",
        width: 10,
        priority: 2,
    },
    ColumnDef {
        name: "Change %",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "Open",
        width: 10,
        priority: 4,
    },
    ColumnDef {
        name: "High",
        width: 10,
        priority: 4,
    },
    ColumnDef {
        name: "Low",
        width: 10,
        priority: 4,
    },
    ColumnDef {
        name: "Volume",
        width: 12,
        priority: 2,
    },
    ColumnDef {
        name: "Value",
        width: 14,
        priority: 3,
    },
    ColumnDef {
        name: "News",
        width: 5,
        priority: 4,
    },
];
/// Number of sortable columns (excludes non-sortable indicator columns like News)
pub(crate) const WATCHLIST_SORTABLE_COLUMNS: usize = 10;

const PORTFOLIO_COLUMNS: &[ColumnDef] = &[
    ColumnDef {
        name: "Symbol",
        width: 8,
        priority: 1,
    },
    ColumnDef {
        name: "Name",
        width: 22,
        priority: 3,
    },
    ColumnDef {
        name: "Lots",
        width: 6,
        priority: 2,
    },
    ColumnDef {
        name: "Avg Price",
        width: 10,
        priority: 3,
    },
    ColumnDef {
        name: "Last",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "Value",
        width: 12,
        priority: 2,
    },
    ColumnDef {
        name: "Cost",
        width: 12,
        priority: 3,
    },
    ColumnDef {
        name: "P/L",
        width: 12,
        priority: 2,
    },
    ColumnDef {
        name: "P/L %",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "News",
        width: 5,
        priority: 4,
    },
];
/// Number of sortable columns (excludes non-sortable indicator columns like News)
pub(crate) const PORTFOLIO_SORTABLE_COLUMNS: usize = 9;

pub(super) fn visible_columns(columns: &[ColumnDef], available_width: u16) -> Vec<usize> {
    let max_priority = columns.iter().map(|c| c.priority).max().unwrap_or(1);
    let mut visible: Vec<usize> = Vec::new();
    for priority_cutoff in 1..=max_priority {
        let candidate: Vec<usize> = columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.priority <= priority_cutoff)
            .map(|(i, _)| i)
            .collect();
        let total_width: u16 = candidate.iter().map(|&i| columns[i].width).sum();
        if total_width <= available_width {
            visible = candidate;
        } else {
            break;
        }
    }
    if visible.is_empty() {
        visible = columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.priority == 1)
            .map(|(i, _)| i)
            .collect();
    }
    visible
}

fn watchlist_cell(
    col_idx: usize,
    q: &StockQuote,
    bold_text: Style,
    text_style: Style,
    chg_style: Style,
    is_selected: bool,
    has_news: bool,
    has_alert: bool,
) -> Cell<'static> {
    match col_idx {
        0 => {
            let label = if has_alert {
                format!("! {}", q.symbol)
            } else {
                q.symbol.clone()
            };
            let style = if has_alert {
                bold_text.fg(Color::Red)
            } else {
                bold_text
            };
            Cell::from(label).style(style)
        }
        1 => Cell::from(truncate_str(&q.short_name, 20)).style(text_style),
        2 => Cell::from(format_price(q.price)).style(bold_text),
        3 => Cell::from(format_change(q.change)).style(chg_style),
        4 => Cell::from(format!("{:+.2}%", q.change_percent)).style(chg_style),
        5 => Cell::from(format_price(q.open)).style(text_style),
        6 => Cell::from(format_price(q.high)).style(text_style),
        7 => Cell::from(format_price(q.low)).style(text_style),
        8 => Cell::from(format_volume(q.volume)).style(text_style),
        9 => {
            let value = q.price * q.volume as f64;
            let style = if is_selected {
                text_style.fg(Color::Cyan)
            } else {
                Style::default()
            };
            Cell::from(format_value(value)).style(style)
        }
        10 => {
            if has_news {
                Cell::from(" * ").style(Style::default().fg(Color::Yellow))
            } else {
                Cell::from("")
            }
        }
        _ => Cell::from(""),
    }
}

fn watchlist_row(
    i: usize,
    symbol: &str,
    quote: Option<&StockQuote>,
    vis: &[usize],
    selected_index: usize,
    has_news: bool,
    has_alert: bool,
) -> Row<'static> {
    let is_selected = i == selected_index;
    if let Some(q) = quote {
        let (change_color, selected_change_color) = if q.change >= 0.0 {
            (Color::Green, Color::LightGreen)
        } else {
            (Color::Red, Color::LightRed)
        };
        let chg_color = if is_selected {
            selected_change_color
        } else {
            change_color
        };
        let text_style = if is_selected {
            Style::default().fg(Color::White)
        } else {
            Style::default()
        };
        let bold_text = if is_selected {
            text_style.add_modifier(Modifier::BOLD)
        } else {
            text_style
        };
        let chg_style = Style::default().fg(chg_color).add_modifier(Modifier::BOLD);
        let cells: Vec<Cell> = vis
            .iter()
            .map(|&col| {
                watchlist_cell(
                    col,
                    q,
                    bold_text,
                    text_style,
                    chg_style,
                    is_selected,
                    has_news,
                    has_alert,
                )
            })
            .collect();
        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(40, 80, 120))
        } else {
            Style::default()
        };
        Row::new(cells).style(row_style)
    } else {
        let style = if is_selected {
            Style::default()
                .bg(Color::Rgb(40, 80, 120))
                .fg(Color::White)
        } else {
            Style::default()
        };
        let cells: Vec<Cell> = vis
            .iter()
            .map(|&col| match col {
                0 => {
                    let label = if has_alert {
                        format!("! {}", symbol)
                    } else {
                        symbol.to_string()
                    };
                    if has_alert {
                        Cell::from(label).style(Style::default().fg(Color::Red))
                    } else {
                        Cell::from(label)
                    }
                }
                10 => {
                    if has_news {
                        Cell::from(" * ").style(Style::default().fg(Color::Yellow))
                    } else {
                        Cell::from("")
                    }
                }
                _ => Cell::from("-"),
            })
            .collect();
        Row::new(cells).style(style)
    }
}

pub(super) fn sort_header_row(
    columns: &[ColumnDef],
    vis: &[usize],
    sort_col: Option<usize>,
    sort_dir: &crate::app::SortDirection,
    color: Color,
) -> Row<'static> {
    let cells: Vec<Cell> = vis
        .iter()
        .map(|&i| {
            let name = columns[i].name;
            let label = if sort_col == Some(i) {
                format!("{} {}", name, sort_dir.indicator())
            } else {
                name.to_string()
            };
            Cell::from(label).style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        })
        .collect();
    Row::new(cells).height(1)
}

pub(super) fn column_constraints(
    columns: &[ColumnDef],
    vis: &[usize],
    stretch_col: Option<usize>,
    available_width: u16,
) -> Vec<Constraint> {
    let total_vis_width: u16 = vis.iter().map(|&i| columns[i].width).sum();
    let extra = available_width.saturating_sub(total_vis_width);

    match stretch_col {
        // Single stretch column absorbs all extra space (e.g. Name in watchlist)
        Some(sc) if extra > 0 => vis
            .iter()
            .map(|&i| {
                if i == sc {
                    Constraint::Min(columns[i].width)
                } else {
                    Constraint::Length(columns[i].width)
                }
            })
            .collect(),
        // No stretch column: fixed widths, trailing spacer absorbs the rest
        _ => {
            let mut constraints: Vec<Constraint> = vis
                .iter()
                .map(|&i| Constraint::Length(columns[i].width))
                .collect();
            if extra > 0 {
                constraints.push(Constraint::Min(0));
            }
            constraints
        }
    }
}

pub fn draw_watchlist(frame: &mut Frame, area: Rect, app: &mut App) {
    app.table_viewport_height = area.height.saturating_sub(3) as usize;
    let available_width = area.width.saturating_sub(2);
    let vis = visible_columns(WATCHLIST_COLUMNS, available_width);
    let header = sort_header_row(
        WATCHLIST_COLUMNS,
        &vis,
        app.watchlist_sort_column,
        &app.watchlist_sort_direction,
        Color::Yellow,
    );

    let watchlist = app.get_filtered_watchlist();
    let rows: Vec<Row> = watchlist
        .iter()
        .enumerate()
        .map(|(i, (symbol, quote))| {
            let has_news = app.has_recent_news(symbol);
            let has_alert = app.config.has_active_alerts(symbol);
            watchlist_row(i, symbol, *quote, &vis, app.selected_index, has_news, has_alert)
        })
        .collect();

    let constraints = column_constraints(WATCHLIST_COLUMNS, &vis, Some(1), available_width);
    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Watchlist "));

    app.watchlist_table_state.select(Some(app.selected_index));
    frame.render_stateful_widget(table, area, &mut app.watchlist_table_state);
}

fn portfolio_cell(
    col_idx: usize,
    holding: &crate::config::Holding,
    short_name: &str,
    metrics: (f64, f64, f64, f64, f64),
    styles: (Style, Style, Style),
    has_news: bool,
    has_alert: bool,
) -> Cell<'static> {
    let (curr_price, value, cost, pl, pl_percent) = metrics;
    let (bold_text, text_style, pl_style) = styles;
    match col_idx {
        0 => {
            let label = if has_alert {
                format!("! {}", holding.symbol)
            } else {
                holding.symbol.clone()
            };
            let style = if has_alert { bold_text.fg(Color::Red) } else { bold_text };
            Cell::from(label).style(style)
        }
        1 => Cell::from(truncate_str(short_name, 20)).style(text_style),
        2 => Cell::from(format!("{}", holding.lots)).style(text_style),
        3 => Cell::from(format_price(holding.avg_price)).style(text_style),
        4 => Cell::from(format_price(curr_price)).style(text_style),
        5 => Cell::from(format_value(value)).style(text_style),
        6 => Cell::from(format_value(cost)).style(text_style),
        7 => Cell::from(format_pl(pl)).style(pl_style),
        8 => Cell::from(format!("{:+.2}%", pl_percent)).style(pl_style),
        9 => {
            if has_news {
                Cell::from(" * ").style(Style::default().fg(Color::Yellow))
            } else {
                Cell::from("")
            }
        }
        _ => Cell::from(""),
    }
}

fn portfolio_row(
    i: usize,
    holding: &crate::config::Holding,
    app: &App,
    vis: &[usize],
    has_news: bool,
    has_alert: bool,
) -> (Row<'static>, f64, f64) {
    let is_selected = i == app.portfolio_selected;
    let quote = app.quotes.get(&holding.symbol);
    let curr_price = quote.map(|q| q.price).unwrap_or(0.0);
    let short_name = quote.map(|q| q.short_name.as_str()).unwrap_or("-");
    let (value, cost, pl, pl_percent) = holding.pl_metrics(curr_price);

    let pl_color = if pl >= 0.0 { Color::Green } else { Color::Red };
    let selected_pl_color = if pl >= 0.0 {
        Color::LightGreen
    } else {
        Color::LightRed
    };
    let chg_color = if is_selected {
        selected_pl_color
    } else {
        pl_color
    };
    let text_style = if is_selected {
        Style::default().fg(Color::White)
    } else {
        Style::default()
    };
    let bold_text = if is_selected {
        text_style.add_modifier(Modifier::BOLD)
    } else {
        text_style
    };
    let pl_style = Style::default().fg(chg_color).add_modifier(Modifier::BOLD);

    let cells: Vec<Cell> = vis
        .iter()
        .map(|&col| {
            portfolio_cell(
                col,
                holding,
                short_name,
                (curr_price, value, cost, pl, pl_percent),
                (bold_text, text_style, pl_style),
                has_news,
                has_alert,
            )
        })
        .collect();
    let row_style = if is_selected {
        Style::default().bg(Color::Rgb(80, 40, 80))
    } else {
        Style::default()
    };
    (Row::new(cells).style(row_style), value, cost)
}

pub fn draw_portfolio(frame: &mut Frame, area: Rect, app: &mut App) {
    app.table_viewport_height = area.height.saturating_sub(3) as usize;
    let available_width = area.width.saturating_sub(2);
    let vis = visible_columns(PORTFOLIO_COLUMNS, available_width);
    let header = sort_header_row(
        PORTFOLIO_COLUMNS,
        &vis,
        app.portfolio_sort_column,
        &app.portfolio_sort_direction,
        Color::Magenta,
    );

    let mut total_value = 0.0;
    let mut total_cost = 0.0;
    let filtered = app.get_filtered_portfolio();
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_orig_idx, holding))| {
            let has_news = app.has_recent_news(&holding.symbol);
            let has_alert = app.config.has_active_alerts(&holding.symbol);
            let (row, value, cost) = portfolio_row(i, holding, app, &vis, has_news, has_alert);
            total_value += value;
            total_cost += cost;
            row
        })
        .collect();

    let total_pl = total_value - total_cost;
    let total_pl_pct = if total_cost > 0.0 {
        (total_pl / total_cost) * 100.0
    } else {
        0.0
    };
    let total_pl_color = if total_pl >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };
    let title = format!(
        " Portfolio | Value: {} | P/L: {} ({:+.2}%) ",
        format_value(total_value),
        format_pl(total_pl),
        total_pl_pct
    );

    let constraints = column_constraints(PORTFOLIO_COLUMNS, &vis, Some(1), available_width);
    let table = Table::new(rows, constraints).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(total_pl_color)),
    );

    app.portfolio_table_state
        .select(Some(app.portfolio_selected));
    frame.render_stateful_widget(table, area, &mut app.portfolio_table_state);
}
