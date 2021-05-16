pub use std::ops::Bound;

pub enum Range<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

pub trait ElementDef {
    // name
    const ID: u32;

    const MIN_OCCURS: usize; // defaults to 0
    const MAX_OCCURS: Option<usize>;
    const LENGTH: Option<Range<usize>>;
    const RECURRING: bool; // defaults to false
    const MIN_VERSION: u64; // defaults to 1
    const MAX_VERSION: u64; // defaults to "EBMLSchema" version
}

pub trait MasterElementDef: ElementDef {
    const UNKNOWN_SIZE_ALLOWED: bool; // defaults to false
    const RECURSIVE: bool; // defaults to false
}

pub trait UIntElementDef: ElementDef {
    const RANGE: Option<Range<u64>>;
    const DEFAULT: Option<u64>;
}

pub trait IntElementDef: ElementDef {
    const RANGE: Option<Range<i64>>;
    const DEFAULT: Option<i64>;
}

pub trait FloatElementDef: ElementDef {
    const RANGE: Option<Range<f64>>;
    const DEFAULT: Option<f64>;
}

pub trait DateElementDef: ElementDef {
    const RANGE: Option<Range<i64>>;
    const DEFAULT: Option<i64>;
}

pub trait StringElementDef: ElementDef {
    const DEFAULT: Option<&'static str>;
}

pub trait UTF8ElementDef: ElementDef {
    const DEFAULT: Option<&'static str>;
}

pub trait BinaryElementDef: ElementDef {
    const DEFAULT: Option<&'static [u8]>;
}
