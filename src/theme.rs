use ratatui::{style::Color, text::Span};

use phf_macros::phf_map;

pub const TEXT: Color = Color::from_u32(0x00ebdbb2);
pub const TEXT_ALT: Color = Color::from_u32(0x00a89984);
pub const BG0: Color = Color::from_u32(0x00282828);
pub const BG1: Color = Color::from_u32(0x003c3836);
pub const BG2: Color = Color::from_u32(0x00504945);

pub const GRAY: Color = Color::from_u32(0x928374);
pub const RED: Color = Color::from_u32(0xfb4934);
pub const GREEN: Color = Color::from_u32(0xb8bb26);
pub const YELLOW: Color = Color::from_u32(0xfabd2f);
pub const BLUE: Color = Color::from_u32(0x83a598);
pub const PURPLE: Color = Color::from_u32(0xd3869b);
pub const AQUA: Color = Color::from_u32(0x8ec07c);
pub const FG: Color = Color::from_u32(0xebdbb2);
pub const ORANGE: Color = Color::from_u32(0x00fe8019);

pub const TERM_COLORS: phf::Map<&'static str, Color> = phf_map! {
    "Red" => RED,
    "Green" => GREEN,
    "Yellow" => YELLOW,
    "Blue" => BLUE,
    "Purple" => PURPLE,
    "Aqua" => AQUA,
    "Gray" => GRAY,
    "Orange" => ORANGE,
};

lazy_static::lazy_static! {
    pub static ref TERM_COLORS_REGEX: String = format!(
        "({})",
        TERM_COLORS.keys().cloned().collect::<Vec<_>>().join("|")
    );
}

pub fn unicode_icon<'a>(icon: u32, color: Color) -> Span<'a> {
    let mut c = String::from(char::from_u32(icon).unwrap_or('X'));
    c.push(' ');
    Span::styled(c, color)
}
