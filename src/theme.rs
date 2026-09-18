use ratatui::style::Color;

pub const ACCENT: Color = Color::Yellow;
pub const KEY: Color = Color::Cyan;
pub const OK: Color = Color::Green;
pub const WARN: Color = Color::Yellow;
pub const ERROR: Color = Color::Red;
pub const MUTED: Color = Color::DarkGray;
pub const TEXT: Color = Color::White;
pub const DIM_TEXT: Color = Color::Gray;
pub const SELECTION_BG: Color = Color::DarkGray;

/// Maps the colour name Warpgate assigns a target group onto the palette.
pub fn group_color(color_name: Option<&str>) -> Color {
    match color_name {
        Some("Primary") => Color::Blue,
        Some("Danger") => Color::Red,
        Some("Warning") => Color::Yellow,
        Some("Success") => Color::Green,
        _ => Color::Gray,
    }
}
