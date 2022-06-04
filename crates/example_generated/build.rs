use iron_ebmlem::parser_gen::SchemaParser;

fn main() {
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .expect("no env variable 'CARGO_MANIFEST_DIR'");

    let schema_file = std::io::BufReader::new(
        std::fs::File::open(cargo_path.join("eg_schema.xml")).expect("couldn't open schema file"),
    );

    SchemaParser::new(schema_file)
        .expect("couldn't parse schema file")
        .generate()
        .expect("couldn't make parser writer")
        .write_package(cargo_path.join("parser"))
        .expect("couldn't write parser crate");
}
