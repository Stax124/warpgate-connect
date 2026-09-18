use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Padding, Paragraph, Widget};
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_textarea::TextArea;
use strum::IntoEnumIterator;

use crate::app::{App, AppScreen, WarpgateSettingsScreenInput};
use crate::screens::common::{
    centered_rect, current_status, draw_card, draw_footer, draw_header, draw_rule,
};
use crate::theme;
use crate::utils::first_line;

const CARD_WIDTH: u16 = 62;
const LABEL_WIDTH: u16 = 11;

/// Catches the common mistake of entering the Warpgate dashboard URL instead of the API URL.
fn check_url_for_known_path(text_area: &TextArea) -> Option<String> {
    const KNOWN_PATH: &str = "/@warpgate/api/targets";

    if first_line(text_area).trim().ends_with(KNOWN_PATH) {
        return None;
    }

    Some(format!("! should end with {KNOWN_PATH}"))
}

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let [
        header_area,
        header_rule_area,
        body_area,
        body_rule_area,
        footer_area,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let status = current_status(app);
    draw_header(AppScreen::Settings, "config.toml", status, header_area, buf);
    draw_rule(header_rule_area, buf);
    draw_settings_card(app, body_area, buf);
    draw_rule(body_rule_area, buf);

    draw_footer(
        &[
            ("F1", "keys", true),
            ("↵", "save", true),
            ("⇥", "next field", true),
            ("^N", "logs", true),
        ],
        status,
        app.update_available.as_deref(),
        footer_area,
        buf,
    );
}

fn draw_settings_card(app: &mut App, area: Rect, buf: &mut Buffer) {
    // One row per field, plus the URL warning's own row, which is always reserved so that the
    // card does not shift by a row when the warning appears.
    let card_area = centered_rect(CARD_WIDTH, 9, area);

    let inner_area = draw_card("Warpgate", Padding::new(2, 2, 1, 1), card_area, buf);

    let [
        url_area,
        url_warning_area,
        username_area,
        token_area,
        port_area,
    ] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner_area);

    let selected = app.warpgate_selected_input;

    // Zipped against the enum so that the draw order cannot drift from the order `Tab` walks.
    for (input, field_area) in
        WarpgateSettingsScreenInput::iter().zip([url_area, username_area, token_area, port_area])
    {
        let [label_area, value_area] =
            Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Fill(1)])
                .areas(field_area);

        Line::from(Span::styled(
            input.label(),
            if input == selected {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::MUTED)
            },
        ))
        .render(label_area, buf);

        app.get_warpgate_input_by_enum(input)
            .render(value_area, buf);
    }

    if let Some(warning) = check_url_for_known_path(&app.ui_inputs.warpgate_url_input) {
        let [_, warning_area] =
            Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Fill(1)])
                .areas(url_warning_area);
        Paragraph::new(warning)
            .style(Style::default().fg(theme::WARN))
            .render(warning_area, buf);
    }
}
