use core::default::Default;
use core::str::FromStr;
use std::io::Read;

use core::ops::Bound;

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
struct ElementAttrs {
    name: String,
    path: String,
    id: u32,
    min_occurs: usize, // defaults to 0
    max_occurs: Option<usize>,
}

#[derive(Debug, PartialEq)]
struct MasterElementAttrs {
    unknown_size_allowed: bool, // defaults to false
    recursive: bool,            // defaults to false
}

impl TypeString for MasterElementAttrs {
    const TYPE: &'static str = "master";
}

#[derive(Debug, PartialEq)]
struct UIntElementAttrs {
    range: Range<u64>, // defaults to (Unbounded, Unbounded)
    default: Option<u64>,
}

impl TypeString for UIntElementAttrs {
    const TYPE: &'static str = "uinteger";
}

#[derive(Debug, PartialEq)]
struct IntElementAttrs {
    range: Range<i64>, // defaults to (Unbounded, Unbounded)
    default: Option<i64>,
}

impl TypeString for IntElementAttrs {
    const TYPE: &'static str = "integer";
}

#[derive(Debug, PartialEq)]
struct FloatElementAttrs {
    range: Range<f64>, // defaults to (Unbounded, Unbounded)
    default: Option<f64>,
}

impl TypeString for FloatElementAttrs {
    const TYPE: &'static str = "float";
}

#[derive(Debug, PartialEq)]
struct DateElementAttrs {
    range: Range<i64>, // defaults to (Unbounded, Unbounded)
    default: Option<i64>,
}

impl TypeString for DateElementAttrs {
    const TYPE: &'static str = "date";
}

#[derive(Debug, PartialEq)]
struct StringElementAttrs {
    default: Option<String>,
}

impl TypeString for StringElementAttrs {
    const TYPE: &'static str = "string";
}

#[derive(Debug, PartialEq)]
struct Utf8ElementAttrs {
    default: Option<String>,
}

impl TypeString for Utf8ElementAttrs {
    const TYPE: &'static str = "utf-8";
}

#[derive(Debug, PartialEq)]
struct BinaryElementAttrs {
    default: Option<Vec<u8>>,
}

impl TypeString for BinaryElementAttrs {
    const TYPE: &'static str = "binary";
}

#[derive(Debug, PartialEq)]
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

            // For now, ignore anything other than element tags
            if depth != 2 {
                return None;
            }

            assert_eq!(name.local_name, "element".to_string());
            //assert_eq!(name.namespace, None);
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
                id: u32::from_str_radix(
                    attributes
                        .remove("id")
                        .expect("'id' attribute is required")
                        .trim_start_matches("0x"),
                    16,
                )
                .unwrap(),
                min_occurs: attributes
                    .remove("minOccurs")
                    .map(|v| v.parse().unwrap())
                    .unwrap_or_default(),
                max_occurs: attributes.remove("maxOccurs").map(|v| v.parse().unwrap()),
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
                            .map(|v| v.parse::<u32>().unwrap() != 0)
                            .unwrap_or_default(),
                        recursive: attributes
                            .remove("recursive")
                            .map(|v| v.parse::<u32>().unwrap() != 0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schema() {
        let stream = b"
<EBMLSchema xmlns=\"urn:ietf:rfc:8794\" docType=\"matroska\" version=\"4\">
  <!-- constraints on EBML Header Elements -->
  <element name=\"EBMLMaxIDLength\" path=\"\\EBML\\EBMLMaxIDLength\" id=\"0x42F2\" type=\"uinteger\" default=\"4\" minOccurs=\"1\" maxOccurs=\"1\"/>
  <element name=\"EBMLMaxSizeLength\" path=\"\\EBML\\EBMLMaxSizeLength\" id=\"0x42F3\" type=\"uinteger\" default=\"8\" minOccurs=\"1\" maxOccurs=\"1\"/>
  <!-- Root Element-->
  <element name=\"Segment\" path=\"\\Segment\" id=\"0x18538067\" type=\"master\" minOccurs=\"1\" maxOccurs=\"1\" unknownsizeallowed=\"1\">
    <documentation lang=\"en\" purpose=\"definition\">The Root Element that contains all other Top-Level Elements (Elements defined only at Level 1).
A Matroska file is composed of 1 Segment.</documentation>
  </element>
  <element name=\"SeekHead\" path=\"\\Segment\\SeekHead\" id=\"0x114D9B74\" type=\"master\" maxOccurs=\"2\">
    <documentation lang=\"en\" purpose=\"definition\">Contains the Segment Position of other Top-Level Elements.</documentation>
  </element>
</EBMLSchema>
";
        let element_descrs = parse_schema(&stream[..]).collect::<Vec<_>>();
        assert_eq!(element_descrs.len(), 4);
        assert_eq!(
            element_descrs[0],
            ElementDescriptor::UInt(
                ElementAttrs {
                    name: "EBMLMaxIDLength".to_string(),
                    path: "\\EBML\\EBMLMaxIDLength".to_string(),
                    id: 0x42F2,
                    min_occurs: 1,
                    max_occurs: Some(1),
                },
                UIntElementAttrs {
                    range: Range::IsWithin(Bound::Unbounded, Bound::Unbounded),
                    default: Some(4)
                }
            )
        );
        assert_eq!(
            element_descrs[1],
            ElementDescriptor::UInt(
                ElementAttrs {
                    name: "EBMLMaxSizeLength".to_string(),
                    path: "\\EBML\\EBMLMaxSizeLength".to_string(),
                    id: 0x42F3,
                    min_occurs: 1,
                    max_occurs: Some(1),
                },
                UIntElementAttrs {
                    range: Range::IsWithin(Bound::Unbounded, Bound::Unbounded),
                    default: Some(8)
                }
            )
        );
        assert_eq!(
            element_descrs[2],
            ElementDescriptor::Master(
                ElementAttrs {
                    name: "Segment".to_string(),
                    path: "\\Segment".to_string(),
                    id: 0x18538067,
                    min_occurs: 1,
                    max_occurs: Some(1),
                },
                MasterElementAttrs {
                    unknown_size_allowed: true,
                    recursive: false,
                }
            )
        );
        assert_eq!(
            element_descrs[3],
            ElementDescriptor::Master(
                ElementAttrs {
                    name: "SeekHead".to_string(),
                    path: "\\Segment\\SeekHead".to_string(),
                    id: 0x114D9B74,
                    min_occurs: 0,
                    max_occurs: Some(2),
                },
                MasterElementAttrs {
                    unknown_size_allowed: false,
                    recursive: false,
                }
            )
        );
    }
}
