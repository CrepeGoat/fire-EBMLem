#[macro_use]
mod parser;
mod element_defs;
mod stream;

mod eg_codegen {
    mod element_defs;
    mod parser;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
