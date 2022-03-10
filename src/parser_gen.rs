// interface loosely based on that of bindgen: https://crates.io/crates/bindgen

use crate::serde_schema::{EbmlSchema, Element};
use crate::trie::Trie;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

/**
The `Builder` object has the following responsibilities:

- validate the schema objects read directly from the schema
- perform all pre-processing in advance required to write the source routines for parsing

**/

pub struct Builder {
    schema: EbmlSchema,
}

impl Builder {
    pub fn new(schema: EbmlSchema) -> Self {
        Self { schema }
    }

    pub fn generate(self) -> Result<Parsers, ()> {
        // Validate inputs & configuration
        // ...
        // Return `Parsers` object
        let pathed_elems = self
            .schema
            .elements
            .into_iter()
            .map(|elem| {
                let path_atoms = elem
                    .path
                    .replace("\\)", ")") // global parent occurrence also uses '\' -> remove before...
                    .split(|c| c == '\\') // ...split on '\'
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                (path_atoms, elem)
            })
            .collect::<Trie<_, _>>();

        todo!()
    }
}

/**
The `Parsers` object has only one job: write valid Rust code as described in the schema.
Everything else (reading the schema, validating the element definitions & hierarchy, etc.)
should be done elsewhere.

**/

pub struct Parsers {
    // u32's are the element ID's
    // ID = `None` -> root document
    elements: HashMap<u32, Element>, // the root doesn't have a schema config
    parents: HashMap<u32, Vec<Option<u32>>>, // the root can BE a parent, but will not HAVE a parent
    children: HashMap<Option<u32>, Vec<u32>>, // the root can HAVE children, but will not BE a child
}

impl Parsers {
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
