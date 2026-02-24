use super::formatters::*;
use super::tables::{ColumnDef, column_constraints, sort_header_row, visible_columns};
use crate::app::App;
use crate::config::Bookmark;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

const BOOKMARK_COLUMNS: &[ColumnDef] = &[
    ColumnDef {
        name: "Bookmarked",
        width: 10,
        priority: 1,
    },
    ColumnDef {
        name: "Published",
        width: 10,
        priority: 2,
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
pub(crate) const BOOKMARK_SORTABLE_COLUMNS: usize = 4;

fn bookmark_row(bookmark: &Bookmark, vis: &[usize]) -> Row<'static> {
    let read_style = if bookmark.read {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };
    let cells: Vec<Cell> = vis
        .iter()
        .map(|&col| match col {
            0 => Cell::from(format_relative_time(bookmark.bookmarked_at)).style(read_style),
            1 => Cell::from(format_relative_time(bookmark.published_at)).style(read_style),
            2 => Cell::from(truncate_str(&bookmark.source, 18)).style(read_style),
            3 => {
                let prefix = if bookmark.read { "  " } else { "● " };
                let headline = format!("{}{}", prefix, bookmark.headline);
                Cell::from(headline).style(if bookmark.read {
                    read_style
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                })
            }
            _ => Cell::from(""),
        })
        .collect();
    Row::new(cells)
}

pub fn draw_bookmarks(frame: &mut Frame, area: Rect, app: &mut App) {
    app.table_viewport_height = area.height.saturating_sub(3) as usize;
    let available_width = area.width.saturating_sub(2);
    let vis = visible_columns(BOOKMARK_COLUMNS, available_width);
    let header = sort_header_row(
        BOOKMARK_COLUMNS,
        &vis,
        app.bookmark_sort_column,
        &app.bookmark_sort_direction,
        Color::Green,
    );

    let filtered = app.get_filtered_bookmarks();
    let unread_count = filtered.iter().filter(|b| !b.read).count();
    let rows: Vec<Row> = filtered
        .iter()
        .map(|bookmark| bookmark_row(bookmark, &vis))
        .collect();

    let title = if filtered.is_empty() {
        " Bookmarks (empty) ".to_string()
    } else if unread_count > 0 {
        format!(
            " Bookmarks ({} articles, {} unread) ",
            filtered.len(),
            unread_count
        )
    } else {
        format!(" Bookmarks ({} articles) ", filtered.len())
    };

    let constraints = column_constraints(BOOKMARK_COLUMNS, &vis, Some(3), available_width);
    let table = Table::new(rows, constraints)
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 80, 40))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(title));

    app.bookmark_table_state.select(Some(app.bookmark_selected));
    frame.render_stateful_widget(table, area, &mut app.bookmark_table_state);
}
