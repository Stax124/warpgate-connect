use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, StatefulWidget, Table},
};

use crate::app::App;

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    const HEADERS: [&str; 1] = ["Select connection type"];
    const ROWS: [&str; 2] = ["SSH", "SFTP"];

    let header_cells = HEADERS.iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);
    let rows: Vec<Row> = ROWS
        .iter()
        .map(|option| {
            Row::new(vec![
                Cell::from(*option).style(Style::default().fg(Color::White)),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(rows, [Constraint::Fill(1)])
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    StatefulWidget::render(table, area, buf, &mut app.table_connection_selection_state);
}
