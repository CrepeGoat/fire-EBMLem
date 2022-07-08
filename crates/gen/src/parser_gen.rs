// interface loosely based on that of bindgen: https://crates.io/crates/bindgen

use crate::serde_schema::{from_reader, EbmlSchema, Element, ElementType};
use crate::trie::Trie;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use core::ops::{Bound, RangeBounds};
use core::str::FromStr;

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

#[derive(Debug)]
pub struct Builder {
    schema: EbmlSchema,
}

impl Builder {
    pub fn new<R: std::io::Read>(schema: R) -> Result<Self, serde_xml_rs::Error> {
        Ok(Self {
            schema: from_reader(schema)?,
        })
    }

    pub fn generate(self) -> Result<Parsers, BuilderGenerateError> {
        // Validate inputs & configuration
        // ...
        // Return `Parsers` object

        //
        let elems: HashMap<u32, Element> = self
            .schema
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
                    .map_err(BuilderGenerateError::InvalidPath)?
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
                //    .ok_or_else(|| BuilderGenerateError::NonNullPathPrefix(elem.path.clone()))?;
                let ((global_span, name), parent_path_atoms) = path_atoms
                    .split_last()
                    .ok_or_else(|| BuilderGenerateError::EmptyPath(elem.name.clone()))?;
                if name != &elem.name {
                    return Err(BuilderGenerateError::MismatchedPathName(
                        elem.name.clone(),
                        name.to_string(),
                    ));
                }

                let parent_trie = pathed_elems
                    .subtrie(parent_path_atoms.iter().copied())
                    .expect("path of parent must necessarily exist for a given child");
                if !parent_path_atoms.is_empty() && parent_trie.get([]).is_none() {
                    return Err(BuilderGenerateError::NoDirectParent(elem.name.clone()));
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
    parents: HashMap<u32, HashSet<Option<u32>>>, // the root can BE a parent, but will not HAVE a parent
    children: HashMap<Option<u32>, HashSet<u32>>, // the root can HAVE children, but will not BE a child
}

impl Parsers {
    pub fn write_element_defs<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(
            r#"
            #[allow(unused_imports)]
            use crate::base::element_defs::{
                BinaryElementDef, DateElementDef, ElementDef, FloatElementDef, IntElementDef, MasterElementDef,
                Range, StringElementDef, UIntElementDef, Utf8ElementDef,
            };

            use core::ops::Bound;
            "#.as_bytes()
        )?;

        for element in self.elements.values() {
            write!(
                writer,
                r#"
                #[derive(Debug, Clone, PartialEq)]
                pub struct {name}Def;

                impl ElementDef for {name}Def {{
                    const ID: u32 = {id};
                    const PATH: &'static str = r"{path}";

                    const MIN_OCCURS: usize = {min_occurs};
                    const MAX_OCCURS: Option<usize> = {max_occurs};
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = {recurring};
                    const MIN_VERSION: u64 = {minver};
                    const MAX_VERSION: Option<u64> = {maxver};
                }}
                "#,
                name = element.name,
                id = element.id,
                path = element.path,
                min_occurs = element.min_occurs.unwrap_or(0),
                max_occurs = element
                    .max_occurs
                    .map(|value| format!("Some({value})"))
                    .unwrap_or_else(|| "None".to_string()),
                recurring = element.recurring.unwrap_or(false),
                minver = element.minver.unwrap_or(1),
                maxver = element
                    .maxver
                    .map(|value| format!("Some({value})"))
                    .unwrap_or_else(|| "None".to_string()),
            )?;

            match element.r#type {
                ElementType::Master => write!(
                    writer,
                    r#"
                    impl MasterElementDef for {name}Def {{
                        const UNKNOWN_SIZE_ALLOWED: bool = {unknown_size_allowed};
                        const RECURSIVE: bool = {recursive};
                    }}
                    "#,
                    name = element.name,
                    unknown_size_allowed = element.unknownsizeallowed.unwrap_or(false),
                    recursive = element.recursive.unwrap_or(false),
                ),
                ElementType::SignedInteger => write!(
                    writer,
                    r#"
                    impl IntElementDef for {name}Def {{
                        const RANGE: Range<i64> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                        const DEFAULT: Option<i64> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::UnsignedInteger => write!(
                    writer,
                    r#"
                    impl UIntElementDef for {name}Def {{
                        const RANGE: Range<u64> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                        const DEFAULT: Option<u64> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::Float => write!(
                    writer,
                    r#"
                    impl FloatElementDef for {name}Def {{
                        const RANGE: Range<f64> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                        const DEFAULT: Option<f64> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::Date => write!(
                    writer,
                    r#"
                    impl DateElementDef for {name}Def {{
                        const RANGE: Range<i64> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                        const DEFAULT: Option<i64> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::String => write!(
                    writer,
                    r#"
                    impl StringElementDef for {name}Def {{
                        const DEFAULT: Option<&'static str> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::Utf8 => write!(
                    writer,
                    r#"
                    impl Utf8ElementDef for {name}Def {{
                        const DEFAULT: Option<&'static str> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
                ElementType::Binary => write!(
                    writer,
                    r#"
                    impl BinaryElementDef for {name}Def {{
                        const DEFAULT: Option<&'static [u8]> = {default};
                    }}
                    "#,
                    name = element.name,
                    default = "None"
                ),
            }?;
        }

        Ok(())
    }

    pub fn write_parsers<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let element_names = self
            .elements
            .values()
            .map(|elem| elem.name.clone())
            .chain(core::iter::once("_Document".to_string()))
            .collect::<Vec<_>>();
        let parent_names = self
            .parents
            .iter()
            .map(|(id, parent_ids)| {
                (
                    self.elements.get(id).unwrap().name.clone(),
                    parent_ids
                        .iter()
                        .map(|parent_id| {
                            parent_id.map_or("_Document".to_string(), |pid| {
                                self.elements.get(&pid).unwrap().name.clone()
                            })
                        })
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let child_names = self
            .children
            .iter()
            .map(|(id, child_ids)| {
                (
                    id.map_or("_Document".to_string(), |pid| {
                        self.elements.get(&pid).unwrap().name.clone()
                    }),
                    child_ids
                        .iter()
                        .map(|child_id| self.elements.get(child_id).unwrap().name.clone())
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        writer.write_all(
            r#"
            use crate::base::element_defs::ElementDef;
            #[allow(unused_imports)]
            use crate::base::parser::{
                BoundTo, ElementReader, ElementState, IntoReader, NextStateNavigation, ReaderError,
                SkipStateNavigation, StateDataParser, StateError,
            };
            #[allow(unused_imports)]
            use crate::base::stream::{parse, serialize, stream_diff};
            use crate::core::element_defs;
            #[allow(unused_imports)]
            use crate::{
                impl_from_readers_for_states, impl_from_subreaders_for_readers, impl_from_substates_for_states,
                impl_into_reader, impl_next_state_navigation, impl_skip_state_navigation,
            };

            use enum_dispatch::enum_dispatch;

            use core::convert::{From, TryInto};
            use core::marker::PhantomData;
            use std::io::BufRead;

            // Top-Level Reader/State Enums #########################################################################
            "#.as_bytes()
        )?;

        for element_name in child_names
            .iter()
            .filter_map(|(name, c_names)| (!c_names.is_empty()).then(|| name))
        {
            write!(
                writer,
                r#"
                #[enum_dispatch({name}NextStates)]
                #[enum_dispatch({name}NextReaders<R>)]
                "#,
                name = element_name,
            )?;
        }
        for element_name in parent_names
            .iter()
            .filter_map(|(name, p_names)| (p_names.len() > 1).then(|| name))
        {
            write!(
                writer,
                r#"
                #[enum_dispatch({name}PrevStates)]
                #[enum_dispatch({name}PrevReaders<R>)]
                "#,
                name = element_name,
            )?;
        }

        writer.write_all(
            r#"
            #[enum_dispatch(States)]
            #[enum_dispatch(Readers<R>)]
            trait BlankTrait {}
            "#
            .as_bytes(),
        )?;

        write!(
            writer,
            r#"
            #[enum_dispatch]
            pub enum States {{
                {elements}
            }}
            "#,
            elements = element_names
                .iter()
                .map(|elem_name| format!("{0}({0}State),", elem_name))
                .collect::<String>()
        )?;
        write!(
            writer,
            r#"
            #[enum_dispatch]
            pub enum Readers<R> {{
                {elements}
            }}
            
            "#,
            elements = element_names
                .iter()
                .map(|elem_name| format!("{0}({0}Reader<R>),", elem_name))
                .collect::<String>()
        )?;

        write!(
            writer,
            r#"
            impl_into_reader!(
                States,
                Readers,
                [{elements}]
            );
            
            impl_from_readers_for_states!(
                Readers,
                States,
                [{elements}]
            );
            
            "#,
            elements = itertools::intersperse(element_names.iter().map(String::as_str), ", ")
                .collect::<String>()
        )?;

        write!(
            writer,
            r#"
            // _Document Objects #########################################################################

            #[derive(Debug, Clone, PartialEq)]
            pub struct _DocumentState;
            pub type _DocumentReader<R> = ElementReader<R, _DocumentState>;

            impl<R: BufRead> _DocumentReader<R> {{
                pub fn new(reader: R) -> Self {{
                    Self {{
                        reader,
                        state: _DocumentState,
                    }}
                }}
            }}

            impl<R: BufRead> IntoReader<R> for _DocumentState {{
                type Reader = _DocumentReader<R>;
                fn into_reader(self, reader: R) -> _DocumentReader<R> {{
                    _DocumentReader::new(reader)
                }}
            }}

            impl_next_state_navigation!(
                _DocumentState,
                _DocumentNextStates,
                [{child_pairs}]
            );
            "#,
            child_pairs = itertools::intersperse(
                child_names
                    .get("_Document")
                    .unwrap()
                    .iter()
                    .map(|cname| format!("({cname}, {cname}State)")),
                ", ".to_string()
            )
            .collect::<String>(),
        )?;

        write!(
            writer,
            r#"
            #[derive(Debug, Clone, PartialEq)]
            #[enum_dispatch]
            pub enum _DocumentNextStates {{
                {child_states}
            }}

            #[derive(Debug, PartialEq)]
            #[enum_dispatch]
            pub enum _DocumentNextReaders<R> {{
                {child_readers}
            }}

            impl_from_substates_for_states!(_DocumentNextStates, States, [{children}]);
            impl_from_subreaders_for_readers!(_DocumentNextReaders, Readers, [{children}]);

            impl_into_reader!(_DocumentNextStates, _DocumentNextReaders, [{children}]);
            impl_from_readers_for_states!(_DocumentNextReaders, _DocumentNextStates, [{children}]);
            "#,
            child_states = child_names
                .get("_Document")
                .unwrap()
                .iter()
                .map(|name| format!("{name}({name}State),"))
                .collect::<String>(),
            child_readers = child_names
                .get("_Document")
                .unwrap()
                .iter()
                .map(|name| format!("{name}({name}Reader<R>),"))
                .collect::<String>(),
            children = itertools::intersperse(
                child_names
                    .get("_Document")
                    .unwrap()
                    .iter()
                    .map(String::as_str),
                ", "
            )
            .collect::<String>(),
        )?;

        let make_state = |name: &str| format!("{}State", name);
        let make_reader = |name: &str| format!("{}Reader", name);
        let make_prev_states = |name: &str| format!("{}PrevStates", name);
        let make_prev_readers = |name: &str| format!("{}PrevReaders", name);
        let make_next_states = |name: &str| format!("{}NextStates", name);
        let make_next_readers = |name: &str| format!("{}NextReaders", name);

        for element_name in element_names {
            if element_name == "_Document" {
                continue;
            }

            let elem_parent_names = parent_names.get(&element_name).unwrap();
            let elem_child_names = child_names.get(&element_name).unwrap();

            let (parent_state_name, parent_reader_name) = if elem_parent_names.len() > 1 {
                (
                    make_prev_states(&element_name),
                    make_prev_readers(&element_name),
                )
            } else {
                let parent_name = parent_names
                    .get(&element_name)
                    .unwrap()
                    .iter()
                    .next()
                    .unwrap();
                (make_state(parent_name), make_reader(parent_name))
            };

            let child_state_name = if !elem_child_names.is_empty() {
                make_next_states(&element_name)
            } else {
                parent_state_name.clone()
            };

            write!(
                writer,
                r#"
                // {name} Objects #########################################################################

                pub type {name}State = ElementState<element_defs::{name}Def, {parent_state}>;
                pub type {name}Reader<R> = ElementReader<R, {name}State>;

                impl {name}State {{
                    pub fn new(bytes_left: usize, parent_state: {parent_state}) -> Self {{
                        Self {{
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }}
                    }}
                }}

                impl<R: BufRead> {name}Reader<R> {{
                    pub fn new(reader: R, state: {name}State) -> Self {{
                        Self {{ reader, state }}
                    }}
                }}

                impl_skip_state_navigation!({name}State, {parent_state});
                impl_next_state_navigation!({name}State, {child_state}, [{child_pairs}]);
                "#,
                name = element_name,
                parent_state = parent_state_name.as_str(),
                child_state = child_state_name,
                child_pairs = itertools::intersperse(
                    elem_child_names
                        .iter()
                        .map(|cname| format!("({cname}, {cname}State)")),
                    ", ".to_string()
                )
                .collect::<String>()
            )?;

            if !elem_child_names.is_empty() {
                write!(
                    writer,
                    r#"
                    #[derive(Debug, Clone, PartialEq)]
                    #[enum_dispatch]
                    pub enum {name}NextStates {{
                        {child_states}
                        Parent({parent_state}),
                    }}

                    #[derive(Debug, PartialEq)]
                    #[enum_dispatch]
                    pub enum {name}NextReaders<R> {{
                        {child_readers}
                        Parent({parent_reader}<R>),
                    }}

                    impl_from_substates_for_states!({name}NextStates, States, [{children}]);
                    impl_from_subreaders_for_readers!({name}NextReaders, Readers, [{children}]);

                    impl_into_reader!({name}NextStates, {name}NextReaders, [{children}]);
                    impl_from_readers_for_states!({name}NextReaders, {name}NextStates, [{children}]);
                    "#,
                    name = element_name,
                    parent_state = parent_state_name,
                    parent_reader = parent_reader_name,
                    child_states = elem_child_names
                        .iter()
                        .map(|name| format!("{name}({name}State),"))
                        .collect::<String>(),
                    child_readers = elem_child_names
                        .iter()
                        .map(|name| format!("{name}({name}Reader<R>),"))
                        .collect::<String>(),
                    children = itertools::intersperse(
                        elem_child_names
                            .iter()
                            .map(String::as_str)
                            .chain(core::iter::once("Parent")),
                        ", "
                    )
                    .collect::<String>(),
                )?;
            }

            if elem_parent_names.len() > 1 {
                write!(
                    writer,
                    r#"
                    #[derive(Debug, Clone, PartialEq)]
                    #[enum_dispatch]
                    pub enum {name}PrevStates {{
                        {parent_states}
                    }}
                    #[derive(Debug, PartialEq)]
                    #[enum_dispatch]
                    pub enum {name}PrevReaders<R> {{
                        {parent_readers}
                    }}

                    impl_from_substates_for_states!({name}PrevStates, States, [_Document, Files, File]);
                    impl_from_subreaders_for_readers!({name}PrevReaders, Readers, [_Document, Files, File]);

                    impl_into_reader!({name}PrevStates, {name}PrevReaders, [_Document, Files, File]);
                    impl_from_readers_for_states!({name}PrevReaders, {name}PrevStates, [_Document, Files, File]);

                    "#,
                    name = element_name,
                    parent_states = elem_parent_names
                        .iter()
                        .map(|name| format!("{name}({name}State),"))
                        .collect::<String>(),
                    parent_readers = elem_parent_names
                        .iter()
                        .map(|name| format!("{name}({name}Reader<R>),"))
                        .collect::<String>(),
                )?;
            }
        }

        Ok(())
    }

    pub fn write_package<P: AsRef<Path>>(&self, path: P) -> Result<(), WriteParserPackageError> {
        let template_dir_path = {
            let mut cwd = std::env::var("CARGO_MANIFEST_DIR")
                .map(std::path::PathBuf::from)
                .map_err(WriteParserPackageError::NoManifestPath)?;
            cwd.pop();
            cwd.push("base_template");
            cwd
        };

        std::fs::create_dir_all((path.as_ref()).join("src/base/"))
            .map_err(WriteParserPackageError::IOError)?;
        std::fs::create_dir_all((path.as_ref()).join("src/core/"))
            .map_err(WriteParserPackageError::IOError)?;

        for filename in &[
            "Cargo.toml",
            "src/lib.rs",
            "src/base/element_defs.rs",
            "src/base/mod.rs",
            "src/base/parser.rs",
            "src/base/stream.rs",
            "src/core/mod.rs",
        ] {
            std::fs::copy(
                template_dir_path.join(filename),
                path.as_ref().join(filename),
            )
            .map_err(WriteParserPackageError::IOError)?;
        }

        {
            let mut writer = std::fs::File::create(path.as_ref().join("src/core/element_defs.rs"))
                .map(std::io::BufWriter::new)
                .map_err(WriteParserPackageError::IOError)?;
            self.write_element_defs(&mut writer)
                .map_err(WriteParserPackageError::IOError)?;
        }

        {
            let mut writer = std::fs::File::create(path.as_ref().join("src/core/parser.rs"))
                .map(std::io::BufWriter::new)
                .map_err(WriteParserPackageError::IOError)?;
            self.write_parsers(&mut writer)
                .map_err(WriteParserPackageError::IOError)?;
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WriteParserPackageError {
    #[error("no path to cargo manifest: {0}")]
    NoManifestPath(std::env::VarError),
    #[error("IO error: {0}")]
    IOError(std::io::Error),
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
    fn builder_generate(schema: EbmlSchema) {
        let result = Builder { schema }.generate();
        let result = result.unwrap();

        assert_eq!(
            result.elements.keys().collect::<Vec<_>>(),
            vec![&0xEC, &0x4282, &0x4286, &0x1A45DFA3]
        );
        assert_eq!(
            result.parents.into_iter().collect::<Vec<_>>(),
            vec![
                (
                    0xEC,
                    vec![None, Some(0x1A45DFA3)]
                        .into_iter()
                        .collect::<HashSet<_>>()
                ),
                (
                    0x4282,
                    vec![Some(0x1A45DFA3)].into_iter().collect::<HashSet<_>>()
                ),
                (
                    0x4286,
                    vec![Some(0x1A45DFA3)].into_iter().collect::<HashSet<_>>()
                ),
                (0x1A45DFA3, vec![None].into_iter().collect::<HashSet<_>>()),
            ]
        );
        assert_eq!(
            result.children.into_iter().collect::<Vec<_>>(),
            vec![
                (
                    None,
                    vec![0x1A45DFA3, 0xEC].into_iter().collect::<HashSet<_>>()
                ),
                (Some(0xEC), HashSet::new()),
                (Some(0x4282), HashSet::new()),
                (Some(0x4286), HashSet::new()),
                (
                    Some(0x1A45DFA3),
                    vec![0x4286, 0x4282, 0xEC]
                        .into_iter()
                        .collect::<HashSet<_>>()
                ),
            ]
        );
    }
}
