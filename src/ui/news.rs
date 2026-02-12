use crate::api::NewsItem;
use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use super::formatters::*;
use super::tables::{ColumnDef, column_constraints, sort_header_row, visible_columns};

const NEWS_COLUMNS: &[ColumnDef] = &[
    ColumnDef { name: "Time",     width: 10, priority: 1 },
    ColumnDef { name: "Source",   width: 20, priority: 2 },
    ColumnDef { name: "Headline", width: 40, priority: 1 },
];

fn news_row(i: usize, item: &NewsItem, vis: &[usize], selected: usize) -> Row<'static> {
    let is_selected = i == selected;
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

    let cells: Vec<Cell> = vis
        .iter()
        .map(|&col| match col {
            0 => Cell::from(format_relative_time(item.published_at)).style(text_style),
            1 => Cell::from(truncate_str(&item.publisher, 18)).style(text_style),
            2 => Cell::from(item.title.clone()).style(bold_text),
            _ => Cell::from(""),
        })
        .collect();

    let row_style = if is_selected {
        Style::default().bg(Color::Rgb(40, 60, 100))
    } else {
        Style::default()
    };
    Row::new(cells).style(row_style)
}

pub fn draw_news(frame: &mut Frame, area: Rect, app: &App) {
    let available_width = area.width.saturating_sub(2);
    let vis = visible_columns(NEWS_COLUMNS, available_width);
    let header = sort_header_row(
        NEWS_COLUMNS,
        &vis,
        app.news_sort_column,
        &app.news_sort_direction,
        Color::Blue,
    );

    let filtered = app.get_filtered_news();
    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, item)| news_row(i, item, &vis, app.news_selected))
        .collect();

    let title = if app.rss_loading {
        " News [Loading...] ".to_string()
    } else {
        format!(" News ({} articles) ", filtered.len())
    };

    let constraints = column_constraints(NEWS_COLUMNS, &vis, Some(2), available_width);
    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    let mut state = TableState::default();
    state.select(Some(app.news_selected));
    frame.render_stateful_widget(table, area, &mut state);
}
