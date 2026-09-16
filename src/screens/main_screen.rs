use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Table, Widget, Wrap,
    },
};

use crate::{
    app::{App, AppScreen},
    screens::common::{
        Status, current_status, draw_footer, draw_header, draw_prompt_row, draw_rule,
        highlighted_table, key_hints, right_width,
    },
    theme,
    utils::get_color_from_group_color,
};

pub fn draw_main_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    let [
        header_area,
        header_rule_area,
        prompt_area,
        prompt_rule_area,
        body_area,
        body_rule_area,
        footer_area,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let status = current_status(app);

    draw_header(
        AppScreen::Main,
        &app.warpgate_host,
        status,
        header_area,
        buf,
    );
    draw_rule(header_rule_area, buf);
    draw_prompt(app, prompt_area, buf);
    draw_rule(prompt_rule_area, buf);
    if app.filtered_targets.is_empty() {
        draw_empty_body(app, status, body_area, buf);
    } else {
        draw_table(app, body_area, buf);
    }
    draw_rule(body_rule_area, buf);

    let has_highlighted_target = app.table_targets_selection_state.selected().is_some();
    let update_version = app.data.update_available.lock().unwrap().clone();

    draw_footer(
        &[
            ("F1", "keys", true),
            ("↵", "connect", has_highlighted_target),
            ("^G", "group", true),
            ("^R", "refresh", true),
            ("^N", "settings", true),
        ],
        status,
        update_version.as_deref(),
        footer_area,
        buf,
    );
}

fn draw_prompt(app: &App, area: Rect, buf: &mut Buffer) {
    let group_name = app.group_filter.as_ref().map_or("all", |g| g.name.as_str());
    let group_color =
        get_color_from_group_color(app.group_filter.as_ref().and_then(|g| g.color.as_deref()));

    let counter_color = if app.filtered_targets.is_empty() {
        theme::ACCENT
    } else {
        theme::MUTED
    };

    let right = Line::from(vec![
        Span::styled("[ ", Style::default().fg(theme::MUTED)),
        Span::styled(
            group_name.to_string(),
            Style::default()
                .fg(group_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ]    ", Style::default().fg(theme::MUTED)),
        Span::styled(
            format!("{}/{}", app.filtered_targets.len(), app.total_ssh_targets),
            Style::default().fg(counter_color),
        ),
    ]);

    let [prompt_area, right_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(right_width(&right, area)),
    ])
    .areas(area);

    draw_prompt_row(&app.ui_inputs.search_input, prompt_area, buf);
    Paragraph::new(right).render(right_area, buf);
}

/// Stands in for the table when there is nothing to list: a failed fetch, a fetch still running,
/// or a search that matched nothing.
fn draw_empty_body(app: &App, status: Status, area: Rect, buf: &mut Buffer) {
    let fetch_error = {
        // The Report is not Clone, so it is formatted from behind the guard. `{e:?}` would drag
        // the whole backtrace into the frame.
        let targets = app.data.warpgate_targets.lock().unwrap();
        targets.as_ref().err().map(|e| format!("{e}"))
    };

    let lines: Vec<Line> = if let Some(error) = fetch_error {
        vec![
            Line::from(Span::styled(
                "✗  could not load targets",
                Style::default()
                    .fg(theme::ERROR)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::raw(""),
            Line::from(Span::styled(error, Style::default().fg(theme::DIM_TEXT))),
            Line::raw(""),
            key_hints(&[("^R", "retry"), ("^N", "settings")], 6),
        ]
    } else if status == Status::Loading {
        vec![Line::from(vec![
            Span::styled("●", Style::default().fg(theme::WARN)),
            Span::styled(" loading targets…", Style::default().fg(theme::MUTED)),
        ])]
    } else {
        let query = app
            .ui_inputs
            .search_input
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();

        let mut headline = if query.is_empty() {
            vec![Span::styled(
                "no SSH targets here",
                Style::default().fg(theme::DIM_TEXT),
            )]
        } else {
            vec![
                Span::styled("no targets match ", Style::default().fg(theme::DIM_TEXT)),
                Span::styled(
                    query,
                    Style::default()
                        .fg(theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]
        };

        if let Some(group) = app.group_filter.as_ref() {
            headline.push(Span::styled(
                " in group ",
                Style::default().fg(theme::DIM_TEXT),
            ));
            headline.push(Span::styled(
                group.name.clone(),
                Style::default()
                    .fg(get_color_from_group_color(group.color.as_deref()))
                    .add_modifier(Modifier::BOLD),
            ));
        }

        vec![
            Line::from(headline),
            Line::raw(""),
            key_hints(
                &[("^G", "pick another group"), ("esc", "clear the search")],
                6,
            ),
        ]
    };

    // Centred on the wrapped height, so a long error message does not push the key hints below it
    // out of view.
    let wrapped_height: usize = lines
        .iter()
        .map(|line| line.width().div_ceil(usize::from(area.width).max(1)).max(1))
        .sum();
    let top_padding = area
        .height
        .saturating_sub(u16::try_from(wrapped_height).unwrap_or(u16::MAX))
        / 2;

    let body_area = Rect {
        y: area.y + top_padding,
        height: area.height - top_padding,
        ..area
    };

    Paragraph::new(lines)
        .centered()
        .wrap(Wrap { trim: true })
        .render(body_area, buf);
}

/// Spans the name with the query's matched positions picked out.
fn highlighted_name(name: &str, name_indices: &[u32]) -> Line<'static> {
    if name_indices.is_empty() {
        return Line::from(Span::styled(
            name.to_string(),
            Style::default().fg(theme::TEXT),
        ));
    }

    let matched_style = Style::default()
        .fg(theme::ACCENT)
        .add_modifier(Modifier::BOLD);
    let plain_style = Style::default().fg(theme::TEXT);

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_is_matched = false;

    for (position, character) in name.chars().enumerate() {
        let is_matched = name_indices.binary_search(&(position as u32)).is_ok();
        if !run.is_empty() && is_matched != run_is_matched {
            let style = if run_is_matched {
                matched_style
            } else {
                plain_style
            };
            spans.push(Span::styled(std::mem::take(&mut run), style));
        }
        run_is_matched = is_matched;
        run.push(character);
    }

    if !run.is_empty() {
        let style = if run_is_matched {
            matched_style
        } else {
            plain_style
        };
        spans.push(Span::styled(run, style));
    }

    Line::from(spans)
}

pub fn draw_table(app: &mut App, area: Rect, buf: &mut Buffer) {
    const HEADERS: [&str; 3] = ["NAME", "GROUP", "DESCRIPTION"];

    let needs_scrollbar = app.filtered_targets.len() > area.height.saturating_sub(1) as usize;
    let [table_area, scrollbar_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(if needs_scrollbar { 1 } else { 0 }),
    ])
    .areas(area);

    let header_cells = HEADERS
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme::MUTED)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .filtered_targets
        .iter()
        .map(|matched| {
            let target = &matched.target;
            let group_color =
                get_color_from_group_color(target.group.as_ref().and_then(|g| g.color.as_deref()));
            Row::new(vec![
                Cell::from(highlighted_name(&target.name, &matched.name_indices)),
                Cell::from(target.group.as_ref().map_or("", |g| g.name.as_str())).style(
                    Style::default()
                        .fg(group_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(target.description.as_deref().unwrap_or(""))
                    .style(Style::default().fg(theme::DIM_TEXT)),
            ])
            .height(1)
        })
        .collect();

    let longest_name_length = app
        .filtered_targets
        .iter()
        .map(|m| m.target.name.chars().count())
        .max()
        .unwrap_or(0);
    let name_column_width = std::cmp::max(4, longest_name_length + 2) as u16;

    let longest_group_name_length = app
        .filtered_targets
        .iter()
        .map(|m| {
            m.target
                .group
                .as_ref()
                .map_or(0, |g| g.name.chars().count())
        })
        .max()
        .unwrap_or(0);
    let group_column_width = std::cmp::max(6, longest_group_name_length + 2) as u16;

    let table = highlighted_table(Table::new(
        rows,
        [
            Constraint::Length(name_column_width),
            Constraint::Length(group_column_width),
            Constraint::Min(0),
        ],
    ))
    .header(header);

    StatefulWidget::render(
        table,
        table_area,
        buf,
        &mut app.table_targets_selection_state,
    );

    if needs_scrollbar {
        let mut scrollbar_state = ScrollbarState::new(app.filtered_targets.len())
            .position(app.table_targets_selection_state.offset());
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            scrollbar_area,
            buf,
            &mut scrollbar_state,
        );
    }
}
