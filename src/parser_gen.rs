// interface loosely based on that of bindgen: https://crates.io/crates/bindgen

use crate::serde_schema::EbmlSchema;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

struct Builder {
    schema: EbmlSchema,
}

impl Builder {
    pub fn new(schema: EbmlSchema) -> Self {
        Self { schema }
    }

    pub fn generate() -> Result<Parsers, ()> {
        todo!()
    }
}

struct Parsers;

impl Parsers {
    fn new() -> Self {
        Self
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_ref())?;
        self.write(Box::new(file))?;
        Ok(())
    }

    pub fn write<'a>(&self, mut writer: Box<dyn io::Write + 'a>) -> io::Result<()> {
        todo!();
    }
}
