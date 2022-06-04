use iron_ebmlem::parser_gen::Builder;

fn main() {
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .expect("no env variable 'CARGO_MANIFEST_DIR'");

    let schema_file = std::io::BufReader::new(
        std::fs::File::open(cargo_path.join("eg_schema.xml")).expect("couldn't open schema file"),
    );

    Builder::new(schema_file)
        .expect("couldn't parse schema file")
        .make_generator()
        .expect("invalid EBML element model")
        .write_package(cargo_path.join("parser"))
        .expect("couldn't write parser crate");
}
