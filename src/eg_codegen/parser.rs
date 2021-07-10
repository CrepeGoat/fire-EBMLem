use core::convert::TryInto;
use core::marker::PhantomData;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{ElementState, StateOf};
use crate::stream::{parse, serialize, stream_diff};

// State Objects
type FilesState = NestedElementStates!(element_defs::FilesDef);

enum FilesNextStates<S> {
    File(ElementState<element_defs::FileDef, ElementState<element_defs::FilesDef, S>>),
    Parent(S),
}

impl<P, S> ElementState<element_defs::FilesDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::FilesDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FilesNextStates<S>, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FilesNextStates::<S>::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream)?;
                let (stream, len) = parse::element_len(stream)?;
                let len: usize = len
                    .expect("todo: handle optionally unsized elements")
                    .try_into()
                    .expect("overflow in storing element bytelength");

                self.bytes_left -= len + stream_diff(orig_stream, stream);

                Ok((
                    stream,
                    match id {
                        <element_defs::FileDef as ElementDef>::ID => {
                            FilesNextStates::<S>::File(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        _ => return Err(nom::Err::Failure(())),
                    },
                ))
            }
        }
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type FileState = NestedElementStates!(element_defs::FileDef, element_defs::FilesDef);

enum FileNextStates<S> {
    FileName(ElementState<element_defs::FileNameDef, ElementState<element_defs::FileDef, S>>),
    MimeType(ElementState<element_defs::MimeTypeDef, ElementState<element_defs::FileDef, S>>),
    ModificationTimestamp(
        ElementState<
            element_defs::ModificationTimestampDef,
            ElementState<element_defs::FileDef, S>,
        >,
    ),
    Data(ElementState<element_defs::DataDef, ElementState<element_defs::FileDef, S>>),
    Parent(S),
}

impl<P, S> ElementState<element_defs::FileDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::FileDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileNextStates<S>, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FileNextStates::<S>::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream)?;
                let (stream, len) = parse::element_len(stream)?;
                let len: usize = len
                    .expect("todo: handle optionally unsized elements")
                    .try_into()
                    .expect("overflow in storing element bytelength");

                self.bytes_left -= len + stream_diff(orig_stream, stream);

                Ok((
                    stream,
                    match id {
                        <element_defs::FileNameDef as ElementDef>::ID => {
                            FileNextStates::<S>::FileName(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::<S>::MimeType(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::<S>::ModificationTimestamp(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::<S>::Data(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        _ => return Err(nom::Err::Failure(())),
                    },
                ))
            }
        }
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type FileNameState = NestedElementStates!(
    element_defs::FileNameDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::FileNameDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::FileNameDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        self.skip(stream)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type MimeTypeState = NestedElementStates!(
    element_defs::MimeTypeDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::MimeTypeDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::MimeTypeDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        self.skip(stream)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type ModificationTimestampState = NestedElementStates!(
    element_defs::ModificationTimestampDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::ModificationTimestampDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::ModificationTimestampDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        self.skip(stream)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type DataState = NestedElementStates!(
    element_defs::DataDef,
    element_defs::FileDef,
    element_defs::FilesDef
);

impl<P, S> ElementState<element_defs::DataDef, S>
where
    S: StateOf<Element = P>,
    P: ParentOf<element_defs::DataDef>,
{
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        self.skip(stream)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
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
