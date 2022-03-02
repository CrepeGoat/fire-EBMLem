use serde_derive::{Deserialize, Serialize};
pub(crate) use serde_xml_rs::{from_str, to_string};

// documentation, element, enum, extension, implementation_note, restriction, EBMLSchema

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "EBMLSchema")]
struct EbmlSchema {
    doctype: String,
    version: u32,
    ebml: u32,
    #[serde(rename = "$value")]
    elements: Vec<Element>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Element {
    name: String,
    path: String,
    id: u32,
    min_occurs: Option<usize>,
    max_occurs: Option<usize>,
    range: Option<String>, // numeric elements only
    length: Option<String>,
    default: Option<String>, // non-master elements only
    r#type: String,
    unknownsizeallowed: Option<bool>, // master elements only
    recursive: Option<bool>,          // master elements only
    recurring: Option<bool>,
    minver: Option<u32>,
    maxver: Option<u32>,

    #[serde(rename = "$value")]
    metadata: Vec<ElementMetadatum>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ElementMetadatum {
    Documentation(Documentation),
    Extension(Extension),
    ImplementationNote(ImplementationNote),
    Restriction(Restriction),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Documentation {
    lang: Option<String>,
    purpose: DocumentationPurpose,
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "purpose")]
enum DocumentationPurpose {
    #[serde(rename = "definition")]
    Definition,
    #[serde(rename = "rationale")]
    Rationale,
    #[serde(rename = "usage notes")]
    UsageNotes,
    #[serde(rename = "references")]
    References,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ImplementationNote {
    note_attribute: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Restriction {
    #[serde(rename = "$value")]
    enums: Vec<Enum>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Enum {
    label: String,
    value: u32,
    #[serde(rename = "$value")]
    docs: Vec<Documentation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Extension {
    r#type: String,
    webm: Option<bool>,
    keep: Option<bool>,
    cppname: Option<String>,
}
