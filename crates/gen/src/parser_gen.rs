// interface loosely based on that of bindgen: https://crates.io/crates/bindgen

use crate::element_model::{EbmlElementModel, EbmlElementModelError, GlobalPlaceholder, PathAtoms};
use crate::serde_schema::{from_reader, EbmlSchema, ElementType};

use std::collections::{HashMap, HashSet};
use std::path::Path;

/**
The `Builder` object is simply a wrapper to abstract away the schema parsing. It provides
methods to convert a schema file into a generator object, without having to directly use
any other parsing functions or manage any other objects.
**/
pub struct Builder {
    schema: EbmlSchema,
}

impl Builder {
    pub fn new<R: std::io::Read>(schema: R) -> Result<Self, serde_xml_rs::Error> {
        Ok(Self {
            schema: from_reader(schema)?,
        })
    }

    pub fn make_generator(self) -> Result<CodeGenerator, EbmlElementModelError> {
        Ok(CodeGenerator {
            elem_model: EbmlElementModel::new(self.schema)?,
        })
    }
}

/**
The `CodeGenerator` object has only one job: write valid Rust code as described in the schema.
Everything else (reading the schema, validating the element definitions & hierarchy, etc.)
should be done elsewhere.

**/

pub struct CodeGenerator {
    elem_model: EbmlElementModel,
}

impl CodeGenerator {
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

        for element in self.elem_model.elements.values() {
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
            .elem_model
            .elements
            .values()
            .map(|elem| elem.name.clone())
            .chain(core::iter::once("_Document".to_string()))
            .collect::<Vec<_>>();
        let parent_names = self
            .elem_model
            .parents
            .iter()
            .map(|(id, parent_ids)| {
                (
                    self.elem_model.elements.get(id).unwrap().name.clone(),
                    parent_ids
                        .iter()
                        .map(|parent_id| {
                            parent_id.map_or("_Document".to_string(), |pid| {
                                self.elem_model.elements.get(&pid).unwrap().name.clone()
                            })
                        })
                        .collect::<HashSet<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let child_names = self
            .elem_model
            .children
            .iter()
            .map(|(id, child_ids)| {
                (
                    id.map_or("_Document".to_string(), |pid| {
                        self.elem_model.elements.get(&pid).unwrap().name.clone()
                    }),
                    child_ids
                        .iter()
                        .map(|child_id| {
                            self.elem_model.elements.get(child_id).unwrap().name.clone()
                        })
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
