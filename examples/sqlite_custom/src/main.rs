use std::error::Error;
use std::io::{Stdout, stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::read;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tui_syntax_highlight::CodeHighlighter;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let syntaxes: SyntaxSet =
        dumps::from_uncompressed_data(include_bytes!("../dumps/sqlite.packdump")).unwrap();
    let themes: ThemeSet = dumps::from_binary(include_bytes!("../dumps/themes.themedump"));

    let mut terminal = setup_terminal()?;

    let highlighter =
        CodeHighlighter::new(themes.themes.get("ansi").unwrap().clone(), syntaxes.clone());
    let highlight = highlighter.highlight_lines(
        "select a,b,c from table;\nselect b,c,d from table2;",
        syntaxes.find_syntax_by_name("SQL").unwrap(),
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
