use crate::serde_schema::{EbmlSchema, Element, ElementType};
use crate::trie::Trie;

use std::collections::{HashMap, HashSet};

use core::ops::{Bound, RangeBounds};
use core::str::FromStr;

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
            return Ok(Self::default());
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
        if s.is_empty() {
            return Ok(Self(Vec::new()));
        }
        Ok(Self(
            s.replace("\\)", ")") // global parent occurrence also uses '\' -> remove...
                .strip_prefix('\\') // each path atom starts with \ -> remove the first...
                .ok_or(Self::Err::MissingPathDivider)?
                .split(|c| c == '\\') // ...then split on '\'
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

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PathAtomsParserError {
    #[error("missing path divider '\\'")]
    MissingPathDivider,
    #[error("invalid global placeholder: {0}")]
    InvalidGlobalPlaceholder(<GlobalPlaceholder as FromStr>::Err),
}

/**
The `EbmlElementModel` object has two jobs:
    1. validate the element model, as defined in the schema
    2. index the element model for easy element navigation
**/
pub(crate) struct EbmlElementModel {
    // u32's are the element ID's
    // ID = `None` -> root document
    pub elements: HashMap<u32, Element>, // the root doesn't have a schema config
    pub parents: HashMap<u32, HashSet<Option<u32>>>, // the root can BE a parent, but will not HAVE a parent
    pub children: HashMap<Option<u32>, HashSet<u32>>, // the root can HAVE children, but will not BE a child
}

impl EbmlElementModel {
    pub fn new(schema: EbmlSchema) -> Result<Self, EbmlElementModelError> {
        // Validate inputs & configuration
        // ...
        // Return `Self` object

        //
        let elems: HashMap<u32, Element> = schema
            .elements
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|elem| (elem.id, elem))
            .collect();

        let pathed_elems: Trie<(GlobalPlaceholder, String), &Element> = elems
            .values()
            .map(|elem| {
                let path_atoms = elem
                    .path
                    .parse::<PathAtoms>()
                    .map_err(EbmlElementModelError::InvalidPath)?
                    .0; // trie should use single path atoms as edges
                Ok((path_atoms, elem))
            })
            .collect::<Result<_, _>>()?;

        let elem_parents: HashMap<u32, HashSet<Option<u32>>> = pathed_elems
            .iter()
            .map(|(path_atoms, elem)| {
                //let expt_first_atom = &[&(Default::default(), "".to_string())];
                //let path_atoms = path_atoms
                //    .strip_prefix(expt_first_atom)
                //    .ok_or_else(|| EbmlElementModelError::NonNullPathPrefix(elem.path.clone()))?;
                let ((global_span, name), parent_path_atoms) = path_atoms
                    .split_last()
                    .ok_or_else(|| EbmlElementModelError::EmptyPath(elem.name.clone()))?;
                if name != &elem.name {
                    return Err(EbmlElementModelError::MismatchedPathName(
                        elem.name.clone(),
                        name.to_string(),
                    ));
                }

                let parent_trie = pathed_elems
                    .subtrie(parent_path_atoms.iter().copied())
                    .expect("path of parent must necessarily exist for a given child");
                if !parent_path_atoms.is_empty() && parent_trie.get([]).is_none() {
                    return Err(EbmlElementModelError::NoDirectParent(elem.name.clone()));
                }
                let mut parent_ids: HashSet<Option<u32>> = parent_trie
                    .iter_depths()
                    .skip_while(|(depth, _elem)| depth < &(global_span.lower_bound as usize))
                    .take_while(|(depth, _elem)| {
                        global_span
                            .upper_bound
                            .map_or(true, |ubnd| depth <= &(ubnd as usize))
                    })
                    .filter(|(_depth, elem)| elem.r#type == ElementType::Master)
                    // v the root trie will have *no* leaf -> treat this as id = None
                    .map(|(_depth, &elem)| Some(elem.id))
                    .collect::<HashSet<_>>();
                if parent_path_atoms.is_empty() && global_span.contains(&0) {
                    parent_ids.insert(None);
                }

                Ok((elem.id, parent_ids))
            })
            .collect::<Result<_, _>>()?;

        let mut elem_children = HashMap::new();
        for (elem_id, parent_ids) in elem_parents.iter() {
            for parent_id in parent_ids.iter() {
                elem_children
                    .entry(*parent_id)
                    .or_insert_with(HashSet::new)
                    .insert(*elem_id);
            }
        }
        for elem_id in elems.keys() {
            elem_children
                .entry(Some(*elem_id))
                .or_insert_with(HashSet::new);
        }

        Ok(Self {
            elements: elems,
            parents: elem_parents,
            children: elem_children,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EbmlElementModelError {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    use crate::serde_schema::*;

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

    #[rstest]
    #[case("", Ok(PathAtoms(Vec::new())))]
    #[case("\\EBML", Ok(PathAtoms(vec![(GlobalPlaceholder::default(), "EBML".to_string())])))]
    #[case("\\EBML\\EBMLVersion", Ok(PathAtoms(vec![
        (GlobalPlaceholder::default(), "EBML".to_string()),
        (GlobalPlaceholder::default(), "EBMLVersion".to_string()),
    ])))]
    #[case("\\(-)Void", Ok(PathAtoms(vec![
        (GlobalPlaceholder{lower_bound: 0, upper_bound: None}, "Void".to_string()),
    ])))]
    fn path_atoms_parse(
        #[case] s: &'static str,
        #[case] expt_result: Result<PathAtoms, PathAtomsParserError>,
    ) {
        assert_eq!(s.parse(), expt_result);
    }

    #[fixture]
    fn schema() -> EbmlSchema {
        EbmlSchema {
            doc_type: "matroska".to_string(),
            version: 4,
            ebml: None,
            elements: Some(vec![
                Element {
                    name: "EBML".to_string(),
                    path: "\\EBML".to_string(),
                    id: 0x1A45DFA3,
                    min_occurs: Some(1),
                    max_occurs: Some(1),
                    range: None,
                    length: None,
                    default: None,
                    r#type: ElementType::Master,
                    unknownsizeallowed: None,
                    recursive: None,
                    recurring: None,
                    minver: None,
                    maxver: None,
                    metadata: None,
                },
                Element {
                    name: "EBMLVersion".to_string(),
                    path: "\\EBML\\EBMLVersion".to_string(),
                    id: 0x4286,
                    min_occurs: Some(1),
                    max_occurs: Some(1),
                    range: Some("not 0".to_string()),
                    length: None,
                    default: Some("1".to_string()),
                    r#type: ElementType::UnsignedInteger,
                    unknownsizeallowed: None,
                    recursive: None,
                    recurring: None,
                    minver: None,
                    maxver: None,
                    metadata: None,
                },
                Element {
                    name: "DocType".to_string(),
                    path: "\\EBML\\DocType".to_string(),
                    id: 0x4282,
                    min_occurs: Some(1),
                    max_occurs: Some(1),
                    range: None,
                    length: Some("&gt;0".to_string()),
                    default: None,
                    r#type: ElementType::String,
                    unknownsizeallowed: None,
                    recursive: None,
                    recurring: None,
                    minver: None,
                    maxver: None,
                    metadata: None,
                },
                Element {
                    name: "Void".to_string(),
                    path: "\\(-\\)Void".to_string(),
                    id: 0xEC,
                    min_occurs: None,
                    max_occurs: Some(1),
                    range: None,
                    length: Some("4".to_string()),
                    default: None,
                    r#type: ElementType::Binary,
                    unknownsizeallowed: None,
                    recursive: None,
                    recurring: None,
                    minver: None,
                    maxver: None,
                    metadata: None,
                },
            ]),
        }
    }

    #[rstest]
    fn ebml_element_model_new(schema: EbmlSchema) {
        let result = EbmlElementModel::new(schema);
        let result = result.unwrap();

        assert_eq!(
            result
                .elements
                .keys()
                .collect::<std::collections::HashSet<_>>(),
            vec![0x1A45DFA3, 0x4286, 0x4282, 0xEC]
                .iter()
                .collect::<std::collections::HashSet<_>>()
        );
        assert_eq!(
            result.parents,
            vec![
                (0x1A45DFA3, vec![None].into_iter().collect::<HashSet<_>>()),
                (
                    0x4286,
                    vec![Some(0x1A45DFA3)].into_iter().collect::<HashSet<_>>()
                ),
                (
                    0x4282,
                    vec![Some(0x1A45DFA3)].into_iter().collect::<HashSet<_>>()
                ),
                (
                    0xEC,
                    vec![None, Some(0x1A45DFA3)]
                        .into_iter()
                        .collect::<HashSet<_>>()
                ),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>()
        );
        assert_eq!(
            result.children,
            vec![
                (
                    None,
                    vec![0x1A45DFA3, 0xEC].into_iter().collect::<HashSet<_>>()
                ),
                (
                    Some(0x1A45DFA3),
                    vec![0x4286, 0x4282, 0xEC]
                        .into_iter()
                        .collect::<HashSet<_>>()
                ),
                (Some(0x4286), HashSet::new()),
                (Some(0x4282), HashSet::new()),
                (Some(0xEC), HashSet::new()),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>()
        );
    }
}
