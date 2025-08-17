#[cfg(feature = "profile")]
use termprofile::{TermProfile, anstyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Converter {
    #[cfg(feature = "profile")]
    profile: TermProfile,
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

impl Converter {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "profile")]
            profile: TermProfile::TrueColor,
        }
    }

    #[cfg(feature = "profile")]
    pub fn with_profile(profile: TermProfile) -> Self {
        Self { profile }
    }

    pub fn syntect_style_to_tui(
        &self,
        style: syntect::highlighting::Style,
    ) -> ratatui::style::Style {
        #[cfg(feature = "profile")]
        if self.profile == termprofile::TermProfile::NoTty {
            return ratatui::style::Style::new();
        }
        let mut tui_style = ratatui::style::Style::new();
        if let Some(fg) = self.syntect_color_to_tui(style.foreground) {
            tui_style = tui_style.fg(fg);
        }
        if let Some(bg) = self.syntect_color_to_tui(style.background) {
            tui_style = tui_style.bg(bg);
        }
        tui_style.add_modifier(syntect_modifiers_to_tui(&style.font_style))
    }

    pub fn syntect_color_to_tui(
        &self,
        color: syntect::highlighting::Color,
    ) -> Option<ratatui::style::Color> {
        if color.a == 0 {
            Some(match color.r {
                0x00 => ratatui::style::Color::Black,
                0x01 => ratatui::style::Color::Red,
                0x02 => ratatui::style::Color::Green,
                0x03 => ratatui::style::Color::Yellow,
                0x04 => ratatui::style::Color::Blue,
                0x05 => ratatui::style::Color::Magenta,
                0x06 => ratatui::style::Color::Cyan,
                0x07 => ratatui::style::Color::Gray,
                0x08 => ratatui::style::Color::DarkGray,
                0x09 => ratatui::style::Color::LightRed,
                0x0A => ratatui::style::Color::LightGreen,
                0x0B => ratatui::style::Color::LightYellow,
                0x0C => ratatui::style::Color::LightBlue,
                0x0D => ratatui::style::Color::LightMagenta,
                0x0E => ratatui::style::Color::LightCyan,
                0x0F => ratatui::style::Color::White,
                c => ratatui::style::Color::Indexed(c),
            })
        } else if color.a == 1 {
            None
        } else {
            #[cfg(feature = "profile")]
            return self
                .profile
                .adapt_color(anstyle::RgbColor(color.r, color.g, color.b))
                .map(anstyle_color_to_tui);
            #[cfg(not(feature = "profile"))]
            return Some(ratatui::style::Color::Rgb(color.r, color.g, color.b));
        }
    }
}

fn syntect_modifiers_to_tui(style: &syntect::highlighting::FontStyle) -> ratatui::style::Modifier {
    let mut modifier = ratatui::style::Modifier::empty();
    if style.intersects(syntect::highlighting::FontStyle::BOLD) {
        modifier |= ratatui::style::Modifier::BOLD;
    }
    if style.intersects(syntect::highlighting::FontStyle::ITALIC) {
        modifier |= ratatui::style::Modifier::ITALIC;
    }
    if style.intersects(syntect::highlighting::FontStyle::UNDERLINE) {
        modifier |= ratatui::style::Modifier::UNDERLINED;
    }
    modifier
}

// TODO: remove when ratatui 0.30 is released
#[cfg(feature = "profile")]
pub(crate) fn anstyle_color_to_tui(color: anstyle::Color) -> ratatui::style::Color {
    match color {
        anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)) => ratatui::style::Color::Rgb(r, g, b),
        anstyle::Color::Ansi256(anstyle::Ansi256Color(i)) => ratatui::style::Color::Indexed(i),
        anstyle::Color::Ansi(ansi) => anstyle_ansi_to_tui(ansi),
    }
}

#[cfg(feature = "profile")]
fn anstyle_ansi_to_tui(color: anstyle::AnsiColor) -> ratatui::style::Color {
    match color {
        anstyle::AnsiColor::Black => ratatui::style::Color::Black,
        anstyle::AnsiColor::Red => ratatui::style::Color::Red,
        anstyle::AnsiColor::Green => ratatui::style::Color::Green,
        anstyle::AnsiColor::Yellow => ratatui::style::Color::Yellow,
        anstyle::AnsiColor::Blue => ratatui::style::Color::Blue,
        anstyle::AnsiColor::Magenta => ratatui::style::Color::Magenta,
        anstyle::AnsiColor::Cyan => ratatui::style::Color::Cyan,
        anstyle::AnsiColor::White => ratatui::style::Color::Gray,
        anstyle::AnsiColor::BrightBlack => ratatui::style::Color::DarkGray,
        anstyle::AnsiColor::BrightRed => ratatui::style::Color::LightRed,
        anstyle::AnsiColor::BrightGreen => ratatui::style::Color::LightGreen,
        anstyle::AnsiColor::BrightYellow => ratatui::style::Color::LightYellow,
        anstyle::AnsiColor::BrightBlue => ratatui::style::Color::LightBlue,
        anstyle::AnsiColor::BrightMagenta => ratatui::style::Color::LightMagenta,
        anstyle::AnsiColor::BrightCyan => ratatui::style::Color::LightCyan,
        anstyle::AnsiColor::BrightWhite => ratatui::style::Color::White,
    }
}

#[cfg(feature = "profile")]
pub(crate) fn tui_color_to_anstyle(color: ratatui::style::Color) -> Option<anstyle::Color> {
    Some(match color {
        ratatui::style::Color::Rgb(r, g, b) => anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)),
        ratatui::style::Color::Indexed(i) => anstyle::Color::Ansi256(anstyle::Ansi256Color(i)),
        ratatui::style::Color::Reset => None?,
        ratatui::style::Color::Black => anstyle::Color::Ansi(anstyle::AnsiColor::Black),
        ratatui::style::Color::Red => anstyle::Color::Ansi(anstyle::AnsiColor::Red),
        ratatui::style::Color::Green => anstyle::Color::Ansi(anstyle::AnsiColor::Green),
        ratatui::style::Color::Yellow => anstyle::Color::Ansi(anstyle::AnsiColor::Yellow),
        ratatui::style::Color::Blue => anstyle::Color::Ansi(anstyle::AnsiColor::Blue),
        ratatui::style::Color::Magenta => anstyle::Color::Ansi(anstyle::AnsiColor::Magenta),
        ratatui::style::Color::Cyan => anstyle::Color::Ansi(anstyle::AnsiColor::Cyan),
        ratatui::style::Color::Gray => anstyle::Color::Ansi(anstyle::AnsiColor::White),
        ratatui::style::Color::DarkGray => anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlack),
        ratatui::style::Color::LightRed => anstyle::Color::Ansi(anstyle::AnsiColor::BrightRed),
        ratatui::style::Color::LightGreen => anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen),
        ratatui::style::Color::LightYellow => {
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightYellow)
        }
        ratatui::style::Color::LightBlue => anstyle::Color::Ansi(anstyle::AnsiColor::BrightBlue),
        ratatui::style::Color::LightMagenta => {
            anstyle::Color::Ansi(anstyle::AnsiColor::BrightMagenta)
        }
        ratatui::style::Color::LightCyan => anstyle::Color::Ansi(anstyle::AnsiColor::BrightCyan),
        ratatui::style::Color::White => anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite),
    })
}

#[cfg(feature = "profile")]
pub(crate) fn tui_style_to_anstyle(style: ratatui::style::Style) -> anstyle::Style {
    let mut anstyle_style = anstyle::Style::new();
    if let Some(fg) = style.fg {
        let fg = tui_color_to_anstyle(fg);
        anstyle_style = anstyle_style.fg_color(fg);
    }
    if let Some(bg) = style.bg {
        let bg = tui_color_to_anstyle(bg);
        anstyle_style = anstyle_style.bg_color(bg);
    }
    anstyle_style = anstyle_style.effects(tui_modifiers_to_anstyle(style.add_modifier));
    anstyle_style
}

#[cfg(feature = "profile")]
fn tui_modifiers_to_anstyle(modifier: ratatui::style::Modifier) -> anstyle::Effects {
    let mut effects = anstyle::Effects::new();
    if modifier.contains(ratatui::style::Modifier::BOLD) {
        effects |= anstyle::Effects::BOLD;
    }
    if modifier.contains(ratatui::style::Modifier::DIM) {
        effects |= anstyle::Effects::DIMMED;
    }
    if modifier.contains(ratatui::style::Modifier::ITALIC) {
        effects |= anstyle::Effects::ITALIC;
    }
    if modifier.contains(ratatui::style::Modifier::UNDERLINED) {
        effects |= anstyle::Effects::UNDERLINE;
    }
    if modifier.contains(ratatui::style::Modifier::SLOW_BLINK)
        || modifier.contains(ratatui::style::Modifier::RAPID_BLINK)
    {
        effects |= anstyle::Effects::BLINK;
    }
    if modifier.contains(ratatui::style::Modifier::REVERSED) {
        effects |= anstyle::Effects::INVERT;
    }
    if modifier.contains(ratatui::style::Modifier::HIDDEN) {
        effects |= anstyle::Effects::HIDDEN;
    }
    if modifier.contains(ratatui::style::Modifier::CROSSED_OUT) {
        effects |= anstyle::Effects::STRIKETHROUGH;
    }
    effects
}

#[cfg(feature = "profile")]
pub(crate) fn anstyle_style_to_tui(style: anstyle::Style) -> ratatui::style::Style {
    ratatui::style::Style {
        fg: style.get_fg_color().map(anstyle_color_to_tui),
        bg: style.get_bg_color().map(anstyle_color_to_tui),
        add_modifier: anstyle_effects_to_tui(style.get_effects()),
        ..Default::default()
    }
}

#[cfg(feature = "profile")]
fn anstyle_effects_to_tui(effect: anstyle::Effects) -> ratatui::style::Modifier {
    let mut modifier = ratatui::style::Modifier::empty();
    if effect.contains(anstyle::Effects::BOLD) {
        modifier |= ratatui::style::Modifier::BOLD;
    }
    if effect.contains(anstyle::Effects::DIMMED) {
        modifier |= ratatui::style::Modifier::DIM;
    }
    if effect.contains(anstyle::Effects::ITALIC) {
        modifier |= ratatui::style::Modifier::ITALIC;
    }
    if effect.contains(anstyle::Effects::UNDERLINE)
        || effect.contains(anstyle::Effects::DOUBLE_UNDERLINE)
        || effect.contains(anstyle::Effects::CURLY_UNDERLINE)
        || effect.contains(anstyle::Effects::DOTTED_UNDERLINE)
        || effect.contains(anstyle::Effects::DASHED_UNDERLINE)
    {
        modifier |= ratatui::style::Modifier::UNDERLINED;
    }
    if effect.contains(anstyle::Effects::BLINK) {
        modifier |= ratatui::style::Modifier::SLOW_BLINK;
    }
    if effect.contains(anstyle::Effects::INVERT) {
        modifier |= ratatui::style::Modifier::REVERSED;
    }
    if effect.contains(anstyle::Effects::HIDDEN) {
        modifier |= ratatui::style::Modifier::HIDDEN;
    }
    if effect.contains(anstyle::Effects::STRIKETHROUGH) {
        modifier |= ratatui::style::Modifier::CROSSED_OUT;
    }
    modifier
}
