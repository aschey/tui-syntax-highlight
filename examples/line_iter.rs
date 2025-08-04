use std::cell::LazyCell;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Stdout, stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::read;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::text::Text;
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
    let syntaxes = ASSETS.with(|a| a.get_syntax_set().cloned()).unwrap();
    let syntax = syntaxes.find_syntax_by_name("Rust").unwrap();

    let highlighter = Highlighter::new(theme);
    let mut lines = highlighter.create_line_highlighter(syntax);
    let reader = File::open("./examples/sqlite_custom/build.rs").unwrap();
    let mut reader = BufReader::new(reader);

    let line_number_style = highlighter.calculate_line_number_style();
    let mut line = String::new();
    let mut formatted = Vec::new();
    let mut i = 1;
    while reader.read_line(&mut line)? > 0 {
        let highlighted =
            highlighter.highlight_line(&mut line, &mut lines, i, line_number_style, &syntaxes)?;
        formatted.push(highlighted);
        line.clear();
        i += 1;
    }
    let text = Text::from_iter(formatted);
    terminal.draw(|frame| {
        frame.render_widget(text, frame.area());
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
