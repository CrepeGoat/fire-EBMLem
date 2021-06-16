use crate::eg_codegen::element_defs;
use crate::element_defs::ParentOf;
use crate::parser;

// State Objects
type FilesState = parser::NestedElementStates![gen_element_defs::FilesDef];

impl<P> parser::ElementState<gen_element_defs::FilesDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::FilesDef>>
{
}

//impl<P, S> FilesState<P, S> where P: element_defs::ParentOf<gen_element_defs::FilesDef> {}

type FileState =
    parser::NestedElementStates![gen_element_defs::FileDef, gen_element_defs::FilesDef,];

impl<P> parser::ElementState<gen_element_defs::FileDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::FileDef>>
{
}

type FileNameState = parser::NestedElementStates![
    gen_element_defs::FileNameDef,
    gen_element_defs::FileDef,
    gen_element_defs::FilesDef
];

impl<P> parser::ElementState<gen_element_defs::FileNameDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::FileNameDef>>
{
}

type MimeTypeState = parser::NestedElementStates![
    gen_element_defs::MimeTypeDef
    gen_element_defs::FileDef,
    gen_element_defs::FilesDef,
];

impl<P> parser::ElementState<gen_element_defs::MimeTypeDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::MimeTypeDef>>
{
}

type ModificationTimestampState = parser::NestedElementStates![
    gen_element_defs::ModificationTimestampDef
    gen_element_defs::FileDef,
    gen_element_defs::FilesDef,
];

impl<P> parser::ElementState<gen_element_defs::ModificationTimestampDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::ModificationTimestampDef>>
{
}

type DataState = parser::NestedElementStates![
    gen_element_defs::DataDef
    gen_element_defs::FileDef,
    gen_element_defs::FilesDef,
];

impl<P> parser::ElementState<gen_element_defs::DataDef, P> where
    P: parser::StateOf<ParentOf<gen_element_defs::DataDef>>
{
}

// Reader Objects
enum FilesReaderNext<P> {
    Parent(P),
    File(FilesReader<FilesState>),
}
struct Reader<S> {
    state: S,
}
