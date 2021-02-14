use std::ops::Bound;


enum ElementDefError {
    RedundantId,
    MultiOccuringHasDefault,
    UnknownSizedInKnownSizedParent,
    UnknownSizedIsRecursive,

}

trait ElementDefinition {
    fn is_valid(&self) -> Result<(), ElementDefError>;
}


enum RangeDef<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}


#[derive(Debug)]
struct MasterElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    length: Option<usize>,
    unknown_size_allowed: Option<bool>,
    recursive: Option<bool>,
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct UIntElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    range: Option<RangeDef>,
    length: Option<usize>,
    default: Option<u64>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct IntElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    range: Option<RangeDef>,
    length: Option<usize>,
    default: Option<i64>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct FloatElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    range: Option<RangeDef>,
    length: Option<usize>,
    default: Option<f64>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct DateElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    range: Option<RangeDef>,
    length: Option<usize>,
    default: Option<i64>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct StringElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    length: Option<usize>,
    default: Option<&'static str>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct Utf8ElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    length: Option<usize>,
    default: Option<&'static str>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}


#[derive(Debug)]
struct BinaryElementDef {
    name: &'static str,
    path: (),
    id: usize,

    minOccurs: Option<usize>,
    maxOccurs: Option<usize>,
    length: Option<usize>,
    default: Option<&'static [u8]>
    recurring: Option<bool>,
    min_version: Option<u64>,
    max_version: Option<u64>,
}
