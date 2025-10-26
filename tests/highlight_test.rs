use std::cell::LazyCell;
use std::fs::File;
use std::sync::LazyLock;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_assets::assets::HighlightingAssets;
use tui_syntax_highlight::Highlighter;

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
thread_local! {
    static ASSETS: LazyCell<HighlightingAssets> = LazyCell::new(HighlightingAssets::from_binary);
}

macro_rules! assert_snapshot {
    ($name:literal, $harness:expr) => {
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($name, $harness.buffer());
        });
    };
}

#[test]
fn highlighter() {
    let highlighter = Highlighter::new(THEMES.themes["base16-ocean.dark"].clone());
    let highlight = highlighter
        .highlight_lines(
            LinesWithEndings::from("select a,b,c from table;\nselect b,c,d from table2;"),
            SYNTAXES.find_syntax_by_name("SQL").unwrap(),
            &SYNTAXES,
        )
        .unwrap();
    assert_snapshot!("highlighter", draw(40, 2, highlight));
}

#[test]
fn highlighter_no_line_number() {
    let highlighter =
        Highlighter::new(THEMES.themes["base16-ocean.dark"].clone()).line_numbers(false);
    let highlight = highlighter
        .highlight_lines(
            LinesWithEndings::from("select a,b,c from table;\nselect b,c,d from table2;"),
            SYNTAXES.find_syntax_by_name("SQL").unwrap(),
            &SYNTAXES,
        )
        .unwrap();
    assert_snapshot!("highlighter_no_line_numbers", draw(40, 2, highlight));
}

#[test]
fn highlighter_override_bg() {
    let highlighter = Highlighter::new(THEMES.themes["base16-ocean.dark"].clone())
        .override_background(Color::Reset);
    let highlight = highlighter
        .highlight_lines(
            LinesWithEndings::from("select a,b,c from table;\nselect b,c,d from table2;"),
            SYNTAXES.find_syntax_by_name("SQL").unwrap(),
            &SYNTAXES,
        )
        .unwrap();
    assert_snapshot!("highlighter_override_bg", draw(40, 2, highlight));
}

#[test]
fn highlighter_template() {
    let highlighter =
        Highlighter::new(THEMES.themes["base16-ocean.dark"].clone()).gutter_template(|n, style| {
            vec![
                Span::raw(n.to_string()),
                Span::raw(" "),
                Span::styled(">", style),
            ]
        });
    let highlight = highlighter
        .highlight_lines(
            LinesWithEndings::from("select a,b,c from table;\nselect b,c,d from table2;"),
            SYNTAXES.find_syntax_by_name("SQL").unwrap(),
            &SYNTAXES,
        )
        .unwrap();
    assert_snapshot!("highlighter_template", draw(40, 2, highlight));
}

#[test]
fn highlight_file_ansi() {
    let theme = ASSETS.with(|a| a.get_theme("ansi").clone());
    let syntaxes = ASSETS.with(|a| a.get_syntax_set().unwrap().clone());
    let syntax = syntaxes.find_syntax_by_name("Rust").unwrap();
    let highlighter = Highlighter::new(theme);
    let highlight = highlighter
        .highlight_reader(
            File::open("./tests/assets/test_file.rs").unwrap(),
            syntax,
            &syntaxes,
        )
        .unwrap();
    assert_snapshot!("highlight_file_ansi", draw(40, 3, highlight));
}

#[test]
fn highlight_range() {
    let highlighter =
        Highlighter::new(THEMES.themes["base16-ocean.dark"].clone()).highlight_range(0..1);
    let highlight = highlighter
        .highlight_lines(
            LinesWithEndings::from("select a,b,c from table;\nselect b,c,d from table2;"),
            SYNTAXES.find_syntax_by_name("SQL").unwrap(),
            &SYNTAXES,
        )
        .unwrap();
    assert_snapshot!("highlight_range", draw(40, 2, highlight));
}

fn draw<W>(width: u16, height: u16, widget: W) -> TestBackend
where
    W: Widget,
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| f.render_widget(widget, f.area()))
        .unwrap();
    terminal.backend().clone()
}
