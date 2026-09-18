use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Widget};
use ratatui::{buffer::Buffer, layout::Rect};
use tui_logger::TuiLoggerWidget;

use crate::app::{App, AppScreen};
use crate::screens::common::{current_status, draw_footer, draw_header, draw_rule};
use crate::theme;

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let [
        header_area,
        header_rule_area,
        logs_area,
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
    draw_header(
        AppScreen::Logs,
        &app.warpgate_host,
        status,
        header_area,
        buf,
    );
    draw_rule(header_rule_area, buf);

    TuiLoggerWidget::default()
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Logs ")
                .title_style(Style::default().bold().fg(theme::ACCENT)),
        )
        .state(&app.logger_state)
        .render(logs_area, buf);

    draw_rule(body_rule_area, buf);

    draw_footer(
        &[
            ("F1", "keys", true),
            ("^N", "targets", true),
            ("^R", "refresh", true),
        ],
        status,
        app.update_available.as_deref(),
        footer_area,
        buf,
    );
}
