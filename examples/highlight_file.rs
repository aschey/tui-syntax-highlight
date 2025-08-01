use std::error::Error;
use std::fs::File;
use std::io::{Stdout, stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::read;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::style::Color;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tui_syntax_highlight::Highlighter;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let syntaxes = SyntaxSet::load_defaults_newlines();
    let themes = ThemeSet::load_defaults();
    let highlighter = Highlighter::new(themes.themes["base16-ocean.dark"].clone())
        .override_background(Color::Reset);
    let syntax = syntaxes.find_syntax_by_name("SQL").unwrap();
    let highlight = highlighter.highlight_reader(
        File::open("./examples/sqlite_custom/build.rs").unwrap(),
        syntax,
        &syntaxes,
    );
    terminal.draw(|frame| frame.render_widget(highlight, frame.area()))?;
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
