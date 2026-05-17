use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_textarea::TextArea;

use crate::app::App;
use crate::screens::common::draw_status_bar;

fn validate_url(text_area: &mut TextArea) {
    let url = text_area.lines()[0].trim();
    let is_valid = url.starts_with("http://") || url.starts_with("https://");

    text_area.set_cursor_line_style(Style::default().fg(if is_valid {
        Color::Green
    } else {
        Color::Red
    }));
}

/// Check that the URL ends like a known path (e.g. /api/v1), if not, display a warning that the user might have entered the wrong URL.
/// Should catch common mistakes like entering the warpgate dashboard URL instead of the API URL.
fn check_url_for_known_path(text_area: &mut TextArea) -> Option<String> {
    let url = text_area.lines()[0].trim();
    let known_path = "/@warpgate/api/targets";
    let has_known_path = url.ends_with(known_path);

    if has_known_path {
        return None;
    }

    Some(format!("Warning: URL does not end with {}", known_path))
}

fn validate_token(text_area: &mut TextArea) {
    let token = text_area.lines()[0].trim();
    let is_valid = !token.is_empty();

    text_area.set_cursor_line_style(Style::default().fg(if is_valid {
        Color::Green
    } else {
        Color::Red
    }));
}

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let has_known_path = check_url_for_known_path(&mut app.ui_inputs.warpgate_url_input);

    let [
        url_area,
        url_warning_area,
        username_area,
        token_area,
        port_area,
        fill_area,
        status_bar_area,
    ] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(if has_known_path.is_some() { 1 } else { 0 }),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    // URL input
    validate_url(&mut app.ui_inputs.warpgate_url_input);
    app.ui_inputs.warpgate_url_input.render(url_area, buf);

    // Optional warning if the URL does not end with a known path
    if let Some(warning) = has_known_path {
        Paragraph::new(warning)
            .style(Style::default().fg(Color::Yellow).bold())
            .render(url_warning_area, buf);
    }

    // Token input
    validate_token(&mut app.ui_inputs.warpgate_token_input);
    app.ui_inputs.warpgate_token_input.render(token_area, buf);

    // Username and port inputs (no validation)
    app.ui_inputs
        .warpgate_username_input
        .render(username_area, buf);
    app.ui_inputs.warpgate_port_input.render(port_area, buf);

    // Instructions at the bottom
    Paragraph::new(Line::from(vec![
        Span::raw("Press "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" to save settings"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().padding(Padding::top(1)))
    .render(fill_area, buf);

    draw_status_bar(app, status_bar_area, buf, &false);
}
