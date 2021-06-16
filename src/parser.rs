use crate::element_defs;

use core::marker::PhantomData;

pub struct ElementState<E, S> {
    stream_start: usize,
    stream_end: usize,
    parent_state: S,
    _phantom: PhantomData<E>,
}

// marks a state; binds a state type to a single element type
pub trait StateOf {
    type Element;
}

impl<E, S> StateOf<E> for ElementState<E, S> {}
impl StateOf<()> for () {}

#[macro_export]
macro_rules! NestedElementStates {
    ($e:ident, $( $e_list:ident ),*) => {ElementState<$e, NestedElementStates[$e_list]>};
    () => {ElementState<(), ()>};
}
