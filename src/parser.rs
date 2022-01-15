use crate::element_defs::ElementDef;
use core::convert::From;

use core::fmt::Debug;
use core::marker::PhantomData;

// marks an object with a single respective element type
pub trait BoundTo
where
    Self::Element: ElementDef,
{
    type Element;
}

pub fn get_element_id<T: BoundTo>(_: &T) -> u32 {
    <T as BoundTo>::Element::ID
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementState<E, S> {
    pub bytes_left: usize,
    pub parent_state: S,
    pub _phantom: PhantomData<E>,
}

impl<E: ElementDef, S> BoundTo for ElementState<E, S> {
    type Element = E;
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

#[derive(Debug, PartialEq)]
pub struct ElementReader<R, S> {
    pub reader: R,
    pub state: S,
}

impl<R, S: BoundTo> BoundTo for ElementReader<R, S> {
    type Element = S::Element;
}

#[derive(thiserror::Error, Debug)]
pub enum ReaderError {
    #[error("IOError: {0}")]
    Io(#[from] std::io::Error),
    #[error("ParseError: {0}")]
    Parse(#[from] nom::Err<StateError>),
}
