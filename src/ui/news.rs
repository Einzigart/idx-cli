use super::formatters::*;
use super::tables::{ColumnDef, column_constraints, sort_header_row, visible_columns};
use crate::api::NewsItem;
use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

const NEWS_COLUMNS: &[ColumnDef] = &[
    ColumnDef {
        name: "Time",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "Source",
        width: 20,
        priority: 2,
    },
    ColumnDef {
        name: "Headline",
        width: 40,
        priority: 1,
    },
];
pub(crate) const NEWS_SORTABLE_COLUMNS: usize = 3;

fn news_row(item: &NewsItem, vis: &[usize], is_bookmarked: bool) -> Row<'static> {
    let cells: Vec<Cell> = vis
        .iter()
        .map(|&col| match col {
            0 => Cell::from(format_relative_time(item.published_at)),
            1 => Cell::from(truncate_str(&item.publisher, 18)),
            2 => {
                if is_bookmarked {
                    Cell::from(format!("★ {}", item.title))
                        .style(Style::default().fg(Color::Yellow))
                } else {
                    Cell::from(item.title.clone())
                }
            }
            _ => Cell::from(""),
        })
        .collect();
    Row::new(cells)
}

pub fn draw_news(frame: &mut Frame, area: Rect, app: &mut App) {
    // rows visible = area height - 2 (borders) - 1 (header)
    app.table_viewport_height = area.height.saturating_sub(3) as usize;
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
        .map(|item| {
            let bookmarked = app.config.is_bookmarked(&item.title, item.url.as_deref());
            news_row(item, &vis, bookmarked)
        })
        .collect();

    let title = if app.rss_loading {
        " News [Loading...] ".to_string()
    } else {
        format!(" News ({} articles) ", filtered.len())
    };

    let constraints = column_constraints(NEWS_COLUMNS, &vis, Some(2), available_width);
    let table = Table::new(rows, constraints)
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 60, 100))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(title));

    app.news_table_state.select(Some(app.news_selected));
    frame.render_stateful_widget(table, area, &mut app.news_table_state);
}
