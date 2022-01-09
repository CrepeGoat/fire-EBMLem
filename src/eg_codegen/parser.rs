use core::convert::{From, TryInto};
use core::marker::PhantomData;
use std::io::BufRead;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{ElementReader, ElementState, ReaderError, StateError, StateOf};
use crate::stream::{parse, serialize, stream_diff};

// Top-Level Reader/State Enums #########################################################################

pub enum States {
    _Document(_DocumentState),
    Files(FilesState),
    File(FileState),
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
    None,
}

pub enum Readers<R> {
    _Document(_DocumentReader<R>),
    Files(FilesReader<R>),
    File(FileReader<R>),
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
    None(R),
}

impl States {
    fn into_reader<R: BufRead>(self, reader: R) -> Readers<R> {
        match self {
            Self::_Document(state) => Readers::_Document(state.into_reader(reader)),
            Self::Files(state) => Readers::Files(state.into_reader(reader)),
            Self::File(state) => Readers::File(state.into_reader(reader)),
            Self::FileName(state) => Readers::FileName(state.into_reader(reader)),
            Self::MimeType(state) => Readers::MimeType(state.into_reader(reader)),
            Self::ModificationTimestamp(state) => {
                Readers::ModificationTimestamp(state.into_reader(reader))
            }
            Self::Data(state) => Readers::Data(state.into_reader(reader)),
            Self::None => Readers::None(reader),
        }
    }
}

impl<R> From<Readers<R>> for States {
    fn from(enumed_reader: Readers<R>) -> Self {
        match enumed_reader {
            Readers::_Document(reader) => Self::_Document(reader.state),
            Readers::Files(reader) => Self::Files(reader.state),
            Readers::File(reader) => Self::File(reader.state),
            Readers::FileName(reader) => Self::FileName(reader.state),
            Readers::MimeType(reader) => Self::MimeType(reader.state),
            Readers::ModificationTimestamp(reader) => Self::ModificationTimestamp(reader.state),
            Readers::Data(reader) => Self::Data(reader.state),
            Readers::None(_) => Self::None,
        }
    }
}

// _Document Objects #########################################################################

pub type _DocumentState = ElementState<(), ()>;
pub type _DocumentReader<R> = ElementReader<R, _DocumentState>;

impl From<_DocumentState> for States {
    fn from(state: _DocumentState) -> Self {
        Self::_Document(state)
    }
}

impl<R: BufRead> From<_DocumentReader<R>> for Readers<R> {
    fn from(reader: _DocumentReader<R>) -> Self {
        Self::_Document(reader)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum _DocumentNextStates {
    Files(FilesState),
    None,
}

#[derive(Debug, PartialEq)]
pub enum _DocumentNextReaders<R> {
    Files(FilesReader<R>),
    None(R),
}

impl From<_DocumentNextStates> for States {
    fn from(enumed_states: _DocumentNextStates) -> Self {
        match enumed_states {
            _DocumentNextStates::Files(state) => state.into(),
            _DocumentNextStates::None => Self::None,
        }
    }
}

impl<R: BufRead> From<_DocumentNextReaders<R>> for Readers<R> {
    fn from(enumed_readers: _DocumentNextReaders<R>) -> Self {
        match enumed_readers {
            _DocumentNextReaders::Files(reader) => reader.into(),
            _DocumentNextReaders::None(read) => Self::None(read),
        }
    }
}

impl _DocumentNextStates {
    fn into_reader<R: BufRead>(self, reader: R) -> _DocumentNextReaders<R> {
        match self {
            Self::Files(state) => _DocumentNextReaders::Files(state.into_reader(reader)),
            Self::None => _DocumentNextReaders::None(reader),
        }
    }
}

impl<R> From<_DocumentNextReaders<R>> for _DocumentNextStates {
    fn from(enumed_reader: _DocumentNextReaders<R>) -> Self {
        match enumed_reader {
            _DocumentNextReaders::Files(reader) => Self::Files(reader.state),
            _DocumentNextReaders::None(_) => Self::None,
        }
    }
}

impl _DocumentState {
    fn into_reader<R: BufRead>(self, reader: R) -> _DocumentReader<R> {
        _DocumentReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], (), StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], _DocumentNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, _DocumentNextStates::None)),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
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
                        id => return Err(nom::Err::Failure(StateError::InvalidChildId(None, id))),
                    },
                ))
            }
        }
    }
}

impl<R: BufRead> _DocumentReader<R> {
    pub fn new(reader: R, state: _DocumentState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<R, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, _next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(self.reader)
    }

    pub fn next(mut self) -> Result<_DocumentNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Files Objects #########################################################################

pub type FilesReader<R> = ElementReader<R, FilesState>;
pub type FilesState = ElementState<element_defs::FilesDef, _DocumentState>;

impl From<FilesState> for States {
    fn from(state: FilesState) -> Self {
        Self::Files(state)
    }
}

impl<R: BufRead> From<FilesReader<R>> for Readers<R> {
    fn from(reader: FilesReader<R>) -> Self {
        Self::Files(reader)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FilesNextStates {
    File(FileState),
    Parent(_DocumentState),
}

#[derive(Debug, PartialEq)]
pub enum FilesNextReaders<R> {
    File(FileReader<R>),
    Parent(_DocumentReader<R>),
}

impl From<FilesNextStates> for States {
    fn from(enumed_states: FilesNextStates) -> Self {
        match enumed_states {
            FilesNextStates::File(state) => state.into(),
            FilesNextStates::Parent(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<FilesNextReaders<R>> for Readers<R> {
    fn from(enumed_readers: FilesNextReaders<R>) -> Self {
        match enumed_readers {
            FilesNextReaders::File(reader) => reader.into(),
            FilesNextReaders::Parent(reader) => reader.into(),
        }
    }
}

impl FilesNextStates {
    fn into_reader<R: BufRead>(self, reader: R) -> FilesNextReaders<R> {
        match self {
            Self::File(state) => FilesNextReaders::File(state.into_reader(reader)),
            Self::Parent(state) => FilesNextReaders::Parent(state.into_reader(reader)),
        }
    }
}

impl<R> From<FilesNextReaders<R>> for FilesNextStates {
    fn from(enumed_reader: FilesNextReaders<R>) -> Self {
        match enumed_reader {
            FilesNextReaders::File(reader) => Self::File(reader.state),
            FilesNextReaders::Parent(reader) => Self::Parent(reader.state),
        }
    }
}

impl FilesState {
    fn into_reader<R: BufRead>(self, reader: R) -> FilesReader<R> {
        FilesReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], _DocumentState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], FilesNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FilesNextStates::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
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
                        id => {
                            return Err(nom::Err::Failure(StateError::InvalidChildId(
                                Some(<<Self as StateOf>::Element as ElementDef>::ID),
                                id,
                            )))
                        }
                    },
                ))
            }
        }
    }
}

impl<R: BufRead> FilesReader<R> {
    pub fn new(reader: R, state: FilesState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<_DocumentReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FilesNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// File Objects #########################################################################

pub type FileState = ElementState<element_defs::FileDef, FilesState>;
pub type FileReader<R> = ElementReader<R, FileState>;

impl From<FileState> for States {
    fn from(state: FileState) -> Self {
        Self::File(state)
    }
}

impl<R: BufRead> From<FileReader<R>> for Readers<R> {
    fn from(reader: FileReader<R>) -> Self {
        Self::File(reader)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FileNextStates {
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
    Parent(FilesState),
}

#[derive(Debug, PartialEq)]
pub enum FileNextReaders<R> {
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
    Parent(FilesReader<R>),
}

impl From<FileNextStates> for States {
    fn from(enumed_states: FileNextStates) -> Self {
        match enumed_states {
            FileNextStates::FileName(state) => state.into(),
            FileNextStates::MimeType(state) => state.into(),
            FileNextStates::ModificationTimestamp(state) => state.into(),
            FileNextStates::Data(state) => state.into(),
            FileNextStates::Parent(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<FileNextReaders<R>> for Readers<R> {
    fn from(enumed_readers: FileNextReaders<R>) -> Self {
        match enumed_readers {
            FileNextReaders::FileName(reader) => reader.into(),
            FileNextReaders::MimeType(reader) => reader.into(),
            FileNextReaders::ModificationTimestamp(reader) => reader.into(),
            FileNextReaders::Data(reader) => reader.into(),
            FileNextReaders::Parent(reader) => reader.into(),
        }
    }
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
    fn into_reader<R: BufRead>(self, reader: R) -> FileNextReaders<R> {
        match self {
            Self::FileName(state) => FileNextReaders::<R>::FileName(state.into_reader(reader)),
            Self::MimeType(state) => FileNextReaders::<R>::MimeType(state.into_reader(reader)),
            Self::ModificationTimestamp(state) => {
                FileNextReaders::<R>::ModificationTimestamp(state.into_reader(reader))
            }
            Self::Data(state) => FileNextReaders::<R>::Data(state.into_reader(reader)),
            Self::Parent(state) => FileNextReaders::<R>::Parent(state.into_reader(reader)),
        }
    }
}

impl FileState {
    fn into_reader<R: BufRead>(self, reader: R) -> FileReader<R> {
        FileReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FilesState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], FileNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FileNextStates::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
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
                        id => {
                            return Err(nom::Err::Failure(StateError::InvalidChildId(
                                Some(<<Self as StateOf>::Element as ElementDef>::ID),
                                id,
                            )))
                        }
                    },
                ))
            }
        }
    }
}

impl<R: BufRead> FileReader<R> {
    pub fn new(reader: R, state: FileState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<FilesReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FileNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// FileName Objects #########################################################################

pub type FileNameState = ElementState<element_defs::FileNameDef, FileState>;
pub type FileNameReader<R> = ElementReader<R, FileNameState>;

impl From<FileNameState> for States {
    fn from(state: FileNameState) -> Self {
        Self::FileName(state)
    }
}

impl<R: BufRead> From<FileNameReader<R>> for Readers<R> {
    fn from(reader: FileNameReader<R>) -> Self {
        Self::FileName(reader)
    }
}

impl FileNameState {
    fn into_reader<R: BufRead>(self, reader: R) -> FileNameReader<R> {
        FileNameReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream)
    }
}

impl<R: BufRead> FileNameReader<R> {
    pub fn new(reader: R, state: FileNameState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// MimeType Objects #########################################################################

pub type MimeTypeState = ElementState<element_defs::MimeTypeDef, FileState>;
pub type MimeTypeReader<R> = ElementReader<R, MimeTypeState>;

impl From<MimeTypeState> for States {
    fn from(state: MimeTypeState) -> Self {
        Self::MimeType(state)
    }
}

impl<R: BufRead> From<MimeTypeReader<R>> for Readers<R> {
    fn from(reader: MimeTypeReader<R>) -> Self {
        Self::MimeType(reader)
    }
}

impl MimeTypeState {
    fn into_reader<R: BufRead>(self, reader: R) -> MimeTypeReader<R> {
        MimeTypeReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream)
    }
}

impl<R: BufRead> MimeTypeReader<R> {
    pub fn new(reader: R, state: MimeTypeState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// ModificationTimestamp Objects #########################################################################

pub type ModificationTimestampState =
    ElementState<element_defs::ModificationTimestampDef, FileState>;
pub type ModificationTimestampReader<R> = ElementReader<R, ModificationTimestampState>;

impl From<ModificationTimestampState> for States {
    fn from(state: ModificationTimestampState) -> Self {
        Self::ModificationTimestamp(state)
    }
}

impl<R: BufRead> From<ModificationTimestampReader<R>> for Readers<R> {
    fn from(reader: ModificationTimestampReader<R>) -> Self {
        Self::ModificationTimestamp(reader)
    }
}

impl ModificationTimestampState {
    fn into_reader<R: BufRead>(self, reader: R) -> ModificationTimestampReader<R> {
        ModificationTimestampReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream)
    }
}

impl<R: BufRead> ModificationTimestampReader<R> {
    pub fn new(reader: R, state: ModificationTimestampState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Data Objects #########################################################################

pub type DataState = ElementState<element_defs::DataDef, FileState>;
pub type DataReader<R> = ElementReader<R, DataState>;

impl From<DataState> for States {
    fn from(state: DataState) -> Self {
        Self::Data(state)
    }
}

impl<R: BufRead> From<DataReader<R>> for Readers<R> {
    fn from(reader: DataReader<R>) -> Self {
        Self::Data(reader)
    }
}

impl DataState {
    fn into_reader<R: BufRead>(self, reader: R) -> DataReader<R> {
        DataReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream)
    }
}

impl<R: BufRead> DataReader<R> {
    pub fn new(reader: R, state: DataState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Void Objects #########################################################################

#[derive(Debug, Clone, PartialEq)]
pub enum VoidPrevStates {
    Files(FilesState),
    File(FileState),
}
#[derive(Debug, PartialEq)]
pub enum VoidPrevReaders<R> {
    Files(FilesReader<R>),
    File(FileReader<R>),
}

impl From<VoidPrevStates> for States {
    fn from(enumed_states: VoidPrevStates) -> Self {
        match enumed_states {
            VoidPrevStates::Files(state) => state.into(),
            VoidPrevStates::File(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<VoidPrevReaders<R>> for Readers<R> {
    fn from(enumed_readers: VoidPrevReaders<R>) -> Self {
        match enumed_readers {
            VoidPrevReaders::Files(reader) => reader.into(),
            VoidPrevReaders::File(reader) => reader.into(),
        }
    }
}

impl VoidPrevStates {
    fn into_reader<R: BufRead>(self, reader: R) -> VoidPrevReaders<R> {
        match self {
            Self::Files(state) => VoidPrevReaders::Files(state.into_reader(reader)),
            Self::File(state) => VoidPrevReaders::File(state.into_reader(reader)),
        }
    }
}

impl<R> From<VoidPrevReaders<R>> for VoidPrevStates {
    fn from(enumed_reader: VoidPrevReaders<R>) -> Self {
        match enumed_reader {
            VoidPrevReaders::Files(reader) => Self::Files(reader.state),
            VoidPrevReaders::File(reader) => Self::File(reader.state),
        }
    }
}

pub type VoidState = ElementState<element_defs::DataDef, VoidPrevStates>;
pub type VoidReader<R> = ElementReader<R, VoidState>;

impl VoidState {
    fn into_reader<R: BufRead>(self, reader: R) -> VoidReader<R> {
        VoidReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], VoidPrevStates, StateError> {
        let (stream, _) = nom::bytes::streaming::take::<_, _, ()>(self.bytes_left)(stream)
            .map_err(nom::Err::convert)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], VoidPrevStates, StateError> {
        self.skip(stream)
    }
}

impl<R: BufRead> VoidReader<R> {
    pub fn new(reader: R, state: VoidState) -> Self {
        Self { reader, state }
    }

    pub fn skip(mut self) -> Result<VoidPrevReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    pub fn next(mut self) -> Result<VoidPrevReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
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
        fn state_next(
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
        fn state_skip(
            element: _DocumentState,
            source: &'static [u8],
            expt_result: (&'static [u8], ()),
        ) {
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
        fn state_next(
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
        fn state_skip(
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
        fn state_next(
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
        fn state_skip(
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
        fn state_next(
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
        fn state_skip(
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
        fn state_next(
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
        fn state_skip(
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
        fn state_next(
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
        fn state_skip(
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
        fn state_next(
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
        fn state_skip(
            element: DataState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }
}
