use crate::element_defs::{
    BinaryElementDef, DateElementDef, ElementDef, FloatElementDef, IntElementDef, MasterElementDef,
    Range, StringElementDef, UIntElementDef, UTF8ElementDef,
};
use crate::element_defs::{Bound, ParentOf};

// parent: (None)
#[derive(Debug, Clone, PartialEq)]
pub struct FilesDef;

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
    const UNKNOWN_SIZE_ALLOWED: bool = false;
    const RECURSIVE: bool = false;
}

impl ParentOf<FileDef> for () {}

// parent: Files
#[derive(Debug, Clone, PartialEq)]
pub struct FileDef;

impl ElementDef for FileDef {
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

impl MasterElementDef for FileDef {
    const UNKNOWN_SIZE_ALLOWED: bool = false;
    const RECURSIVE: bool = false;
}

impl ParentOf<FileDef> for FilesDef {}

// parent: File
#[derive(Debug, Clone, PartialEq)]
pub struct FileNameDef;

impl ElementDef for FileNameDef {
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

impl UTF8ElementDef for FileNameDef {
    const DEFAULT: Option<&'static str> = None;
}

impl ParentOf<FileNameDef> for FileDef {}

// parent: File
#[derive(Debug, Clone, PartialEq)]
pub struct MimeTypeDef;

impl ElementDef for MimeTypeDef {
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

impl StringElementDef for MimeTypeDef {
    const DEFAULT: Option<&'static str> = None;
}

impl ParentOf<MimeTypeDef> for FileDef {}

// parent: File
#[derive(Debug, Clone, PartialEq)]
pub struct ModificationTimestampDef;

impl ElementDef for ModificationTimestampDef {
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

impl DateElementDef for ModificationTimestampDef {
    const RANGE: Range<i64> =
        Range::IsWithin(core::ops::Bound::Unbounded, core::ops::Bound::Unbounded);
    const DEFAULT: Option<i64> = None;
}

impl ParentOf<ModificationTimestampDef> for FileDef {}

// parent: File
#[derive(Debug, Clone, PartialEq)]
pub struct DataDef;

impl ElementDef for DataDef {
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

impl BinaryElementDef for DataDef {
    const DEFAULT: Option<&'static [u8]> = None;
}

impl ParentOf<DataDef> for FileDef {}
