use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Widget};
use ratatui::{buffer::Buffer, layout::Rect};
use tui_logger::TuiLoggerWidget;

use crate::app::App;
use crate::screens::common::draw_status_bar;

pub fn draw(app: &mut App, area: Rect, buf: &mut Buffer) {
    let loading_guard = app.data.loading_targets.lock().unwrap();
    let is_loading = *loading_guard;
    drop(loading_guard);

    let [logs_area, status_bar_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

    TuiLoggerWidget::default()
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Logs ")
                .title_style(Style::default().bold().fg(ratatui::style::Color::Yellow)),
        )
        .state(&app.logger_state)
        .render(logs_area, buf);

    draw_status_bar(app, status_bar_area, buf, &is_loading);
}
