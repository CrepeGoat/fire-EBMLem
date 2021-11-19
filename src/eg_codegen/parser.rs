use core::convert::{From, TryInto};
use core::marker::PhantomData;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{ElementReader, ElementState, StateOf};
use crate::stream::{parse, serialize, stream_diff};

// _Document Objects #########################################################################

type _DocumentState = ElementState<(), ()>;
type _DocumentReader<R> = ElementReader<R, _DocumentState>;

#[derive(Debug, Clone, PartialEq)]
enum _DocumentNextStates {
    Files(FilesState),
    None,
}

#[derive(Debug, PartialEq)]
enum _DocumentNextReaders<R> {
    Files(FilesReader<R>),
    None(R),
}

impl _DocumentNextStates {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> _DocumentNextReaders<R> {
        match self {
            _DocumentNextStates::Files(state) => {
                _DocumentNextReaders::Files(FilesReader::<R> { reader, state })
            }
            _DocumentNextStates::None => _DocumentNextReaders::None(reader),
        }
    }
}

impl<R> From<_DocumentNextReaders<R>> for _DocumentNextStates {
    fn from(enumed_reader: _DocumentNextReaders<R>) -> _DocumentNextStates {
        match enumed_reader {
            _DocumentNextReaders::Files(reader) => _DocumentNextStates::Files(reader.state),
            _DocumentNextReaders::None(_) => _DocumentNextStates::None,
        }
    }
}

impl _DocumentState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> _DocumentReader<R> {
        _DocumentReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], (), ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

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
}

impl<R: std::io::BufRead> _DocumentReader<R> {
    fn new(reader: R, state: _DocumentState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<R> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(self.reader)
    }

    fn next(self) -> std::io::Result<_DocumentNextReaders<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }
}

// Files Objects #########################################################################

type FilesReader<R> = ElementReader<R, FilesState>;
type FilesState = ElementState<element_defs::FilesDef, _DocumentState>;

#[derive(Debug, Clone, PartialEq)]
enum FilesNextStates {
    File(FileState),
    Parent(_DocumentState),
}

#[derive(Debug, PartialEq)]
enum FilesNextReaders<R> {
    File(FileReader<R>),
    Parent(_DocumentReader<R>),
}

impl FilesNextStates {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> FilesNextReaders<R> {
        match self {
            FilesNextStates::File(state) => FilesNextReaders::File(FileReader { reader, state }),
            FilesNextStates::Parent(state) => {
                FilesNextReaders::Parent(_DocumentReader { reader, state })
            }
        }
    }
}

impl<R> From<FilesNextReaders<R>> for FilesNextStates {
    fn from(enumed_reader: FilesNextReaders<R>) -> FilesNextStates {
        match enumed_reader {
            FilesNextReaders::File(reader) => FilesNextStates::File(reader.state),
            FilesNextReaders::Parent(reader) => FilesNextStates::Parent(reader.state),
        }
    }
}

impl FilesState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> FilesReader<R> {
        FilesReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], _DocumentState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FilesNextStates, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FilesNextStates::Parent(self.parent_state))),
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
                            FilesNextStates::File(ElementState {
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
}

impl<R: std::io::BufRead> FilesReader<R> {
    fn new(reader: R, state: FilesState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<_DocumentReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FilesNextReaders<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }
}

// File Objects #########################################################################

type FileState = ElementState<element_defs::FileDef, FilesState>;
type FileReader<R> = ElementReader<R, FileState>;

#[derive(Debug, Clone, PartialEq)]
enum FileNextStates {
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
    Parent(FilesState),
}

#[derive(Debug, PartialEq)]
enum FileNextReaders<R> {
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
    Parent(FilesReader<R>),
}

impl<R> From<FileNextReaders<R>> for FileNextStates {
    fn from(enumed_reader: FileNextReaders<R>) -> Self {
        match enumed_reader {
            FileNextReaders::FileName(reader) => Self::FileName(reader.state),
            FileNextReaders::MimeType(reader) => Self::MimeType(reader.state),
            FileNextReaders::ModificationTimestamp(reader) => {
                Self::ModificationTimestamp(reader.state)
            }
            FileNextReaders::Data(reader) => Self::Data(reader.state),
            FileNextReaders::Parent(reader) => Self::Parent(reader.state),
        }
    }
}

impl FileNextStates {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> FileNextReaders<R> {
        match self {
            Self::FileName(state) => {
                FileNextReaders::<R>::FileName(FileNameReader::new(reader, state))
            }
            Self::MimeType(state) => {
                FileNextReaders::<R>::MimeType(MimeTypeReader::new(reader, state))
            }
            Self::ModificationTimestamp(state) => FileNextReaders::<R>::ModificationTimestamp(
                ModificationTimestampReader::new(reader, state),
            ),
            Self::Data(state) => FileNextReaders::<R>::Data(DataReader::new(reader, state)),
            Self::Parent(state) => FileNextReaders::<R>::Parent(FilesReader::new(reader, state)),
        }
    }
}

impl FileState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> FileReader<R> {
        FileReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FilesState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileNextStates, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FileNextStates::Parent(self.parent_state))),
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
                            FileNextStates::FileName(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::MimeType(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::ModificationTimestampDef as ElementDef>::ID => {
                            FileNextStates::ModificationTimestamp(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::DataDef as ElementDef>::ID => {
                            FileNextStates::Data(ElementState {
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
}

impl<R: std::io::BufRead> FileReader<R> {
    fn new(reader: R, state: FileState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<FilesReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FileNextReaders<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }
}

// FileName Objects #########################################################################

type FileNameState = ElementState<element_defs::FileNameDef, FileState>;
type FileNameReader<R> = ElementReader<R, FileNameState>;

impl FileNameState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> FileNameReader<R> {
        FileNameReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        self.skip(stream)
    }
}

impl<R: std::io::BufRead> FileNameReader<R> {
    fn new(reader: R, state: FileNameState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<FileReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FileReader<R>> {
        self.skip()
    }
}

// MimeType Objects #########################################################################

type MimeTypeState = ElementState<element_defs::MimeTypeDef, FileState>;
type MimeTypeReader<R> = ElementReader<R, MimeTypeState>;

impl MimeTypeState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> MimeTypeReader<R> {
        MimeTypeReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        self.skip(stream)
    }
}

impl<R: std::io::BufRead> MimeTypeReader<R> {
    fn new(reader: R, state: MimeTypeState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<FileReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FileReader<R>> {
        self.skip()
    }
}

// ModificationTimestamp Objects #########################################################################

type ModificationTimestampState = ElementState<element_defs::ModificationTimestampDef, FileState>;
type ModificationTimestampReader<R> = ElementReader<R, ModificationTimestampState>;

impl ModificationTimestampState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> ModificationTimestampReader<R> {
        ModificationTimestampReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        self.skip(stream)
    }
}

impl<R: std::io::BufRead> ModificationTimestampReader<R> {
    fn new(reader: R, state: ModificationTimestampState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<FileReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FileReader<R>> {
        self.skip()
    }
}

// Data Objects #########################################################################

type DataState = ElementState<element_defs::DataDef, FileState>;
type DataReader<R> = ElementReader<R, DataState>;

impl DataState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> DataReader<R> {
        DataReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], FileState, ()> {
        self.skip(stream)
    }
}

impl<R: std::io::BufRead> DataReader<R> {
    fn new(reader: R, state: DataState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<FileReader<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<FileReader<R>> {
        self.skip()
    }
}

// Void Objects #########################################################################

#[derive(Debug, Clone, PartialEq)]
enum VoidPrevStates {
    Files(FilesState),
    File(FileState),
}
#[derive(Debug, PartialEq)]
enum VoidPrevReaders<R> {
    Files(FilesReader<R>),
    File(FileReader<R>),
}

impl VoidPrevStates {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> VoidPrevReaders<R> {
        match self {
            VoidPrevStates::Files(state) => VoidPrevReaders::Files(FilesReader::new(reader, state)),
            VoidPrevStates::File(state) => VoidPrevReaders::File(FileReader::new(reader, state)),
        }
    }
}

impl<R> From<VoidPrevReaders<R>> for VoidPrevStates {
    fn from(enumed_reader: VoidPrevReaders<R>) -> VoidPrevStates {
        match enumed_reader {
            VoidPrevReaders::Files(reader) => VoidPrevStates::Files(reader.state),
            VoidPrevReaders::File(reader) => VoidPrevStates::File(reader.state),
        }
    }
}

type VoidState = ElementState<element_defs::DataDef, VoidPrevStates>;
type VoidReader<R> = ElementReader<R, VoidState>;

impl VoidState {
    fn to_reader<R: std::io::BufRead>(self, reader: R) -> VoidReader<R> {
        VoidReader::new(reader, self)
    }

    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], VoidPrevStates, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], VoidPrevStates, ()> {
        self.skip(stream)
    }
}

impl<R: std::io::BufRead> VoidReader<R> {
    fn new(reader: R, state: VoidState) -> Self {
        Self { reader, state }
    }

    fn skip(self) -> std::io::Result<VoidPrevReaders<R>> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        self.reader.consume(next_stream.len() - stream.len());

        Ok(next_state.to_reader(self.reader))
    }

    fn next(self) -> std::io::Result<VoidPrevReaders<R>> {
        self.skip()
    }
}

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
            expt_result: (&'static [u8], FilesNextStates),
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
            expt_result: (&'static [u8], FileNextStates),
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
