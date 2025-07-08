use ratatui::{style::Color, text::Span};

pub const TEXT: Color = Color::from_u32(0x00ebdbb2);
pub const TEXT_ALT: Color = Color::from_u32(0x00a89984);
pub const BG0: Color = Color::from_u32(0x00282828);
pub const BG1: Color = Color::from_u32(0x003c3836);
// pub const BG2: Color = Color::from_u32(0x00504945);
pub const BLUE: Color = Color::from_u32(0x0083a598);
pub const ORANG: Color = Color::from_u32(0x00fe8019);

pub fn unicode_icon<'a>(icon: u32, color: Color) -> Span<'a> {
    let mut c = String::from(char::from_u32(icon).unwrap_or('X'));
    c.push(' ');
    Span::styled(c, color)
}
