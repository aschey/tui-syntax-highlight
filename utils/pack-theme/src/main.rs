use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;

fn main() {
    let ts = ThemeSet::load_from_folder("./utils/pack-theme/themes").unwrap();
    dump_to_file(&ts, "./ratatui-syntax/dumps/themes.themedump").unwrap();
}
