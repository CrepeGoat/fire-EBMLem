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

pub #[derive(Debug)]
enum StateError {
    InvalidChildID(Option<u32>, u32),
    Unimplemented(&'static str),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChildID(super_id, sub_id) => write!(
                f, "invalid subelement id {} for superelement id {}", sub_id, super_id
            ),
            Self::Unimplemented(string) => write!(f, "Unimplemented feature: {}", id),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ElementReader<R, S> {
    pub reader: R,
    pub state: S,
}

#[derive(Debug)]
pub enum ReaderError {
    IO(std::io::Error),
    Parse(nom::Err<StateError>),
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => write!(f, "IOError: {}", error),
            Self::Parse(error) => write!(f, "ParseError: {}", error),
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
