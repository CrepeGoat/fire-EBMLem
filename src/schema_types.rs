use std::ops::Bound;

enum RangeDef<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

trait Element {
    // name
    // path
    const id: u32;

    const minOccurs: Option<usize>;
    const maxOccurs: Option<usize>;
    const length: Option<usize>;
    const recurring: Option<bool>;
    const min_version: Option<u64>;
    const max_version: Option<u64>;
}

trait MasterElement: Element {
    const unknown_size_allowed: Option<bool>;
    const recursive: Option<bool>;
}

trait UIntElement: Element {
    const range: Option<RangeDef<u64>>;
    const default: Option<u64>;
}

trait IntElement: Element {
    const range: Option<RangeDef<i64>>;
    const default: Option<i64>;
}

trait FloatElement: Element {
    const range: Option<RangeDef<f64>>;
    const default: Option<f64>;
}

trait DateElement: Element {
    const range: Option<RangeDef<i64>>;
    const default: Option<i64>;
}

trait StringElement: Element {
    const default: Option<&'static str>;
}

trait UTF8Element: Element {
    const default: Option<&'static str>;
}

trait BinaryElement: Element {
    const default: Option<&'static [u8]>;
}
