use core::convert::{From, TryInto};
use core::marker::PhantomData;
use std::io::BufRead;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{
    ElementReader, ElementState, IntoReader, ReaderError, StateError, StateNavigation, StateOf,
};
use crate::stream::{parse, serialize, stream_diff};

// Top-Level Reader/State Enums #########################################################################

pub enum States {
    _Document(_DocumentState),
    Void(VoidState),
    Files(FilesState),
    File(FileState),
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
}

pub enum Readers<R> {
    _Document(_DocumentReader<R>),
    Void(VoidReader<R>),
    Files(FilesReader<R>),
    File(FileReader<R>),
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
}

impl<R: BufRead> IntoReader<R> for States {
    type Reader = Readers<R>;

    fn into_reader(self, reader: R) -> Readers<R> {
        match self {
            Self::_Document(state) => Readers::_Document(state.into_reader(reader)),
            Self::Void(state) => Readers::Void(state.into_reader(reader)),
            Self::Files(state) => Readers::Files(state.into_reader(reader)),
            Self::File(state) => Readers::File(state.into_reader(reader)),
            Self::FileName(state) => Readers::FileName(state.into_reader(reader)),
            Self::MimeType(state) => Readers::MimeType(state.into_reader(reader)),
            Self::ModificationTimestamp(state) => {
                Readers::ModificationTimestamp(state.into_reader(reader))
            }
            Self::Data(state) => Readers::Data(state.into_reader(reader)),
        }
    }
}

impl<R> From<Readers<R>> for States {
    fn from(enumed_reader: Readers<R>) -> Self {
        match enumed_reader {
            Readers::_Document(reader) => Self::_Document(reader.state),
            Readers::Void(reader) => Self::Void(reader.state),
            Readers::Files(reader) => Self::Files(reader.state),
            Readers::File(reader) => Self::File(reader.state),
            Readers::FileName(reader) => Self::FileName(reader.state),
            Readers::MimeType(reader) => Self::MimeType(reader.state),
            Readers::ModificationTimestamp(reader) => Self::ModificationTimestamp(reader.state),
            Readers::Data(reader) => Self::Data(reader.state),
        }
    }
}

// _Document Objects #########################################################################

#[derive(Debug, Clone, PartialEq)]
pub struct _DocumentState;
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
    Void(VoidState),
    Files(FilesState),
}

#[derive(Debug, PartialEq)]
pub enum _DocumentNextReaders<R> {
    Void(VoidReader<R>),
    Files(FilesReader<R>),
}

impl From<_DocumentNextStates> for States {
    fn from(enumed_states: _DocumentNextStates) -> Self {
        match enumed_states {
            _DocumentNextStates::Void(state) => state.into(),
            _DocumentNextStates::Files(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<_DocumentNextReaders<R>> for Readers<R> {
    fn from(enumed_readers: _DocumentNextReaders<R>) -> Self {
        match enumed_readers {
            _DocumentNextReaders::Void(reader) => reader.into(),
            _DocumentNextReaders::Files(reader) => reader.into(),
        }
    }
}

impl<R: BufRead> IntoReader<R> for _DocumentNextStates {
    type Reader = _DocumentNextReaders<R>;
    fn into_reader(self, reader: R) -> _DocumentNextReaders<R> {
        match self {
            Self::Void(state) => _DocumentNextReaders::Void(state.into_reader(reader)),
            Self::Files(state) => _DocumentNextReaders::Files(state.into_reader(reader)),
        }
    }
}

impl<R> From<_DocumentNextReaders<R>> for _DocumentNextStates {
    fn from(enumed_reader: _DocumentNextReaders<R>) -> Self {
        match enumed_reader {
            _DocumentNextReaders::Void(reader) => Self::Void(reader.state),
            _DocumentNextReaders::Files(reader) => Self::Files(reader.state),
        }
    }
}

impl _DocumentState {
    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], _DocumentNextStates, StateError> {
        let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
        let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
        let len: usize = len
            .ok_or(nom::Err::Failure(StateError::Unimplemented(
                "TODO: handle optionally unsized elements",
            )))?
            .try_into()
            .expect("overflow in storing element bytelength");

        Ok((
            stream,
            match id {
                <element_defs::VoidDef as ElementDef>::ID => {
                    _DocumentNextStates::Void(VoidState::new(len, self.into()))
                }
                <element_defs::FilesDef as ElementDef>::ID => {
                    _DocumentNextStates::Files(FilesState::new(len, self))
                }
                id => return Err(nom::Err::Failure(StateError::InvalidChildId(None, id))),
            },
        ))
    }
}

impl<R: BufRead> _DocumentReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            state: _DocumentState,
        }
    }

    pub fn next(mut self) -> Result<_DocumentNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

impl<R: BufRead> IntoReader<R> for _DocumentState {
    type Reader = _DocumentReader<R>;
    fn into_reader(self, reader: R) -> _DocumentReader<R> {
        _DocumentReader::new(reader)
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
pub enum FilesNextStates {
    Void(VoidState),
    File(FileState),
    Parent(_DocumentState),
}

#[derive(Debug, PartialEq)]
pub enum FilesNextReaders<R> {
    Void(VoidReader<R>),
    File(FileReader<R>),
    Parent(_DocumentReader<R>),
}

impl From<FilesNextStates> for States {
    fn from(enumed_states: FilesNextStates) -> Self {
        match enumed_states {
            FilesNextStates::Void(state) => state.into(),
            FilesNextStates::File(state) => state.into(),
            FilesNextStates::Parent(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<FilesNextReaders<R>> for Readers<R> {
    fn from(enumed_readers: FilesNextReaders<R>) -> Self {
        match enumed_readers {
            FilesNextReaders::Void(reader) => reader.into(),
            FilesNextReaders::File(reader) => reader.into(),
            FilesNextReaders::Parent(reader) => reader.into(),
        }
    }
}

impl<R: BufRead> IntoReader<R> for FilesNextStates {
    type Reader = FilesNextReaders<R>;
    fn into_reader(self, reader: R) -> FilesNextReaders<R> {
        match self {
            Self::Void(state) => FilesNextReaders::Void(state.into_reader(reader)),
            Self::File(state) => FilesNextReaders::File(state.into_reader(reader)),
            Self::Parent(state) => FilesNextReaders::Parent(state.into_reader(reader)),
        }
    }
}

impl<R> From<FilesNextReaders<R>> for FilesNextStates {
    fn from(enumed_reader: FilesNextReaders<R>) -> Self {
        match enumed_reader {
            FilesNextReaders::Void(reader) => Self::Void(reader.state),
            FilesNextReaders::File(reader) => Self::File(reader.state),
            FilesNextReaders::Parent(reader) => Self::Parent(reader.state),
        }
    }
}

impl FilesState {
    pub fn new(bytes_left: usize, parent_state: _DocumentState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::FilesDef>,
        }
    }
}

impl StateNavigation for FilesState {
    type PrevStates = _DocumentState;
    type NextStates = FilesNextStates;

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
                        <element_defs::VoidDef as ElementDef>::ID => {
                            FilesNextStates::Void(VoidState::new(len, self.into()))
                        }
                        <element_defs::FileDef as ElementDef>::ID => {
                            FilesNextStates::File(FileState::new(len, self))
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
}

impl<R: BufRead> IntoReader<R> for FilesState {
    type Reader = FilesReader<R>;
    fn into_reader(self, reader: R) -> FilesReader<R> {
        FilesReader::new(reader, self)
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
pub enum FileNextStates {
    Void(VoidState),
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
    Parent(FilesState),
}

#[derive(Debug, PartialEq)]
pub enum FileNextReaders<R> {
    Void(VoidReader<R>),
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
    Parent(FilesReader<R>),
}

impl From<FileNextStates> for States {
    fn from(enumed_states: FileNextStates) -> Self {
        match enumed_states {
            FileNextStates::Void(state) => state.into(),
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
            FileNextReaders::Void(reader) => reader.into(),
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
            FileNextReaders::Void(reader) => Self::Void(reader.state),
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

impl<R: BufRead> IntoReader<R> for FileNextStates {
    type Reader = FileNextReaders<R>;
    fn into_reader(self, reader: R) -> FileNextReaders<R> {
        match self {
            Self::Void(state) => FileNextReaders::<R>::Void(state.into_reader(reader)),
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
    pub fn new(bytes_left: usize, parent_state: FilesState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::FileDef>,
        }
    }
}

impl StateNavigation for FileState {
    type PrevStates = FilesState;
    type NextStates = FileNextStates;

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
                        <element_defs::VoidDef as ElementDef>::ID => {
                            FileNextStates::Void(VoidState::new(len, self.into()))
                        }
                        <element_defs::FileNameDef as ElementDef>::ID => {
                            FileNextStates::FileName(FileNameState::new(len, self))
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::MimeType(MimeTypeState::new(len, self))
                        }
                        <element_defs::ModificationTimestampDef as ElementDef>::ID => {
                            FileNextStates::ModificationTimestamp(ModificationTimestampState::new(
                                len, self,
                            ))
                        }
                        <element_defs::DataDef as ElementDef>::ID => {
                            FileNextStates::Data(DataState::new(len, self))
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
}

impl<R: BufRead> IntoReader<R> for FileState {
    type Reader = FileReader<R>;
    fn into_reader(self, reader: R) -> FileReader<R> {
        FileReader::new(reader, self)
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
    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::FileNameDef>,
        }
    }

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (FileState, &str), StateError> {
        let (stream, data) =
            parse::unicode_str(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl StateNavigation for FileNameState {
    type PrevStates = FileState;
    type NextStates = FileState;

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

    pub fn read(&mut self) -> Result<&str, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: BufRead> IntoReader<R> for FileNameState {
    type Reader = FileNameReader<R>;
    fn into_reader(self, reader: R) -> FileNameReader<R> {
        FileNameReader::new(reader, self)
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
    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::MimeTypeDef>,
        }
    }

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (FileState, &str), StateError> {
        let (stream, data) =
            parse::ascii_str(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl StateNavigation for MimeTypeState {
    type PrevStates = FileState;
    type NextStates = FileState;

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

    pub fn read(&mut self) -> Result<&str, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: BufRead> IntoReader<R> for MimeTypeState {
    type Reader = MimeTypeReader<R>;
    fn into_reader(self, reader: R) -> MimeTypeReader<R> {
        MimeTypeReader::new(reader, self)
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
    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::ModificationTimestampDef>,
        }
    }

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (FileState, i64), StateError> {
        let (stream, data) = parse::date(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl StateNavigation for ModificationTimestampState {
    type PrevStates = FileState;
    type NextStates = FileState;

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

    pub fn read(&mut self) -> Result<i64, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: BufRead> IntoReader<R> for ModificationTimestampState {
    type Reader = ModificationTimestampReader<R>;
    fn into_reader(self, reader: R) -> ModificationTimestampReader<R> {
        ModificationTimestampReader::new(reader, self)
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
    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::DataDef>,
        }
    }

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (FileState, &[u8]), StateError> {
        let (stream, data) = parse::binary(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl StateNavigation for DataState {
    type PrevStates = FileState;
    type NextStates = FileState;

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

    pub fn read(&mut self) -> Result<&[u8], ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: BufRead> IntoReader<R> for DataState {
    type Reader = DataReader<R>;
    fn into_reader(self, reader: R) -> DataReader<R> {
        DataReader::new(reader, self)
    }
}

// Void Objects #########################################################################

pub type VoidState = ElementState<element_defs::VoidDef, VoidPrevStates>;
pub type VoidReader<R> = ElementReader<R, VoidState>;

impl From<VoidState> for States {
    fn from(state: VoidState) -> Self {
        Self::Void(state)
    }
}

impl<R: BufRead> From<VoidReader<R>> for Readers<R> {
    fn from(reader: VoidReader<R>) -> Self {
        Self::Void(reader)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VoidPrevStates {
    _Document(_DocumentState),
    Files(FilesState),
    File(FileState),
}
#[derive(Debug, PartialEq)]
pub enum VoidPrevReaders<R> {
    _Document(_DocumentReader<R>),
    Files(FilesReader<R>),
    File(FileReader<R>),
}

impl From<VoidPrevStates> for States {
    fn from(enumed_states: VoidPrevStates) -> Self {
        match enumed_states {
            VoidPrevStates::_Document(state) => state.into(),
            VoidPrevStates::Files(state) => state.into(),
            VoidPrevStates::File(state) => state.into(),
        }
    }
}

impl<R: BufRead> From<VoidPrevReaders<R>> for Readers<R> {
    fn from(enumed_readers: VoidPrevReaders<R>) -> Self {
        match enumed_readers {
            VoidPrevReaders::_Document(reader) => reader.into(),
            VoidPrevReaders::Files(reader) => reader.into(),
            VoidPrevReaders::File(reader) => reader.into(),
        }
    }
}

impl<R: BufRead> IntoReader<R> for VoidPrevStates {
    type Reader = VoidPrevReaders<R>;
    fn into_reader(self, reader: R) -> VoidPrevReaders<R> {
        match self {
            Self::_Document(state) => VoidPrevReaders::_Document(state.into_reader(reader)),
            Self::Files(state) => VoidPrevReaders::Files(state.into_reader(reader)),
            Self::File(state) => VoidPrevReaders::File(state.into_reader(reader)),
        }
    }
}

impl<R> From<VoidPrevReaders<R>> for VoidPrevStates {
    fn from(enumed_reader: VoidPrevReaders<R>) -> Self {
        match enumed_reader {
            VoidPrevReaders::_Document(reader) => Self::_Document(reader.state),
            VoidPrevReaders::Files(reader) => Self::Files(reader.state),
            VoidPrevReaders::File(reader) => Self::File(reader.state),
        }
    }
}

impl From<_DocumentState> for VoidPrevStates {
    fn from(state: _DocumentState) -> Self {
        Self::_Document(state)
    }
}

impl From<FilesState> for VoidPrevStates {
    fn from(state: FilesState) -> Self {
        Self::Files(state)
    }
}

impl From<FileState> for VoidPrevStates {
    fn from(state: FileState) -> Self {
        Self::File(state)
    }
}

impl VoidState {
    pub fn new(bytes_left: usize, parent_state: VoidPrevStates) -> Self {
        Self {
            bytes_left,
            parent_state,
            _phantom: PhantomData::<element_defs::VoidDef>,
        }
    }
}

impl StateNavigation for VoidState {
    type PrevStates = VoidPrevStates;
    type NextStates = VoidPrevStates;

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
}

impl<R: BufRead> IntoReader<R> for VoidState {
    type Reader = VoidReader<R>;
    fn into_reader(self, reader: R) -> VoidReader<R> {
        VoidReader::new(reader, self)
    }
}

// Tests #########################################################################

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    mod document {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                _DocumentState,
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], _DocumentNextStates::Files(FilesState{bytes_left: 2, parent_state: _DocumentState, _phantom: PhantomData}))
            ),
        )]
        fn state_next(
            element: _DocumentState,
            source: &'static [u8],
            expt_result: (&'static [u8], _DocumentNextStates),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }
    }

    mod files {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FilesState{bytes_left: 5, parent_state: _DocumentState, _phantom: PhantomData},
                &[0x61, 0x46, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::File(FileState{bytes_left: 2, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::Parent(_DocumentState))
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
                FilesState{bytes_left: 5, parent_state: _DocumentState, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], _DocumentState)
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
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::FileName(FileNameState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x4D, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::MimeType(MimeTypeState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x54, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::ModificationTimestamp(ModificationTimestampState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x64, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Data(DataState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Parent(FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}))
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
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 1, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FilesState{bytes_left: 1, parent_state: _DocumentState, _phantom: PhantomData})
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
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}),
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
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData})
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
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}),
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
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData})
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
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}),
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
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData})
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
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}),
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
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState, _phantom: PhantomData}, _phantom: PhantomData})
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
