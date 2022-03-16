use serde_derive::{Deserialize, Serialize};
pub use serde_xml_rs::{from_reader, from_str, to_string};

// documentation, element, enum, extension, implementation_note, restriction, EBMLSchema

pub mod custom_serde {
    pub mod hexadecimal {
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

        // This deserializer was originally written with u64 in mind. Then it was made generic by
        // changing u64 to T everywhere and adding boundaries. Same with the serializer.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
        where
            D: Deserializer<'de>,
        {
            u32::from_str_radix(
                String::deserialize(deserializer)?
                    .strip_prefix("0x")
                    .ok_or_else(|| D::Error::custom("missing hexadecimal prefix '0x'"))?,
                16,
            )
            .map_err(|e| D::Error::custom(format!("{}", e)))
        }

        pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("{:#X}", value).serialize(serializer)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "EBMLSchema")]
#[serde(rename_all = "camelCase")]
pub struct EbmlSchema {
    pub doc_type: String,
    pub version: u32,
    pub ebml: Option<u32>,
    #[serde(rename = "$value")]
    pub elements: Option<Vec<Element>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub name: String,
    pub path: String,
    #[serde(with = "custom_serde::hexadecimal")]
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
    pub metadata: Option<Vec<ElementValue>>,
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
    pub docs: Option<Vec<Documentation>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Extension {
    pub r#type: String,
    pub webm: Option<bool>,
    pub keep: Option<bool>,
    pub cppname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        r#"
        <EBMLSchema xmlns="urn:ietf:rfc:8794" docType="files-in-ebml-demo" version="1"/>
        "#,
        EbmlSchema {
            doc_type: "files-in-ebml-demo".to_string(),
            version: 1,
            ebml: None,
            elements: None,
        },
    )]
    fn test_deserialize_ebmlschema(#[case] source: &str, #[case] expt_result: EbmlSchema) {
        let result: EbmlSchema = from_str(source).unwrap();
        assert_eq!(result, expt_result);
    }

    #[rstest]
    #[case(
        r#"<element name="EBML" path="\EBML" id="0x1A45DFA3" type="master" minOccurs="1" maxOccurs="1"/>"#,
        Element {
            name: "EBML".to_string(),
            path: "\\EBML".to_string(),
            id: 0x1A45DFA3,
            min_occurs: Some(1),
            max_occurs: Some(1),
            range: None,
            length: None,
            default: None,
            r#type: ElementType::Master,
            unknownsizeallowed: None,
            recursive: None,
            recurring: None,
            minver: None,
            maxver: None,
            metadata: None,
        },
    )]
    fn test_deserialize_element(#[case] source: &str, #[case] expt_result: Element) {
        let result: Element = from_str(source).unwrap();
        assert_eq!(result, expt_result);
    }

    #[rstest]
    #[case(r#"<master/>"#, ElementType::Master)]
    #[case(r#"<integer/>"#, ElementType::SignedInteger)]
    #[case(r#"<uinteger/>"#, ElementType::UnsignedInteger)]
    #[case(r#"<float/>"#, ElementType::Float)]
    #[case(r#"<string/>"#, ElementType::String)]
    #[case(r#"<date/>"#, ElementType::Date)]
    #[case(r#"<utf-8/>"#, ElementType::Utf8)]
    #[case(r#"<binary/>"#, ElementType::Binary)]
    fn test_deserialize_element_type(#[case] source: &str, #[case] expt_result: ElementType) {
        let result: ElementType = from_str(source).unwrap();
        assert_eq!(result, expt_result);
    }
}
