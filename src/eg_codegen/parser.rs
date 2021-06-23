use crate::eg_codegen::element_defs;
use crate::element_defs::ParentOf;
use crate::parser::{ElementState, StateOf};

// State Objects
type FilesState = NestedElementStates!(element_defs::FilesDef);

impl<P, S> ElementState<element_defs::FilesDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::FilesDef>,
{
}

type FileState =
    NestedElementStates!(element_defs::FileDef, element_defs::FilesDef);

impl<P, S> ElementState<element_defs::FileDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::FileDef>,
{
}

type FileNameState = NestedElementStates!(
    element_defs::FileNameDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::FileNameDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::FileNameDef>,
{
}

type MimeTypeState = NestedElementStates!(
    element_defs::MimeTypeDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::MimeTypeDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::MimeTypeDef>,
{
}

type ModificationTimestampState = NestedElementStates!(
    element_defs::ModificationTimestampDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::ModificationTimestampDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::ModificationTimestampDef>,
{
}

type DataState = NestedElementStates!(
    element_defs::DataDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::DataDef, S>
where
    S: StateOf<Element=P>,
    P: ParentOf<element_defs::DataDef>,
{
}

// Reader Objects
/*
enum FilesReaderNext<P> {
    Parent(P),
    File(FilesReader<FilesState>),
}
struct Reader<S> {
    state: S,
}
*/
