use syntect::dumps::dump_to_uncompressed_file;
use syntect::parsing::SyntaxSetBuilder;

fn main() {
    println!("cargo:rerun-if-changed=./assets");

    let mut builder = SyntaxSetBuilder::new();
    builder.add_plain_text_syntax();
    builder.add_from_folder("./assets/SQL", true).unwrap();
    let ss = builder.build();
    dump_to_uncompressed_file(&ss, "./dumps/sqlite.packdump").unwrap();
}
