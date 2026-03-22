use ratatui::style::Color;

pub const BG: Color = Color::Rgb(13, 13, 18);
pub const BORDER: Color = Color::Rgb(45, 45, 65);
pub const BORDER_FOCUS: Color = Color::Rgb(90, 130, 255);
pub const TEXT: Color = Color::Rgb(195, 195, 210);
pub const DIM: Color = Color::Rgb(80, 80, 100);
pub const ACCENT: Color = Color::Rgb(90, 130, 255);
pub const KEY_COLOR: Color = Color::Rgb(170, 215, 255);
pub const RUNNING: Color = Color::Rgb(70, 215, 120);
pub const WARN: Color = Color::Rgb(255, 185, 55);
pub const SEL_BG: Color = Color::Rgb(28, 28, 42);
pub const PRESS_COL: Color = Color::Rgb(110, 195, 255);
pub const RELEASE_COL: Color = Color::Rgb(255, 140, 110);
pub const WAIT_COL: Color = Color::Rgb(195, 195, 90);

pub fn border_color(focused: bool) -> Color {
    if focused {
        BORDER_FOCUS
    } else {
        BORDER
    }
}
