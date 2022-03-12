// interface loosely based on that of bindgen: https://crates.io/crates/bindgen

use crate::serde_schema::{EbmlSchema, Element};
use crate::trie::Trie;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

use core::ops::{Bound, RangeBounds};
use std::str::FromStr;

/**
The `Builder` object has the following responsibilities:

- validate the schema objects read directly from the schema
- perform all pre-processing in advance required to write the source routines for parsing

**/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlobalPlaceholder {
    lower_bound: u64,
    upper_bound: Option<u64>,
}

impl RangeBounds<u64> for GlobalPlaceholder {
    fn start_bound(&self) -> Bound<&u64> {
        Bound::Included(&self.lower_bound)
    }

    fn end_bound(&self) -> Bound<&u64> {
        match self.upper_bound.as_ref() {
            Some(b) => Bound::Included(b),
            None => Bound::Unbounded,
        }
    }
}

impl FromStr for GlobalPlaceholder {
    type Err = GlobalPlaceHolderParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self {
                lower_bound: 0,
                upper_bound: None,
            });
        }

        let s = s
            .strip_prefix('(')
            .ok_or(Self::Err::MissingToken('('))?
            .strip_suffix(')')
            .ok_or(Self::Err::MissingToken(')'))?;

        let (s1, s2) = s.split_once('-').ok_or(Self::Err::MissingToken('-'))?;

        Ok(Self {
            lower_bound: if s1.is_empty() {
                0
            } else {
                s1.parse().map_err(Self::Err::InvalidBound)?
            },
            upper_bound: if s2.is_empty() {
                None
            } else {
                Some(s2.parse().map_err(Self::Err::InvalidBound)?)
            },
        })
    }
}

impl Default for GlobalPlaceholder {
    fn default() -> Self {
        Self {
            lower_bound: 0,
            upper_bound: Some(0),
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum GlobalPlaceHolderParserError {
    #[error("invalid bound: {0}")]
    InvalidBound(<u64 as FromStr>::Err),
    #[error("missing token {0}")]
    MissingToken(char),
}

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

        //
        let elems: HashMap<u32, Element> = self
            .schema
            .elements
            .into_iter()
            .map(|elem| (elem.id, elem))
            .collect();

        let pathed_elems = elems
            .values()
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

        let elem_parents = pathed_elems
            .iter()
            .map(|(fullpath, elem)| {
                let (fullname, parent_path) = fullpath.split_last().unwrap();
                let global_span: GlobalPlaceholder =
                    fullname.strip_suffix(&elem.name).unwrap().parse().unwrap();

                let parent_ids = if global_span == Default::default() {
                    vec![(!parent_path.is_empty())
                        .then(|| pathed_elems.get(parent_path.iter().copied()).unwrap().id)]
                } else {
                    todo!()
                };

                (elem.id, parent_ids)
            })
            .collect::<HashMap<_, _>>();

        let mut elem_children = HashMap::new();
        for (elem_id, parent_ids) in elem_parents.iter() {
            for parent_id in parent_ids.iter() {
                elem_children
                    .entry(*parent_id)
                    .or_insert(Vec::new())
                    .push(*elem_id);
            }
        }

        Ok(Parsers {
            elements: elems,
            parents: elem_parents,
            children: elem_children,
        })
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
