#[cfg(feature = "profile")]
use termprofile::TermProfile;

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
                .adapt_color(ratatui::style::Color::Rgb(color.r, color.g, color.b));
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
