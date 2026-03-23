use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, StatefulWidget, Table, Widget},
};

use crate::{app::App, screens::common::draw_status_bar, utils::get_color_from_group_color};

pub fn draw_main_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    let loading_guard = app.data.loading_targets.lock().unwrap();
    let is_loading = *loading_guard;
    drop(loading_guard);

    crate::utils::try_set_first_index(app, &is_loading);

    let [top_area, middle_area, bottom_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_search_bar_and_filters(app, top_area, buf);
    draw_table(app, middle_area, buf);
    draw_status_bar(app, bottom_area, buf, &is_loading);
}

pub fn draw_table(app: &mut App, area: Rect, buf: &mut Buffer) {
    const HEADERS: [&str; 3] = ["Group", "Name", "Description"];

    let header_cells = HEADERS.iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .filtered_targets
        .iter()
        .map(|target| {
            let group_color = crate::utils::get_color_from_group_color(
                &target.group.as_ref().and_then(|g| g.color.clone()),
            );
            Row::new(vec![
                Cell::from(target.group.as_ref().map_or("", |g| g.name.as_str())).style(
                    Style::default()
                        .fg(group_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(target.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(target.description.as_deref().unwrap_or("").to_string())
                    .style(Style::default().add_modifier(Modifier::DIM)),
            ])
            .height(1)
        })
        .collect();

    let longest_group_name_length = app
        .filtered_targets
        .iter()
        .map(|t| t.group.as_ref().map_or(0, |g| g.name.len()))
        .max()
        .unwrap_or(0);
    let first_column_width = std::cmp::max(6, longest_group_name_length + 2) as u16;

    let longest_name_length = app
        .filtered_targets
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    let name_column_width = std::cmp::max(4, longest_name_length + 2) as u16;

    // Check if we have any target selected, if not, select the first one (if it exists)
    if app.table_state.selected().is_none() && !rows.is_empty() {
        app.table_state.select(Some(0));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(first_column_width),
            Constraint::Length(name_column_width),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    StatefulWidget::render(table, area, buf, &mut app.table_state);
}

pub fn draw_search_bar_and_filters(app: &App, area: Rect, buf: &mut Buffer) {
    let [left_side, right_side] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(20)]).areas(area);

    app.ui_inputs.search_input.render(left_side, buf);

    let filter_block = Block::default()
        .borders(Borders::ALL)
        .fg({
            if app.group_filter.is_some() {
                Color::Green
            } else {
                Color::DarkGray
            }
        })
        .border_style(Style::new().add_modifier(Modifier::BOLD))
        .title(" Group [⇧G] ")
        .padding(Padding::horizontal(1));

    let selected_group = app.group_filter.as_ref();
    let group_name = selected_group.map(|g| g.name.as_str()).unwrap_or("All");

    Paragraph::new(group_name)
        .block(filter_block)
        .alignment(Alignment::Right)
        .fg(get_color_from_group_color(
            &app.group_filter.as_ref().and_then(|v| v.color.clone()),
        ))
        .add_modifier(Modifier::BOLD)
        .render(right_side, buf);
}
