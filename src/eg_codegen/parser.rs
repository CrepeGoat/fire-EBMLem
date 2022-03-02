use core::convert::{From, TryInto};
use core::marker::PhantomData;
use std::io::BufRead;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{
    ElementReader, ElementState, IntoReader, ReaderError, StateDataParser, StateError,
    StateNavigation, StateOf,
};
use crate::stream::{parse, serialize, stream_diff};
use crate::{
    impl_from_readers_for_states, impl_from_subreaders_for_readers, impl_from_substates_for_states,
    impl_into_reader,
};

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

impl_into_reader!(
    States,
    Readers,
    [
        _Document,
        Void,
        Files,
        File,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data
    ]
);

impl_from_readers_for_states!(
    Readers,
    States,
    [
        _Document,
        Void,
        Files,
        File,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data
    ]
);

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

impl_from_substates_for_states!(_DocumentNextStates, States, [Void, Files]);
impl_from_subreaders_for_readers!(_DocumentNextReaders, Readers, [Void, Files]);

impl_into_reader!(_DocumentNextStates, _DocumentNextReaders, [Void, Files]);
impl_from_readers_for_states!(_DocumentNextReaders, _DocumentNextStates, [Void, Files]);

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

impl_from_substates_for_states!(FilesNextStates, States, [Void, File, Parent]);
impl_from_subreaders_for_readers!(FilesNextReaders, Readers, [Void, File, Parent]);

impl_into_reader!(FilesNextStates, FilesNextReaders, [Void, File, Parent]);
impl_from_readers_for_states!(FilesNextReaders, FilesNextStates, [Void, File, Parent]);

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

impl_from_substates_for_states!(
    FileNextStates,
    States,
    [
        Void,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data,
        Parent
    ]
);
impl_from_subreaders_for_readers!(
    FileNextReaders,
    Readers,
    [
        Void,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data,
        Parent
    ]
);

impl_into_reader!(
    FileNextStates,
    FileNextReaders,
    [
        Void,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data,
        Parent
    ]
);
impl_from_readers_for_states!(
    FileNextReaders,
    FileNextStates,
    [
        Void,
        FileName,
        MimeType,
        ModificationTimestamp,
        Data,
        Parent
    ]
);

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

impl_from_substates_for_states!(VoidPrevStates, States, [_Document, Files, File]);
impl_from_subreaders_for_readers!(VoidPrevReaders, Readers, [_Document, Files, File]);

impl_into_reader!(VoidPrevStates, VoidPrevReaders, [_Document, Files, File]);
impl_from_readers_for_states!(VoidPrevReaders, VoidPrevStates, [_Document, Files, File]);

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
