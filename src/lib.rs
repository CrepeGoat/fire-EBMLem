#[macro_use]
mod parser;
mod element_defs;
mod parser_gen;
mod serde_schema;
mod stream;
mod trie;

mod eg_codegen {
    mod element_defs;
    mod integration;
    mod parser;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
