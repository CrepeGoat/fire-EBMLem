use crate::element_defs::{
    BinaryElementDef, DateElementDef, ElementDef, FloatElementDef, IntElementDef, StringElementDef,
    UIntElementDef, UTF8ElementDef,
};
use crate::stream::parse;

use core::convert::From;
use core::fmt;
use core::fmt::Debug;
use core::marker::PhantomData;

#[derive(Debug, Clone, PartialEq)]
pub struct ElementState<E, S> {
    pub bytes_left: usize,
    pub parent_state: S,
    pub _phantom: PhantomData<E>,
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("invalid subelement id {1} (parent id = {:?})", *.0)]
    InvalidChildId(Option<u32>, u32),
    #[error("unimplemeted feature: {0}")]
    Unimplemented(&'static str),
    #[error("error parsing token")]
    BadToken,
}

impl From<()> for StateError {
    fn from(_value: ()) -> Self {
        Self::BadToken
    }
}

pub trait SkipStateNavigation {
    type PrevStates;

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], Self::PrevStates, StateError>;
}

pub trait NextStateNavigation {
    type NextStates;

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], Self::NextStates, StateError>;
}

pub struct UIntParserMarker;
pub struct IntParserMarker;
pub struct FloatParserMarker;
pub struct DateParserMarker;
pub struct StringParserMarker;
pub struct UTF8ParserMarker;
pub struct BinaryParserMarker;

pub trait ParserMarker {}
impl ParserMarker for UIntParserMarker {}
impl ParserMarker for IntParserMarker {}
impl ParserMarker for FloatParserMarker {}
impl ParserMarker for DateParserMarker {}
impl ParserMarker for StringParserMarker {}
impl ParserMarker for UTF8ParserMarker {}
impl ParserMarker for BinaryParserMarker {}

pub trait StateDataParser<'a, M: ParserMarker, T: 'a> {
    type NextState;
    fn read(self, stream: &'a [u8]) -> nom::IResult<&[u8], (Self::NextState, T), StateError>;
}

impl<E: UIntElementDef, S> StateDataParser<'_, UIntParserMarker, u64> for ElementState<E, S> {
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, u64), StateError> {
        let (stream, data) = parse::uint(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<E: IntElementDef, S> StateDataParser<'_, IntParserMarker, i64> for ElementState<E, S> {
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, i64), StateError> {
        let (stream, data) = parse::int(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<E: FloatElementDef, S> StateDataParser<'_, FloatParserMarker, f64> for ElementState<E, S> {
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, f64), StateError> {
        let (stream, data) = parse::float64(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<E: DateElementDef, S> StateDataParser<'_, DateParserMarker, i64> for ElementState<E, S> {
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, i64), StateError> {
        let (stream, data) = parse::date(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<'a, E: StringElementDef, S> StateDataParser<'a, StringParserMarker, &'a str>
    for ElementState<E, S>
{
    type NextState = S;

    fn read(self, stream: &'a [u8]) -> nom::IResult<&[u8], (S, &'a str), StateError> {
        let (stream, data) =
            parse::ascii_str(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<'a, E: UTF8ElementDef, S> StateDataParser<'a, UTF8ParserMarker, &'a str>
    for ElementState<E, S>
{
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, &str), StateError> {
        let (stream, data) =
            parse::unicode_str(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

impl<'a, E: BinaryElementDef, S> StateDataParser<'a, BinaryParserMarker, &'a [u8]>
    for ElementState<E, S>
{
    type NextState = S;

    fn read(self, stream: &[u8]) -> nom::IResult<&[u8], (S, &[u8]), StateError> {
        let (stream, data) = parse::binary(stream, self.bytes_left).map_err(nom::Err::convert)?;

        Ok((stream, (self.parent_state, data)))
    }
}

// marks a state; binds a state type to a single element type
pub trait StateOf {
    type Element;
}

impl<E, S> StateOf for ElementState<E, S> {
    type Element = E;
}
impl StateOf for () {
    type Element = ();
}

#[derive(Debug, PartialEq)]
pub struct ElementReader<R, S> {
    pub reader: R,
    pub state: S,
}

#[derive(thiserror::Error, Debug)]
pub enum ReaderError {
    #[error("IOError: {0}")]
    Io(#[from] std::io::Error),
    #[error("ParseError: {0}")]
    Parse(#[from] nom::Err<StateError>),
}

pub trait SkipReaderNavigation<R> {
    type PrevReaders;

    fn skip(self) -> Result<Self::PrevReaders, ReaderError>;
}

pub trait NextReaderNavigation<R> {
    type NextReaders;

    fn next(self) -> Result<Self::NextReaders, ReaderError>;
}

impl<R: std::io::BufRead, S: SkipStateNavigation> SkipReaderNavigation<R> for ElementReader<R, S>
where
    S::PrevStates: IntoReader<R>,
{
    type PrevReaders = <S::PrevStates as IntoReader<R>>::Reader;

    fn skip(mut self) -> Result<Self::PrevReaders, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

impl<R: std::io::BufRead, S: NextStateNavigation> NextReaderNavigation<R> for ElementReader<R, S>
where
    S::NextStates: IntoReader<R>,
{
    type NextReaders = <S::NextStates as IntoReader<R>>::Reader;

    fn next(mut self) -> Result<Self::NextReaders, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

pub trait ReaderDataParser<'a, R, M: ParserMarker, T: 'a> {
    fn read(&'a mut self) -> Result<T, ReaderError>;
}

impl<R: std::io::BufRead, E: UIntElementDef + Clone, S: Clone>
    ReaderDataParser<'_, R, UIntParserMarker, u64> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<u64, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: std::io::BufRead, E: IntElementDef + Clone, S: Clone>
    ReaderDataParser<'_, R, IntParserMarker, i64> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<i64, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: std::io::BufRead, E: FloatElementDef + Clone, S: Clone>
    ReaderDataParser<'_, R, FloatParserMarker, f64> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<f64, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<R: std::io::BufRead, E: DateElementDef + Clone, S: Clone>
    ReaderDataParser<'_, R, DateParserMarker, i64> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<i64, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<'a, R: std::io::BufRead, E: StringElementDef + Clone, S: Clone>
    ReaderDataParser<'a, R, StringParserMarker, &'a str> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<&str, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<'a, R: std::io::BufRead, E: UTF8ElementDef + Clone, S: Clone>
    ReaderDataParser<'a, R, UTF8ParserMarker, &'a str> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<&str, ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

impl<'a, R: std::io::BufRead, E: BinaryElementDef + Clone, S: Clone>
    ReaderDataParser<'a, R, BinaryParserMarker, &'a [u8]> for ElementReader<R, ElementState<E, S>>
{
    fn read(&mut self) -> Result<&[u8], ReaderError> {
        let stream = self.reader.fill_buf()?;
        let (_, (_, data)) = self.state.clone().read(stream)?;

        Ok(data)
    }
}

pub trait IntoReader<R: std::io::BufRead> {
    type Reader;

    fn into_reader(self, reader: R) -> Self::Reader;
}

#[macro_export]
macro_rules! impl_into_reader {
    ( $States:ident, $Readers:ident, [ $( $ElementName:ident ),* ] ) => {
        impl<R: BufRead> IntoReader<R> for $States {
            type Reader = $Readers<R>;
            fn into_reader(self, reader: R) -> Self::Reader {
                match self {
                    $(
                        Self::$ElementName(state) => Self::Reader::$ElementName(state.into_reader(reader)),
                    )*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_readers_for_states {
    ( $Readers:ident, $States:ident, [ $( $ElementName:ident ),* ] ) => {
        impl<R> From<$Readers<R>> for $States {
            fn from(enumed_reader: $Readers<R>) -> Self {
                match enumed_reader {
                    $(
                        $Readers::$ElementName(reader) => Self::$ElementName(reader.state),
                    )*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_substates_for_states {
    ( $SubStates:ident, $States:ident, [ $( $ElementName:ident ),* ] ) => {
        impl From<$SubStates> for $States {
            fn from(enumed_states: $SubStates) -> Self {
                match enumed_states {
                    $(
                        $SubStates::$ElementName(state) => state.into(),
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_from_subreaders_for_readers {
    ( $SubReaders:ident, $Readers:ident, [ $( $ElementName:ident ),* ] ) => {
        impl<R: BufRead> From<$SubReaders<R>> for $Readers<R> {
            fn from(enumed_states: $SubReaders<R>) -> Self {
                match enumed_states {
                    $(
                        $SubReaders::$ElementName(state) => state.into(),
                    )*
                }
            }
        }
    }
}
