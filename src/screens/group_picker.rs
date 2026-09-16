use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Cell, Padding, Row, StatefulWidget, Table},
};

use crate::{
    app::App,
    screens::common::{draw_modal, draw_rule, draw_split_row, highlighted_table, key_hints},
    theme,
    utils::get_color_from_group_color,
};

const VISIBLE_ROWS: u16 = 8;

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let row_count = u16::try_from(app.group_picker_matches.len())
        .unwrap_or(VISIBLE_ROWS)
        .clamp(1, VISIBLE_ROWS);

    let inner_area = draw_modal(
        "Group",
        38,
        row_count + 6,
        Padding::horizontal(1),
        area,
        buf,
    );

    let [prompt_area, rule_area, table_area, _, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(row_count),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    crate::screens::common::draw_prompt_row(&app.ui_inputs.group_picker_input, prompt_area, buf);
    draw_rule(rule_area, buf);

    let rows: Vec<Row> = app
        .group_picker_matches
        .iter()
        .filter_map(|index| app.group_picker_rows.get(*index))
        .map(|row| {
            let (symbol, color) = match row.group.as_ref() {
                Some(group) => ("●", get_color_from_group_color(group.color.as_deref())),
                None => ("○", theme::MUTED),
            };

            Row::new(vec![
                Cell::from(symbol).style(Style::default().fg(color)),
                Cell::from(
                    row.group
                        .as_ref()
                        .map_or("all", |group| group.name.as_str()),
                )
                .style(Style::default().fg(theme::TEXT)),
                Cell::from(Line::from(row.count.to_string()).right_aligned())
                    .style(Style::default().fg(theme::MUTED)),
            ])
            .height(1)
        })
        .collect();

    let table = highlighted_table(Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(5),
        ],
    ));

    StatefulWidget::render(table, table_area, buf, &mut app.table_group_picker_state);

    draw_split_row(
        key_hints(&[("↵", "apply")], 3),
        key_hints(&[("esc", "cancel")], 3),
        footer_area,
        buf,
    );
}
