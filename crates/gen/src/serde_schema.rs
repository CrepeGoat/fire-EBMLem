use serde_derive::{Deserialize, Serialize};
pub use serde_xml_rs::{from_reader, from_str, to_string};

// documentation, element, enum, extension, implementation_note, restriction, EBMLSchema

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "EBMLSchema")]
pub struct EbmlSchema {
    pub doctype: String,
    pub version: u32,
    pub ebml: u32,
    #[serde(rename = "$value")]
    pub elements: Vec<Element>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub name: String,
    pub path: String,
    pub id: u32,
    pub min_occurs: Option<usize>,
    pub max_occurs: Option<usize>,
    pub range: Option<String>, // numeric elements only
    pub length: Option<String>,
    pub default: Option<String>, // non-master elements only
    pub r#type: ElementType,
    pub unknownsizeallowed: Option<bool>, // master elements only
    pub recursive: Option<bool>,          // master elements only
    pub recurring: Option<bool>,
    pub minver: Option<u32>,
    pub maxver: Option<u32>,

    #[serde(rename = "$value")]
    pub metadata: Vec<ElementValue>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "type")]
pub enum ElementType {
    #[serde(rename = "integer")]
    SignedInteger,
    #[serde(rename = "uinteger")]
    UnsignedInteger,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "master")]
    Master,
    #[serde(rename = "binary")]
    Binary,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ElementValue {
    Documentation(Documentation),
    Extension(Extension),
    ImplementationNote(ImplementationNote),
    Restriction(Restriction),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Documentation {
    pub lang: Option<String>,
    pub purpose: DocumentationPurpose,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "purpose")]
pub enum DocumentationPurpose {
    #[serde(rename = "definition")]
    Definition,
    #[serde(rename = "rationale")]
    Rationale,
    #[serde(rename = "usage notes")]
    UsageNotes,
    #[serde(rename = "references")]
    References,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ImplementationNote {
    pub note_attribute: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Restriction {
    #[serde(rename = "$value")]
    pub enums: Vec<Enum>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Enum {
    pub label: String,
    pub value: u32,
    #[serde(rename = "$value")]
    pub docs: Vec<Documentation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Extension {
    pub r#type: String,
    pub webm: Option<bool>,
    pub keep: Option<bool>,
    pub cppname: Option<String>,
}
