use std::cell::LazyCell;
use std::error::Error;
use std::fs::File;
use std::io::{Stdout, stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::read;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use syntect_assets::assets::HighlightingAssets;
use termprofile::{DetectorSettings, TermProfile};
use tui_syntax_highlight::Highlighter;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

thread_local! {
    static ASSETS: LazyCell<HighlightingAssets> = LazyCell::new(HighlightingAssets::from_binary);
}

fn main() -> Result<()> {
    let term_profile = TermProfile::detect(&stdout(), DetectorSettings::with_query()?);

    let mut terminal = setup_terminal()?;
    let theme = ASSETS.with(|a| {
        if term_profile >= TermProfile::Ansi256 {
            a.get_theme("Nord").clone()
        } else {
            a.get_theme("ansi").clone()
        }
    });
    let highlighter = Highlighter::with_profile(theme, term_profile);
    let syntaxes = ASSETS.with(|a| a.get_syntax_set().cloned())?;
    let syntax = syntaxes.find_syntax_by_name("Rust").unwrap();
    let highlight = highlighter.highlight_reader(
        File::open("./examples/sqlite_custom/build.rs")?,
        syntax,
        &syntaxes,
    )?;
    terminal.draw(|frame| {
        frame.render_widget(highlight, frame.area());
    })?;
    read()?;
    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
