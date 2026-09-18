use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Padding, Row, StatefulWidget, Table, Widget},
};
use strum::IntoEnumIterator;

use crate::{
    app::{App, ConnectionType},
    screens::common::{draw_modal, draw_split_row, highlighted_table, key_hints},
    theme,
};

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let Some(target) = app.selected_target.clone() else {
        tracing::warn!("Connect modal is open with no selected target");
        return;
    };

    let inner_area = draw_modal("Connect", 52, 8, Padding::horizontal(2), area, buf);

    let [target_area, _, table_area, _, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    let mut target_line = vec![Span::styled(
        target.name.clone(),
        Style::default()
            .fg(theme::TEXT)
            .add_modifier(Modifier::BOLD),
    )];
    if let Some(group) = target.group.as_ref() {
        target_line.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
        target_line.push(Span::styled(
            group.name.clone(),
            Style::default().fg(theme::group_color(group.color.as_deref())),
        ));
    }
    Line::from(target_line).render(target_area, buf);

    // The highlighted row is decoded back into a `ConnectionType` by index, so the rows must stay
    // in variant order.
    let rows: Vec<Row> = ConnectionType::iter()
        .map(|connection_type| {
            Row::new([
                Cell::from(connection_type.to_string()).style(Style::default().fg(theme::TEXT)),
                Cell::from(connection_type.description())
                    .style(Style::default().fg(theme::DIM_TEXT)),
            ])
            .height(1)
        })
        .collect();

    let table = highlighted_table(Table::new(
        rows,
        [Constraint::Length(7), Constraint::Min(0)],
    ));

    StatefulWidget::render(
        table,
        table_area,
        buf,
        &mut app.table_connection_selection_state,
    );

    draw_split_row(
        key_hints(&[("↵", "connect")], 3),
        key_hints(&[("esc", "back")], 3),
        footer_area,
        buf,
    );
}
