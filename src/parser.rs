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

pub trait StateNavigation {
    type PrevStates;
    type NextStates;

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], Self::PrevStates, StateError>;
    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], Self::NextStates, StateError>;
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

pub trait IntoReader<R: std::io::BufRead> {
    type Reader;

    fn into_reader(self, reader: R) -> Self::Reader;
}
