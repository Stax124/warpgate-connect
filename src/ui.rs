use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    app::{App, AppScreen},
    screens::{main_screen::draw_main_screen, warpgate_settings},
};

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            AppScreen::Main => draw_main_screen(self, area, buf),
            AppScreen::WarpgateSettings => warpgate_settings::draw(self, area, buf),
        }
    }
}
