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

#[derive(Debug)]
pub enum ReaderError {
    IO(std::io::Error),
    Parse(nom::Err<()>),
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => write!(f, "IOError: {}", error),
            Self::Parse(error) => write!(f, "ParseError"), // TODO: properly print error
        }
    }
}

impl From<std::io::Error> for ReaderError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<nom::Err<()>> for ReaderError {
    fn from(err: nom::Err<()>) -> Self {
        Self::Parse(err)
    }
}
