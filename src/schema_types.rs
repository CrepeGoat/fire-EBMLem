pub use std::ops::Bound;

pub enum RangeDef<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
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

    type Elements;

    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Self::Elements, ()>
    where
        Self: std::marker::Sized,
    {
        unreachable!()
    }
    fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Self::Elements, ()>;
}

#[macro_export]
macro_rules! Element_next {
    () => {
        fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Self::Elements, ()> {
            self.skip(stream)
        }
    };
    ( $( $subelement:ident ),* ) => {
        fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Self::Elements, ()> {
            match self {
                Self { bytes_left: 0, parent: p} => Ok((stream, p.into())),
                _ => {
                    let orig_stream = stream;

                    let (stream, id) = parse::element_id(stream)?;
                    let (stream, len) = parse::element_len(stream)?;
                    let len: usize = len
                        .expect("todo: handle optionally unsized elements")
                        .try_into()
                        .expect("overflow in storing element bytelength");

                    self.bytes_left -= len + stream_diff(orig_stream, stream);

                    Ok((
                        stream,
                        match id {
                            $(
                                $subelement::ID => $subelement {
                                    bytes_left: len,
                                    parent: self,
                                }
                                .into(),
                            )*
                            _ => return Err(nom::Err::Failure(())),
                        },
                    ))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! Element_skip {
    () => {
        fn skip<'a>(self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Self::Elements, ()> {
            let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
            Ok((stream, self.parent.into()))
        }
    };
}

pub trait MasterElement: Element {
    const UNKNOWN_SIZE_ALLOWED: Option<bool>;
    const RECURSIVE: Option<bool>;
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

#[macro_export]
macro_rules! Elements {
    ( $( $element_type:ident ),* ) => {
        #[derive(Debug, Clone, PartialEq)]
        enum Elements {
            None,
            $(
                $element_type($element_type),
            )*
        }

        $(
            impl From<$element_type> for Elements {
                fn from(element: $element_type) -> Elements {
                    Elements::$element_type(element)
                }
            }
        )*
    };
}
