use std::convert::{From, TryInto};

use crate::schema_types::Bound;
use crate::schema_types::{
    BinaryElementDef, DateElementDef, ElementDef, FloatElementDef, IntElementDef, MasterElementDef,
    Range, StringElementDef, UIntElementDef, UTF8ElementDef,
};
use crate::stream::{parse, stream_diff};

// parent: (None)
#[derive(Debug, Clone, PartialEq)]
struct FilesDef;

impl ElementDef for FilesDef {
    const ID: u32 = 0x1946696C;

    type LastParent = ();
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 0;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl MasterElementDef for FilesDef {
    const UNKNOWN_SIZE_ALLOWED: bool = False;
    const RECURSIVE: bool = False;
}

// parent: Files
#[derive(Debug, Clone, PartialEq)]
struct FileDef;

impl Element for FileDef {
    const ID: u32 = 0x6146;

    type LastParent = FilesDef;
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 1;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl MasterElement for FileDef {
    const UNKNOWN_SIZE_ALLOWED: bool = False;
    const RECURSIVE: bool = False;
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct FileNameDef;

impl Element for FileNameDef {
    const ID: u32 = 0x614E;

    type LastParent = FileDef;
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 1;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl UTF8Element for FileNameDef {
    const DEFAULT: Option<&'static str> = None;
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct MimeTypeDef;

impl Element for MimeTypeDef {
    const ID: u32 = 0x464D;

    type LastParent = FileDef;
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 1;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl StringElement for MimeTypeDef {
    const DEFAULT: Option<&'static str> = None;
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct ModificationTimestampDef;

impl Element for ModificationTimestampDef {
    const ID: u32 = 0x4654;

    type LastParent = FileDef;
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 1;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsExactly(8);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl DateElement for ModificationTimestampDef {
    const RANGE: Option<RangeDef<i64>> = None;
    const DEFAULT: Option<i64> = None;
}

// parent: File
#[derive(Debug, Clone, PartialEq)]
struct DataDef;

impl Element for DataDef {
    const ID: u32 = 0x4664;

    type LastParent = FileDef;
    const GLOBAL_PARENT_OCCURENCE: (usize, usize) = (0, 0);

    const MIN_OCCURS: usize = 1;
    const MAX_OCCURS: usize = usize::MAX;
    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
    const RECURRING: bool = false;
    const MIN_VERSION: u64 = 1;
    const MAX_VERSION: u64 = 1;
}

impl BinaryElement for DataDef {
    const DEFAULT: Option<&'static [u8]> = None;
}
