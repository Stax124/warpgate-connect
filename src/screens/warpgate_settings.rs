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
    let [
        url_area,
        username_area,
        token_area,
        port_area,
        fill_area,
        status_bar_area,
    ] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_warpgate_url_input_screen(app, url_area, buf);
    draw_warpgate_token_input_screen(app, token_area, buf);
    draw_warpgate_username_input_screen(app, username_area, buf);
    draw_warpgate_port_input_screen(app, port_area, buf);

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

pub fn draw_warpgate_url_input_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    validate_url(&mut app.ui_inputs.warpgate_url_input);
    app.ui_inputs.warpgate_url_input.render(area, buf);
}

pub fn draw_warpgate_token_input_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    validate_token(&mut app.ui_inputs.warpgate_token_input);
    app.ui_inputs.warpgate_token_input.render(area, buf);
}

pub fn draw_warpgate_username_input_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    app.ui_inputs.warpgate_username_input.render(area, buf);
}

pub fn draw_warpgate_port_input_screen(app: &mut App, area: Rect, buf: &mut Buffer) {
    app.ui_inputs.warpgate_port_input.render(area, buf);
}
