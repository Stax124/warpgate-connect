use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Paragraph, Widget},
};

use crate::app::App;

pub fn draw_status_bar(app: &App, area: Rect, buf: &mut Buffer, is_loading: &bool) {
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(area);

    Paragraph::new(format!(
        "Status: {}",
        if *is_loading {
            "Loading targets..."
        } else if app.data.warpgate_targets.lock().unwrap().is_ok() {
            "Ready"
        } else {
            "Error loading targets"
        }
    ))
    .alignment(Alignment::Left)
    .fg(match *is_loading {
        true => Color::Yellow,
        false => {
            if app.data.warpgate_targets.lock().unwrap().is_ok() {
                Color::Green
            } else {
                Color::Red
            }
        }
    })
    .bold()
    .render(left_area, buf);

    let update_text = {
        let update_available = app.data.update_available.lock().unwrap();
        if let Some(ref version) = *update_available {
            format!(" | [U]pdate v{}", version)
        } else {
            String::new()
        }
    };

    Paragraph::new(format!("[R]efresh | [N]ext page | [Q]uit{}", update_text))
        .alignment(Alignment::Right)
        .fg(Color::Cyan)
        .bold()
        .render(right_area, buf);
}
