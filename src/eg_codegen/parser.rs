use core::convert::TryInto;
use core::marker::PhantomData;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{ElementState, StateOf};
use crate::stream::{parse, serialize, stream_diff};

// State Objects
type _DocumentState = NestedElementStates!();

#[derive(Debug, Clone, PartialEq)]
enum _DocumentNextStates {
    Files(ElementState<element_defs::FilesDef, ElementState<(), ()>>),
    None,
}

impl ElementState<(), ()> {
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], _DocumentNextStates, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, _DocumentNextStates::None)),
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
                        <element_defs::FilesDef as ElementDef>::ID => {
                            _DocumentNextStates::Files(ElementState {
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

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], (), ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }
}

type FilesState = NestedElementStates!(element_defs::FilesDef);

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
                        <element_defs::ModificationTimestampDef as ElementDef>::ID => {
                            FileNextStates::<S>::ModificationTimestamp(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::DataDef as ElementDef>::ID => {
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
    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
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
    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
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
    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
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
    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], S, ()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    mod document {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                _DocumentState{bytes_left: 7, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], _DocumentNextStates::Files(FilesState{bytes_left: 2, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF][..], _DocumentNextStates::None)
            ),
        )]
        fn next(
            element: _DocumentState,
            source: &'static [u8],
            expt_result: (&'static [u8], _DocumentNextStates),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                _DocumentState{bytes_left: 7, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], ())
            ),
        )]
        fn skip(element: _DocumentState, source: &'static [u8], expt_result: (&'static [u8], ())) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod files {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FilesState{bytes_left: 5, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x46, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::File(FileState{bytes_left: 2, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::Parent(_DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}))
            ),
        )]
        fn next(
            element: FilesState,
            source: &'static [u8],
            expt_result: (&'static [u8], FilesNextStates<_DocumentState>),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FilesState{bytes_left: 5, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FilesState,
            source: &'static [u8],
            expt_result: (&'static [u8], _DocumentState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod file {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::FileName(FileNameState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x4D, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::MimeType(MimeTypeState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x54, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::ModificationTimestamp(ModificationTimestampState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x64, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Data(DataState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Parent(FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}))
            ),
        )]
        fn next(
            element: FileState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileNextStates<FilesState>),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 1, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FilesState{bytes_left: 1, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FileState,
            source: &'static [u8],
            expt_result: (&'static [u8], FilesState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod filename {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: FileNameState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FileNameState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod mimetype {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: MimeTypeState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: MimeTypeState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod modificationtimestamp {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: ModificationTimestampState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: ModificationTimestampState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod data {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: DataState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: DataState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }
}
