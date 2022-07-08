
            #[allow(unused_imports)]
            use crate::base::element_defs::{
                BinaryElementDef, DateElementDef, ElementDef, FloatElementDef, IntElementDef, MasterElementDef,
                Range, StringElementDef, UIntElementDef, Utf8ElementDef,
            };

            use core::ops::Bound;
            
                #[derive(Debug, Clone, PartialEq)]
                pub struct VoidDef;

                impl ElementDef for VoidDef {
                    const ID: u32 = 236;
                    const PATH: &'static str = r"\(-\)Void";

                    const MIN_OCCURS: usize = 0;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl BinaryElementDef for VoidDef {
                        const DEFAULT: Option<&'static [u8]> = None;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct MimeTypeDef;

                impl ElementDef for MimeTypeDef {
                    const ID: u32 = 17997;
                    const PATH: &'static str = r"\Files\File\MimeType";

                    const MIN_OCCURS: usize = 1;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl StringElementDef for MimeTypeDef {
                        const DEFAULT: Option<&'static str> = None;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct ModificationTimestampDef;

                impl ElementDef for ModificationTimestampDef {
                    const ID: u32 = 18004;
                    const PATH: &'static str = r"\Files\File\ModificationTimestamp";

                    const MIN_OCCURS: usize = 1;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl DateElementDef for ModificationTimestampDef {
                        const RANGE: Range<i64> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                        const DEFAULT: Option<i64> = None;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct DataDef;

                impl ElementDef for DataDef {
                    const ID: u32 = 18020;
                    const PATH: &'static str = r"\Files\File\Data";

                    const MIN_OCCURS: usize = 1;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl BinaryElementDef for DataDef {
                        const DEFAULT: Option<&'static [u8]> = None;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct FileDef;

                impl ElementDef for FileDef {
                    const ID: u32 = 24902;
                    const PATH: &'static str = r"\Files\File";

                    const MIN_OCCURS: usize = 1;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl MasterElementDef for FileDef {
                        const UNKNOWN_SIZE_ALLOWED: bool = false;
                        const RECURSIVE: bool = false;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct FileNameDef;

                impl ElementDef for FileNameDef {
                    const ID: u32 = 24910;
                    const PATH: &'static str = r"\Files\File\FileName";

                    const MIN_OCCURS: usize = 1;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl Utf8ElementDef for FileNameDef {
                        const DEFAULT: Option<&'static str> = None;
                    }
                    
                #[derive(Debug, Clone, PartialEq)]
                pub struct FilesDef;

                impl ElementDef for FilesDef {
                    const ID: u32 = 424044908;
                    const PATH: &'static str = r"\Files";

                    const MIN_OCCURS: usize = 0;
                    const MAX_OCCURS: Option<usize> = None;
                    const LENGTH: Range<usize> = Range::IsWithin(Bound::Unbounded, Bound::Unbounded);
                    const RECURRING: bool = false;
                    const MIN_VERSION: u64 = 1;
                    const MAX_VERSION: Option<u64> = None;
                }
                
                    impl MasterElementDef for FilesDef {
                        const UNKNOWN_SIZE_ALLOWED: bool = false;
                        const RECURSIVE: bool = false;
                    }
                    