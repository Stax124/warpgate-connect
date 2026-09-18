use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::app::{App, AppScreen, Modal};

pub mod common;
pub mod connect_modal;
pub mod group_picker;
pub mod help;
pub mod logs;
pub mod settings;
pub mod targets;

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            AppScreen::Targets => targets::draw(self, area, buf),
            AppScreen::Settings => settings::draw(self, area, buf),
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
