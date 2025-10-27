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
use ratatui::layout::Alignment;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};
use syntect_assets::assets::HighlightingAssets;
use tui_syntax_highlight::Highlighter;

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

thread_local! {
    static ASSETS: LazyCell<HighlightingAssets> = LazyCell::new(HighlightingAssets::from_binary);
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let theme = ASSETS.with(|a| a.get_theme("Nord").clone());
    let highlighter = Highlighter::new(theme);
    let syntaxes = ASSETS.with(|a| a.get_syntax_set().cloned())?;
    let syntax = syntaxes
        .find_syntax_by_name("Rust")
        .expect("syntax missing");
    let highlight = highlighter.highlight_reader(
        File::open("./examples/sqlite_custom/build.rs")?,
        syntax,
        &syntaxes,
    )?;

    let bg = highlighter.get_background_color().unwrap_or_default();
    // Set the background on the text container so it matches.
    let paragraph = Paragraph::new(highlight).bg(bg).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(0))
            .title("Syntax Highlight!")
            .title_alignment(Alignment::Center),
    );
    terminal.draw(|frame| {
        frame.render_widget(paragraph, frame.area());
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
