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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPlaceholder {
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
                upper_bound: Some(0),
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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum GlobalPlaceHolderParserError {
    #[error("invalid bound: {0}")]
    InvalidBound(<u64 as FromStr>::Err),
    #[error("missing token {0}")]
    MissingToken(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathAtoms(Vec<(GlobalPlaceholder, String)>);

impl FromStr for PathAtoms {
    type Err = PathAtomsParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.replace("\\)", ")") // global parent occurrence also uses '\' -> remove before...
                .split(|c| c == '\\') // ...split on '\'
                .map(|s| {
                    let divider = s.find(')').map_or(0, |i| i + 1);
                    let (s1, s2) = s.split_at(divider);
                    Ok((
                        s1.parse().map_err(Self::Err::InvalidGlobalPlaceholder)?,
                        s2.to_string(),
                    ))
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PathAtomsParserError {
    #[error("invalid global placeholder: {0}")]
    InvalidGlobalPlaceholder(<GlobalPlaceholder as FromStr>::Err),
}

#[derive(Debug)]
pub struct Builder {
    schema: EbmlSchema,
}

impl Builder {
    pub fn new(schema: EbmlSchema) -> Self {
        Self { schema }
    }

    pub fn generate(self) -> Result<Parsers, BuilderGenerateError> {
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

        let pathed_elems: Trie<(GlobalPlaceholder, String), &Element> = elems
            .values()
            .map(|elem| {
                let path_atoms = elem
                    .path
                    .parse::<PathAtoms>()
                    .map_err(BuilderGenerateError::InvalidPath)?
                    .0; // trie should use single path atoms as edges
                Ok((path_atoms, elem))
            })
            .collect::<Result<_, _>>()?;

        let elem_parents: HashMap<u32, Vec<Option<u32>>> = pathed_elems
            .iter()
            .map(|(path_atoms, elem)| {
                let expt_first_atom = &[&(Default::default(), "".to_string())];
                let path_atoms = path_atoms
                    .strip_prefix(expt_first_atom)
                    .ok_or_else(|| BuilderGenerateError::NonNullPathPrefix(elem.path.clone()))?;
                let ((global_span, name), parent_path_atoms) = path_atoms
                    .split_last()
                    .ok_or_else(|| BuilderGenerateError::EmptyPath(elem.name.clone()))?;
                if name != &elem.name {
                    return Err(BuilderGenerateError::MismatchedPathName(
                        elem.name.clone(),
                        name.to_string(),
                    ));
                }

                let mut parent_ids: Vec<Option<u32>> = pathed_elems
                    .subtrie(parent_path_atoms.iter().copied())
                    .ok_or_else(|| BuilderGenerateError::NoDirectParent(elem.name.clone()))?
                    .iter_depths()
                    .skip_while(|(depth, _trie)| depth < &(global_span.lower_bound as usize))
                    .take_while(|(depth, _trie)| {
                        global_span
                            .upper_bound
                            .map_or(true, |ubnd| depth <= &(ubnd as usize))
                    })
                    // v the root trie will have *no* leaf -> treat this as id = None
                    .map(|(_depth, &elem)| Some(elem.id))
                    .collect::<Vec<_>>();
                if parent_path_atoms.is_empty() && global_span.contains(&0) {
                    parent_ids.push(None);
                }

                Ok((elem.id, parent_ids))
            })
            .collect::<Result<_, _>>()?;

        let mut elem_children = HashMap::new();
        for (elem_id, parent_ids) in elem_parents.iter() {
            for parent_id in parent_ids.iter() {
                elem_children
                    .entry(*parent_id)
                    .or_insert_with(Vec::new)
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

#[derive(thiserror::Error, Debug)]
pub enum BuilderGenerateError {
    #[error("invalid path: {0}")]
    InvalidPath(<PathAtoms as FromStr>::Err),
    #[error("empty path for element name {0}")]
    EmptyPath(String),
    #[error("inconsistent element name: element labeled {0}, but path terminated with {1}")]
    MismatchedPathName(String, String),
    #[error("no direct parent element in path {0}")]
    NoDirectParent(String),
    #[error("expected a null prefix in path {0}")]
    NonNullPathPrefix(String),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("", Ok(GlobalPlaceholder{lower_bound: 0, upper_bound: Some(0)}))]
    #[case("(-)", Ok(GlobalPlaceholder{lower_bound: 0, upper_bound: None}))]
    #[case("(1-)", Ok(GlobalPlaceholder{lower_bound: 1, upper_bound: None}))]
    #[case("(-3)", Ok(GlobalPlaceholder{lower_bound: 0, upper_bound: Some(3)}))]
    #[case("(2-3)", Ok(GlobalPlaceholder{lower_bound: 2, upper_bound: Some(3)}))]
    #[case("(23)", Err(GlobalPlaceHolderParserError::MissingToken('-')))]
    #[case("2-3)", Err(GlobalPlaceHolderParserError::MissingToken('(')))]
    #[case("(2-3", Err(GlobalPlaceHolderParserError::MissingToken(')')))]
    fn global_placeholder_parse(
        #[case] s: &'static str,
        #[case] expt_result: Result<GlobalPlaceholder, GlobalPlaceHolderParserError>,
    ) {
        assert_eq!(s.parse(), expt_result);
    }
}
