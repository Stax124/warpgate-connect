use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    app::{App, AppScreen, Modal},
    screens::{
        connect_modal, group_picker, help, logs, main_screen::draw_main_screen, warpgate_settings,
    },
};

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            AppScreen::Main => draw_main_screen(self, area, buf),
            AppScreen::WarpgateSettings => warpgate_settings::draw(self, area, buf),
            AppScreen::Logs => logs::draw(self, area, buf),
        }

        if let Some(modal) = self.modal {
            match modal {
                Modal::Connect => connect_modal::draw(self, area, buf),
                Modal::GroupPicker => group_picker::draw(self, area, buf),
                Modal::Help => help::draw(area, buf),
            }
        }
    }
}
