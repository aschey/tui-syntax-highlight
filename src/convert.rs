#[cfg(feature = "termprofile")]
use termprofile::TermProfile;

/// Converts between [`syntect`] styles and [`ratatui`](ratatui_core) styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Converter {
    #[cfg(feature = "termprofile")]
    profile: TermProfile,
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

impl Converter {
    /// Creates a new [`Converter`].
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "termprofile")]
            profile: TermProfile::TrueColor,
        }
    }

    /// Creates a new [`Converter`] with the given [`TermProfile`].
    #[cfg(feature = "termprofile")]
    pub fn with_profile(profile: TermProfile) -> Self {
        Self { profile }
    }

    /// Converts the syntect [`Style`](syntect::highlighting::Style) to a ratatui
    /// [`Style`](ratatui_core::style::Style).
    pub fn syntect_style_to_tui(
        &self,
        style: syntect::highlighting::Style,
    ) -> ratatui_core::style::Style {
        #[cfg(feature = "termprofile")]
        if self.profile == termprofile::TermProfile::NoTty {
            return ratatui_core::style::Style::new();
        }
        let mut tui_style = ratatui_core::style::Style::new();
        if let Some(fg) = self.syntect_color_to_tui(style.foreground) {
            tui_style = tui_style.fg(fg);
        }
        if let Some(bg) = self.syntect_color_to_tui(style.background) {
            tui_style = tui_style.bg(bg);
        }
        tui_style.add_modifier(syntect_modifiers_to_tui(&style.font_style))
    }

    /// Converts the syntect [`Color`](ratatui_core::style::Color) to a ratatui
    /// [`Color`](ratatui_core::style::Color).
    pub fn syntect_color_to_tui(
        &self,
        color: syntect::highlighting::Color,
    ) -> Option<ratatui_core::style::Color> {
        if color.a == 0 {
            Some(match color.r {
                0x00 => ratatui_core::style::Color::Black,
                0x01 => ratatui_core::style::Color::Red,
                0x02 => ratatui_core::style::Color::Green,
                0x03 => ratatui_core::style::Color::Yellow,
                0x04 => ratatui_core::style::Color::Blue,
                0x05 => ratatui_core::style::Color::Magenta,
                0x06 => ratatui_core::style::Color::Cyan,
                0x07 => ratatui_core::style::Color::Gray,
                0x08 => ratatui_core::style::Color::DarkGray,
                0x09 => ratatui_core::style::Color::LightRed,
                0x0A => ratatui_core::style::Color::LightGreen,
                0x0B => ratatui_core::style::Color::LightYellow,
                0x0C => ratatui_core::style::Color::LightBlue,
                0x0D => ratatui_core::style::Color::LightMagenta,
                0x0E => ratatui_core::style::Color::LightCyan,
                0x0F => ratatui_core::style::Color::White,
                c => ratatui_core::style::Color::Indexed(c),
            })
        } else if color.a == 1 {
            None
        } else {
            #[cfg(feature = "termprofile")]
            return self
                .profile
                .adapt_color(ratatui_core::style::Color::Rgb(color.r, color.g, color.b));
            #[cfg(not(feature = "termprofile"))]
            return Some(ratatui_core::style::Color::Rgb(color.r, color.g, color.b));
        }
    }
}

fn syntect_modifiers_to_tui(
    style: &syntect::highlighting::FontStyle,
) -> ratatui_core::style::Modifier {
    let mut modifier = ratatui_core::style::Modifier::empty();
    if style.intersects(syntect::highlighting::FontStyle::BOLD) {
        modifier |= ratatui_core::style::Modifier::BOLD;
    }
    if style.intersects(syntect::highlighting::FontStyle::ITALIC) {
        modifier |= ratatui_core::style::Modifier::ITALIC;
    }
    if style.intersects(syntect::highlighting::FontStyle::UNDERLINE) {
        modifier |= ratatui_core::style::Modifier::UNDERLINED;
    }
    modifier
}
