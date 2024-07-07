use std::error::Error;
use std::io::{stdout, Stdout};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui_syntax::{find_syntax_by_name, load_default_themes, load_syntaxes, CodeHighlighter};

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    load_syntaxes!("../dumps/sqlite.packdump");
    load_default_themes();

    let mut terminal = setup_terminal()?;

    let highlighter = CodeHighlighter::new_ansi();
    let text = highlighter.highlight_lines(
        "select a,b,c from table;\nselect b,c,d from table2;",
        Some(find_syntax_by_name("SQL")),
    );
    terminal.draw(|frame| frame.render_widget(text, frame.size()))?;
    std::thread::sleep(Duration::from_secs(3));
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
