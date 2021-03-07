pub use std::ops::Bound;

pub enum RangeDef<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

pub enum ElementParsingStage<T, G> {
    Start,
    Child(T),
    Global(T, G)
    Finish,
    EndOfStream,
}

pub trait Element {
    // name
    // path
    const ID: u32;

    const MIN_OCCURS: Option<usize>;
    const MAX_OCCURS: Option<usize>;
    const LENGTH: Option<RangeDef<usize>>;
    const RECURRING: Option<bool>;
    const MIN_VERSION: Option<u64>;
    const MAX_VERSION: Option<u64>;
}

pub trait MasterElement: Element {
    const UNKNOWN_SIZE_ALLOWED: Option<bool>;
    const RECURSIVE: Option<bool>;

    type SubElements;
    fn inner(&self) -> &ElementParsingStage<Self::SubElements>;
    fn inner_mut(&mut self) -> &mut ElementParsingStage<Self::SubElements>;
}

pub trait UIntElement: Element {
    const RANGE: Option<RangeDef<u64>>;
    const DEFAULT: Option<u64>;

    fn read(&self, input: &[u8]) -> u64 {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: u64) {
        todo!()
    }
}

pub trait IntElement: Element {
    const RANGE: Option<RangeDef<i64>>;
    const DEFAULT: Option<i64>;

    fn read(&self, input: &[u8]) -> i64 {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: i64) {
        todo!()
    }
}

pub trait FloatElement: Element {
    const RANGE: Option<RangeDef<f64>>;
    const DEFAULT: Option<f64>;

    fn read(&self, input: &[u8]) -> f64 {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: f64) {
        todo!()
    }
}

pub trait DateElement: Element {
    const RANGE: Option<RangeDef<i64>>;
    const DEFAULT: Option<i64>;

    fn read(&self, input: &[u8]) -> i64 {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: i64) {
        todo!()
    }
}

pub trait StringElement: Element {
    const DEFAULT: Option<&'static str>;

    fn read(&self, input: &[u8]) -> &'static str {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: &'static str) {
        todo!()
    }
}

pub trait UTF8Element: Element {
    const DEFAULT: Option<&'static str>;

    fn read(&self, input: &[u8]) -> &'static str {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: &'static str) {
        todo!()
    }
}

pub trait BinaryElement: Element {
    const DEFAULT: Option<&'static [u8]>;

    fn read(&self, input: &[u8]) -> &'static [u8] {
        todo!()
    }
    fn overwrite(&self, output: &mut [u8], value: &'static [u8]) {
        todo!()
    }
}
