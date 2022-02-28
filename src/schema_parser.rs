use core::default::Default;
use core::str::FromStr;
use std::io::Read;

use core::ops::Bound;

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
pub enum Range<T> {
    IsExactly(T),
    Excludes(T),
    IsWithin(Bound<T>, Bound<T>),
}

impl<T> Default for Range<T> {
    fn default() -> Self {
        Self::IsWithin(Bound::Unbounded, Bound::Unbounded)
    }
}

impl<T: FromStr> FromStr for Range<T> {
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

trait TypeString {
    const TYPE: &'static str;
}

#[derive(Debug, Default)]
struct ElementAttrs {
    name: String,
    path: String,
    id: u32,
    min_occurs: usize, // defaults to 0
    max_occurs: Option<usize>,
}

#[derive(Debug, Default)]
struct MasterElementAttrs {
    unknown_size_allowed: bool, // defaults to false
    recursive: bool,            // defaults to false
}

impl TypeString for MasterElementAttrs {
    const TYPE: &'static str = "master";
}

#[derive(Debug, Default)]
struct UIntElementAttrs {
    range: Range<u64>, // defaults to (Unbounded, Unbounded)
    default: Option<u64>,
}

impl TypeString for UIntElementAttrs {
    const TYPE: &'static str = "uinteger";
}

#[derive(Debug, Default)]
struct IntElementAttrs {
    range: Range<i64>, // defaults to (Unbounded, Unbounded)
    default: Option<i64>,
}

impl TypeString for IntElementAttrs {
    const TYPE: &'static str = "integer";
}

#[derive(Debug, Default)]
struct FloatElementAttrs {
    range: Range<f64>, // defaults to (Unbounded, Unbounded)
    default: Option<f64>,
}

impl TypeString for FloatElementAttrs {
    const TYPE: &'static str = "float";
}

#[derive(Debug, Default)]
struct DateElementAttrs {
    range: Range<i64>, // defaults to (Unbounded, Unbounded)
    default: Option<i64>,
}

impl TypeString for DateElementAttrs {
    const TYPE: &'static str = "date";
}

#[derive(Debug, Default)]
struct StringElementAttrs {
    default: Option<String>,
}

impl TypeString for StringElementAttrs {
    const TYPE: &'static str = "string";
}

#[derive(Debug, Default)]
struct Utf8ElementAttrs {
    default: Option<String>,
}

impl TypeString for Utf8ElementAttrs {
    const TYPE: &'static str = "utf-8";
}

#[derive(Debug, Default)]
struct BinaryElementAttrs {
    default: Option<Vec<u8>>,
}

impl TypeString for BinaryElementAttrs {
    const TYPE: &'static str = "binary";
}

enum ElementDescriptor {
    Master(ElementAttrs, MasterElementAttrs),
    UInt(ElementAttrs, UIntElementAttrs),
    Int(ElementAttrs, IntElementAttrs),
    Float(ElementAttrs, FloatElementAttrs),
    Date(ElementAttrs, DateElementAttrs),
    String(ElementAttrs, StringElementAttrs),
    Utf8(ElementAttrs, Utf8ElementAttrs),
    Binary(ElementAttrs, BinaryElementAttrs),
}

fn parse_schema<R: Read>(reader: R) -> impl Iterator<Item = ElementDescriptor> {
    // TODO: replace w/ serde implementation?

    let parser = EventReader::new(reader);

    let mut depth = 0;
    parser.into_iter().filter_map(move |e| match e {
        Ok(XmlEvent::StartElement {
            name, attributes, ..
        }) => {
            depth += 1;
            if depth != 1 {
                return None;
            }

            assert_eq!(name.local_name, "element".to_string());
            assert_eq!(name.namespace, None);
            assert_eq!(name.prefix, None);

            let mut attributes = attributes
                .into_iter()
                .map(|owned_attr| (owned_attr.name.local_name, owned_attr.value))
                .collect::<std::collections::HashMap<_, _>>();

            let elem_attrs = ElementAttrs {
                name: attributes
                    .remove("name")
                    .expect("'name' attribute is required"),
                path: attributes
                    .remove("path")
                    .expect("'path' attribute is required"),
                id: attributes
                    .remove("id")
                    .expect("'id' attribute is required")
                    .parse()
                    .unwrap(),
                min_occurs: attributes
                    .remove("min_occurs")
                    .map(|v| v.parse().unwrap())
                    .unwrap_or_default(),
                max_occurs: attributes.remove("max_occurs").map(|v| v.parse().unwrap()),
            };

            let type_ = attributes
                .remove("type")
                .expect("'id' attribute is required");
            Some(match &type_[..] {
                "integer" => ElementDescriptor::Int(
                    elem_attrs,
                    IntElementAttrs {
                        range: attributes
                            .remove("range")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                        default: attributes.remove("default").map(|v| v.parse().unwrap()),
                    },
                ),
                "uinteger" => ElementDescriptor::UInt(
                    elem_attrs,
                    UIntElementAttrs {
                        range: attributes
                            .remove("range")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                        default: attributes.remove("default").map(|v| v.parse().unwrap()),
                    },
                ),
                "float" => ElementDescriptor::Float(
                    elem_attrs,
                    FloatElementAttrs {
                        range: attributes
                            .remove("range")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                        default: attributes.remove("default").map(|v| v.parse().unwrap()),
                    },
                ),
                "string" => ElementDescriptor::String(
                    elem_attrs,
                    StringElementAttrs {
                        default: attributes.remove("default"),
                    },
                ),
                "date" => ElementDescriptor::Date(
                    elem_attrs,
                    DateElementAttrs {
                        range: attributes
                            .remove("range")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                        default: attributes.remove("default").map(|v| v.parse().unwrap()),
                    },
                ),
                "utf-8" => ElementDescriptor::Utf8(
                    elem_attrs,
                    Utf8ElementAttrs {
                        default: attributes.remove("default"),
                    },
                ),
                "master" => ElementDescriptor::Master(
                    elem_attrs,
                    MasterElementAttrs {
                        unknown_size_allowed: attributes
                            .remove("unknownsizeallowed")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                        recursive: attributes
                            .remove("recursive")
                            .map(|v| v.parse().unwrap())
                            .unwrap_or_default(),
                    },
                ),
                "binary" => ElementDescriptor::Binary(
                    elem_attrs,
                    BinaryElementAttrs {
                        default: attributes.remove("default").map(|v| v.bytes().collect()),
                    },
                ),
                _ => panic!("invalid element type {}", type_),
            })
        }
        Ok(XmlEvent::EndElement { name: _ }) => {
            depth -= 1;
            None
        }
        Err(e) => {
            panic!("Error: {}", e);
        }
        _ => None,
    })
}
