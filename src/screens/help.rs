use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Padding, Paragraph, Widget},
};

use crate::{
    screens::common::{draw_modal, draw_split_row, key_hints},
    theme,
};

const KEYS: [(&str, &str); 8] = [
    ("↵", "connect to the highlighted target"),
    ("^G", "pick a group"),
    ("esc", "clear the search, or close this"),
    ("^R", "refresh targets"),
    ("^N", "next screen"),
    ("^A", "select all, in a text field"),
    ("^U", "apply the available update"),
    ("^Q", "quit"),
];

const KEY_WIDTH: usize = 5;

pub fn draw(area: Rect, buf: &mut Buffer) {
    let inner_area = draw_modal(
        "Keys",
        48,
        KEYS.len() as u16 + 6,
        Padding::new(2, 2, 1, 1),
        area,
        buf,
    );

    let [list_area, _, footer_area] = Layout::vertical([
        Constraint::Length(KEYS.len() as u16),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    let lines: Vec<Line> = KEYS
        .iter()
        .map(|(key, action)| {
            Line::from(vec![
                Span::styled(
                    format!("{key:<KEY_WIDTH$}"),
                    Style::default().fg(theme::KEY),
                ),
                Span::styled((*action).to_string(), Style::default().fg(theme::DIM_TEXT)),
            ])
        })
        .collect();

    Paragraph::new(lines).render(list_area, buf);

    draw_split_row(
        Line::raw(""),
        key_hints(&[("esc", "close")], 3),
        footer_area,
        buf,
    );
}
