/*
Example schema (https://github.com/ietf-wg-cellar/ebml-specification/blob/master/ebml_schema_example.xml):

<?xml version="1.0" encoding="utf-8"?>
<EBMLSchema xmlns="urn:ietf:rfc:8794"
  docType="files-in-ebml-demo" version="1">
 <!-- constraints to the range of two EBML Header Elements -->
 <element name="EBMLReadVersion" path="\EBML\EBMLReadVersion"
   id="0x42F7" minOccurs="1" maxOccurs="1" range="1" default="1"
   type="uinteger"/>
 <element name="EBMLMaxSizeLength"
   path="\EBML\EBMLMaxSizeLength" id="0x42F3" minOccurs="1"
   maxOccurs="1" range="8" default="8" type="uinteger"/>
 <!-- Root Element-->
 <element name="Files" path="\Files" id="0x1946696C"
   type="master">
  <documentation lang="en"
    purpose="definition">Container of data and
  attributes representing one or many files.</documentation>
 </element>
 <element name="File" path="\Files\File" id="0x6146"
   type="master" minOccurs="1">
  <documentation lang="en" purpose="definition">
    An attached file.
  </documentation>
 </element>
 <element name="FileName" path="\Files\File\FileName"
   id="0x614E" type="utf-8"
   minOccurs="1">
  <documentation lang="en" purpose="definition">
    Filename of the attached file.
  </documentation>
 </element>
 <element name="MimeType" path="\Files\File\MimeType"
   id="0x464D" type="string"
     minOccurs="1">
  <documentation lang="en" purpose="definition">
    MIME type of the file.
  </documentation>
 </element>
 <element name="ModificationTimestamp"
   path="\Files\File\ModificationTimestamp" id="0x4654"
   type="date" minOccurs="1">
  <documentation lang="en" purpose="definition">
    Modification timestamp of the file.
  </documentation>
 </element>
 <element name="Data" path="\Files\File\Data" id="0x4664"
   type="binary" minOccurs="1">
  <documentation lang="en" purpose="definition">
    The data of the file.
  </documentation>
 </element>
</EBMLSchema>
*/

use std::convert::{From, TryInto};

use crate::schema_types::Bound;
use crate::schema_types::{
    BinaryElement, DateElement, Element, FloatElement, IntElement, MasterElement, RangeDef,
    StringElement, UIntElement, UTF8Element,
};
use crate::stream::{parse, stream_diff};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Implicit Items

// parent: None
#[derive(Debug, Clone, PartialEq)]
struct Document {
    bytes_left: usize,
}

impl Element for Document {
    const ID: u32 = 0; // no ID for this element

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for Document {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = Some(true);
    const RECURSIVE: Option<bool> = None;
}

// parent: None
#[derive(Debug, Clone, PartialEq)]
struct EBML {
    bytes_left: usize,
    parent: Document,
}

impl Element for EBML {
    const ID: u32 = 0x1A45DFA3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for EBML {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;
}

impl From<EBML> for Elements {
    fn from(element: EBML) -> Elements {
        Elements::EBML(element)
    }
}

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLVersion {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLVersion {
    const ID: u32 = 0x4286;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

impl From<EBMLVersion> for Elements {
    fn from(element: EBMLVersion) -> Elements {
        Elements::EBMLVersion(element)
    }
}

/*
// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLReadVersion {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLReadVersion {
    const ID: u32 = 0x42F7;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(1));
    const DEFAULT: Option<u64> = Some(1);
}

impl From<EBMLReadVersion> for Elements {
    fn from(element: EBMLReadVersion) -> Elements {
        Elements::EBMLReadVersion(element)
    }
}

*/

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLMaxIDLength {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLMaxIDLength {
    const ID: u32 = 0x42F2;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxIDLength {
    const RANGE: Option<RangeDef<u64>> =
        Some(RangeDef::IsWithin(Bound::Included(4), Bound::Unbounded));
    const DEFAULT: Option<u64> = Some(4);
}

impl From<EBMLMaxIDLength> for Elements {
    fn from(element: EBMLMaxIDLength) -> Elements {
        Elements::EBMLMaxIDLength(element)
    }
}

/*
// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLMaxSizeLength {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLMaxSizeLength {
    const ID: u32 = 0x42F3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxSizeLength {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(8);
}

impl From<EBMLMaxSizeLength> for Elements {
    fn from(element: EBMLMaxSizeLength) -> Elements {
        Elements::EBMLMaxSizeLength(element)
    }
}
*/

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct DocType {
    bytes_left: usize,
    parent: EBML,
}

impl Element for DocType {
    const ID: u32 = 0x4282;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> =
        Some(RangeDef::IsWithin(Bound::Excluded(0), Bound::Unbounded));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for DocType {
    const DEFAULT: Option<&'static str> = None;
}

impl From<DocType> for Elements {
    fn from(element: DocType) -> Elements {
        Elements::DocType(element)
    }
}

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct DocTypeVersion {
    bytes_left: usize,
    parent: EBML,
}

impl Element for DocTypeVersion {
    const ID: u32 = 0x4287;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

impl From<DocTypeVersion> for Elements {
    fn from(element: DocTypeVersion) -> Elements {
        Elements::DocTypeVersion(element)
    }
}

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct DocTypeReadVersion {
    bytes_left: usize,
    parent: EBML,
}

impl Element for DocTypeReadVersion {
    const ID: u32 = 0x4285;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

impl From<DocTypeReadVersion> for Elements {
    fn from(element: DocTypeReadVersion) -> Elements {
        Elements::DocTypeReadVersion(element)
    }
}

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct DocTypeExtension {
    bytes_left: usize,
    parent: EBML,
}

impl Element for DocTypeExtension {
    const ID: u32 = 0x4281;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for DocTypeExtension {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;
}

impl From<DocTypeExtension> for Elements {
    fn from(element: DocTypeExtension) -> Elements {
        Elements::DocTypeExtension(element)
    }
}

// parent: DocTypeExtension
#[derive(Debug, Clone, PartialEq)]
struct DocTypeExtensionName {
    bytes_left: usize,
    parent: DocTypeExtension,
}

impl Element for DocTypeExtensionName {
    const ID: u32 = 0x4283;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> =
        Some(RangeDef::IsWithin(Bound::Included(1), Bound::Unbounded));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for DocTypeExtensionName {
    const DEFAULT: Option<&'static str> = None;
}

impl From<DocTypeExtensionName> for Elements {
    fn from(element: DocTypeExtensionName) -> Elements {
        Elements::DocTypeExtensionName(element)
    }
}

// parent: DocTypeExtension
#[derive(Debug, Clone, PartialEq)]
struct DocTypeExtensionVersion {
    bytes_left: usize,
    parent: DocTypeExtension,
}

impl Element for DocTypeExtensionVersion {
    const ID: u32 = 0x4284;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeExtensionVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = None;
}

impl From<DocTypeExtensionVersion> for Elements {
    fn from(element: DocTypeExtensionVersion) -> Elements {
        Elements::DocTypeExtensionVersion(element)
    }
}

/*
#[derive(Debug, Clone, PartialEq)]
struct CRC32 {}

impl Element for CRC32 {
    const ID: u32 = 0xBF;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = Some(RangeDef::IsExactly(4));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for CRC32 {
    const DEFAULT: Option<&'static [u8]> = None;
}

impl From<CRC32> for Elements {
    fn from(element: CRC32) -> Elements {
        Elements::CRC32(element)
    }
}


#[derive(Debug, Clone, PartialEq)]
struct Void {}

impl Element for Void {
    const ID: u32 = 0xEC;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for Void {
    const DEFAULT: Option<&'static [u8]> = None;
}

impl From<Void> for Elements {
    fn from(element: Void) -> Elements {
        Elements::Void(element)
    }
}
*/

////////////////////////////////////////////////////////////////////////////////////////////////////
// Explicit Items

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLReadVersion {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLReadVersion {
    const ID: u32 = 0x42F7;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(1));
    const DEFAULT: Option<u64> = Some(1);
}

impl From<EBMLReadVersion> for Elements {
    fn from(element: EBMLReadVersion) -> Elements {
        Elements::EBMLReadVersion(element)
    }
}

// parent: EBML
#[derive(Debug, Clone, PartialEq)]
struct EBMLMaxSizeLength {
    bytes_left: usize,
    parent: EBML,
}

impl Element for EBMLMaxSizeLength {
    const ID: u32 = 0x42F3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxSizeLength {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(8));
    const DEFAULT: Option<u64> = Some(8);
}

impl From<EBMLMaxSizeLength> for Elements {
    fn from(element: EBMLMaxSizeLength) -> Elements {
        Elements::EBMLMaxSizeLength(element)
    }
}

// parent: (None)
#[derive(Debug, Clone, PartialEq)]
struct Files {
    bytes_left: usize,
    parent: Document,
}

impl Element for Files {
    const ID: u32 = 0x1946696C;

    const MIN_OCCURS: Option<usize> = None;
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for Files {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;
}

impl From<Files> for Elements {
    fn from(element: Files) -> Elements {
        Elements::Files(element)
    }
}

// parent: Files
#[derive(Debug, Clone, PartialEq)]
struct File {
    bytes_left: usize,
    parent: Files,
}

impl Element for File {
    const ID: u32 = 0x6146;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for File {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;
}

impl From<File> for Elements {
    fn from(element: File) -> Elements {
        Elements::File(element)
    }
}

impl File {
    fn next<'a>(mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], Elements, ()> {
        match self {
            Self {
                bytes_left: 0,
                parent: _,
            } => Ok((stream, self.parent.into())),
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
                        FileName::ID => FileName {
                            bytes_left: len,
                            parent: self,
                        }
                        .into(),
                        MimeType::ID => MimeType {
                            bytes_left: len,
                            parent: self,
                        }
                        .into(),
                        ModificationTimestamp::ID => ModificationTimestamp {
                            bytes_left: len,
                            parent: self,
                        }
                        .into(),
                        Data::ID => Data {
                            bytes_left: len,
                            parent: self,
                        }
                        .into(),
                        _ => return Err(nom::Err::Failure(())),
                    },
                ))
            }
        }
    }
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct FileName {
    bytes_left: usize,
    parent: File,
}

impl Element for FileName {
    const ID: u32 = 0x614E;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UTF8Element for FileName {
    const DEFAULT: Option<&'static str> = None;
}

impl From<FileName> for Elements {
    fn from(element: FileName) -> Elements {
        Elements::FileName(element)
    }
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct MimeType {
    bytes_left: usize,
    parent: File,
}

impl Element for MimeType {
    const ID: u32 = 0x464D;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for MimeType {
    const DEFAULT: Option<&'static str> = None;
}

impl From<MimeType> for Elements {
    fn from(element: MimeType) -> Elements {
        Elements::MimeType(element)
    }
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct ModificationTimestamp {
    bytes_left: usize,
    parent: File,
}

impl Element for ModificationTimestamp {
    const ID: u32 = 0x4654;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl DateElement for ModificationTimestamp {
    const RANGE: Option<RangeDef<i64>> = None;
    const DEFAULT: Option<i64> = None;
}

impl From<ModificationTimestamp> for Elements {
    fn from(element: ModificationTimestamp) -> Elements {
        Elements::ModificationTimestamp(element)
    }
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct Data {
    bytes_left: usize,
    parent: File,
}

impl Element for Data {
    const ID: u32 = 0x4664;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for Data {
    const DEFAULT: Option<&'static [u8]> = None;
}

impl From<Data> for Elements {
    fn from(element: Data) -> Elements {
        Elements::Data(element)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Elements {
    EBML(EBML),
    EBMLVersion(EBMLVersion),
    EBMLReadVersion(EBMLReadVersion),
    EBMLMaxIDLength(EBMLMaxIDLength),
    EBMLMaxSizeLength(EBMLMaxSizeLength),
    DocType(DocType),
    DocTypeVersion(DocTypeVersion),
    DocTypeReadVersion(DocTypeReadVersion),
    DocTypeExtension(DocTypeExtension),
    DocTypeExtensionName(DocTypeExtensionName),
    DocTypeExtensionVersion(DocTypeExtensionVersion),

    Files(Files),
    File(File),
    FileName(FileName),
    MimeType(MimeType),
    ModificationTimestamp(ModificationTimestamp),
    Data(Data),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest(element, source, expt_result,
        case(
            File{bytes_left: 5, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}},
            &[0x61, 0x4E, 0x82, 0xFF, 0xFF],
            (&[0xFF, 0xFF][..], FileName{bytes_left: 2, parent: File{bytes_left: 0, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}}}.into())
        ),
        case(
            File{bytes_left: 5, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}},
            &[0x46, 0x4D, 0x82, 0xFF, 0xFF],
            (&[0xFF, 0xFF][..], MimeType{bytes_left: 2, parent: File{bytes_left: 0, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}}}.into())
        ),
        case(
            File{bytes_left: 5, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}},
            &[0x46, 0x54, 0x82, 0xFF, 0xFF],
            (&[0xFF, 0xFF][..], ModificationTimestamp{bytes_left: 2, parent: File{bytes_left: 0, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}}}.into())
        ),
        case(
            File{bytes_left: 5, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}},
            &[0x46, 0x64, 0x82, 0xFF, 0xFF],
            (&[0xFF, 0xFF][..], Data{bytes_left: 2, parent: File{bytes_left: 0, parent: Files{bytes_left: 0, parent: Document{bytes_left: 0}}}}.into())
        ),
    )]
    fn test_file_next(
        element: File,
        source: &'static [u8],
        expt_result: (&'static [u8], Elements),
    ) {
        assert_eq!(element.next(source).unwrap(), expt_result);
    }
}
