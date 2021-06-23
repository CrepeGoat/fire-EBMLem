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

impl<E, S> StateOf for ElementState<E, S> {
    type Element = E;
}
impl StateOf for () {
    type Element = ();
}

#[macro_export]
macro_rules! NestedElementStates {
    ($e:ty, $( $e_list:ty ),+) => {ElementState<$e, NestedElementStates!($($e_list),+)>};
    ($e:ty) => {ElementState<$e, NestedElementStates!()>};
    () => {ElementState<(), ()>};
}
