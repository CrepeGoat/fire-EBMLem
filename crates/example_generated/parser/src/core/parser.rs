
            use crate::base::element_defs::ElementDef;
            #[allow(unused_imports)]
            use crate::base::parser::{
                BoundTo, ElementReader, ElementState, IntoReader, NextStateNavigation, ReaderError,
                SkipStateNavigation, StateDataParser, StateError,
            };
            #[allow(unused_imports)]
            use crate::base::stream::{parse, serialize, stream_diff};
            use crate::core::element_defs;
            #[allow(unused_imports)]
            use crate::{
                impl_from_readers_for_states, impl_from_subreaders_for_readers, impl_from_substates_for_states,
                impl_into_reader, impl_next_state_navigation, impl_skip_state_navigation,
            };

            use enum_dispatch::enum_dispatch;

            use core::convert::{From, TryInto};
            use core::marker::PhantomData;
            use std::io::BufRead;

            // Top-Level Reader/State Enums #########################################################################
            
                #[enum_dispatch(FilesNextStates)]
                #[enum_dispatch(FilesNextReaders<R>)]
                
                #[enum_dispatch(FileNextStates)]
                #[enum_dispatch(FileNextReaders<R>)]
                
                #[enum_dispatch(_DocumentNextStates)]
                #[enum_dispatch(_DocumentNextReaders<R>)]
                
                #[enum_dispatch(VoidPrevStates)]
                #[enum_dispatch(VoidPrevReaders<R>)]
                
            #[enum_dispatch(States)]
            #[enum_dispatch(Readers<R>)]
            trait BlankTrait {}
            
            #[enum_dispatch]
            pub enum States {
                Void(VoidState),Files(FilesState),FileName(FileNameState),MimeType(MimeTypeState),ModificationTimestamp(ModificationTimestampState),Data(DataState),File(FileState),_Document(_DocumentState),
            }
            
            #[enum_dispatch]
            pub enum Readers<R> {
                Void(VoidReader<R>),Files(FilesReader<R>),FileName(FileNameReader<R>),MimeType(MimeTypeReader<R>),ModificationTimestamp(ModificationTimestampReader<R>),Data(DataReader<R>),File(FileReader<R>),_Document(_DocumentReader<R>),
            }
            
            
            impl_into_reader!(
                States,
                Readers,
                [Void, Files, FileName, MimeType, ModificationTimestamp, Data, File, _Document]
            );
            
            impl_from_readers_for_states!(
                Readers,
                States,
                [Void, Files, FileName, MimeType, ModificationTimestamp, Data, File, _Document]
            );
            
            
            // _Document Objects #########################################################################

            #[derive(Debug, Clone, PartialEq)]
            pub struct _DocumentState;
            pub type _DocumentReader<R> = ElementReader<R, _DocumentState>;

            impl<R: BufRead> _DocumentReader<R> {
                pub fn new(reader: R) -> Self {
                    Self {
                        reader,
                        state: _DocumentState,
                    }
                }
            }

            impl<R: BufRead> IntoReader<R> for _DocumentState {
                type Reader = _DocumentReader<R>;
                fn into_reader(self, reader: R) -> _DocumentReader<R> {
                    _DocumentReader::new(reader)
                }
            }

            impl_next_state_navigation!(
                _DocumentState,
                _DocumentNextStates,
                [(Files, FilesState), (Void, VoidState)]
            );
            
            #[derive(Debug, Clone, PartialEq)]
            #[enum_dispatch]
            pub enum _DocumentNextStates {
                Files(FilesState),Void(VoidState),
            }

            #[derive(Debug, PartialEq)]
            #[enum_dispatch]
            pub enum _DocumentNextReaders<R> {
                Files(FilesReader<R>),Void(VoidReader<R>),
            }

            impl_from_substates_for_states!(_DocumentNextStates, States, [Files, Void]);
            impl_from_subreaders_for_readers!(_DocumentNextReaders, Readers, [Files, Void]);

            impl_into_reader!(_DocumentNextStates, _DocumentNextReaders, [Files, Void]);
            impl_from_readers_for_states!(_DocumentNextReaders, _DocumentNextStates, [Files, Void]);
            
                // Void Objects #########################################################################

                pub type VoidState = ElementState<element_defs::VoidDef, VoidPrevStates>;
                pub type VoidReader<R> = ElementReader<R, VoidState>;

                impl VoidState {
                    pub fn new(bytes_left: usize, parent_state: VoidPrevStates) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> VoidReader<R> {
                    pub fn new(reader: R, state: VoidState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(VoidState, VoidPrevStates);
                impl_next_state_navigation!(VoidState, VoidPrevStates, []);
                
                    #[derive(Debug, Clone, PartialEq)]
                    #[enum_dispatch]
                    pub enum VoidPrevStates {
                        _Document(_DocumentState),File(FileState),Files(FilesState),
                    }
                    #[derive(Debug, PartialEq)]
                    #[enum_dispatch]
                    pub enum VoidPrevReaders<R> {
                        _Document(_DocumentReader<R>),File(FileReader<R>),Files(FilesReader<R>),
                    }

                    impl_from_substates_for_states!(VoidPrevStates, States, [_Document, Files, File]);
                    impl_from_subreaders_for_readers!(VoidPrevReaders, Readers, [_Document, Files, File]);

                    impl_into_reader!(VoidPrevStates, VoidPrevReaders, [_Document, Files, File]);
                    impl_from_readers_for_states!(VoidPrevReaders, VoidPrevStates, [_Document, Files, File]);

                    
                // Files Objects #########################################################################

                pub type FilesState = ElementState<element_defs::FilesDef, _DocumentState>;
                pub type FilesReader<R> = ElementReader<R, FilesState>;

                impl FilesState {
                    pub fn new(bytes_left: usize, parent_state: _DocumentState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> FilesReader<R> {
                    pub fn new(reader: R, state: FilesState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(FilesState, _DocumentState);
                impl_next_state_navigation!(FilesState, FilesNextStates, [(Void, VoidState), (File, FileState)]);
                
                    #[derive(Debug, Clone, PartialEq)]
                    #[enum_dispatch]
                    pub enum FilesNextStates {
                        Void(VoidState),File(FileState),
                        Parent(_DocumentState),
                    }

                    #[derive(Debug, PartialEq)]
                    #[enum_dispatch]
                    pub enum FilesNextReaders<R> {
                        Void(VoidReader<R>),File(FileReader<R>),
                        Parent(_DocumentReader<R>),
                    }

                    impl_from_substates_for_states!(FilesNextStates, States, [Void, File, Parent]);
                    impl_from_subreaders_for_readers!(FilesNextReaders, Readers, [Void, File, Parent]);

                    impl_into_reader!(FilesNextStates, FilesNextReaders, [Void, File, Parent]);
                    impl_from_readers_for_states!(FilesNextReaders, FilesNextStates, [Void, File, Parent]);
                    
                // FileName Objects #########################################################################

                pub type FileNameState = ElementState<element_defs::FileNameDef, FileState>;
                pub type FileNameReader<R> = ElementReader<R, FileNameState>;

                impl FileNameState {
                    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> FileNameReader<R> {
                    pub fn new(reader: R, state: FileNameState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(FileNameState, FileState);
                impl_next_state_navigation!(FileNameState, FileState, []);
                
                // MimeType Objects #########################################################################

                pub type MimeTypeState = ElementState<element_defs::MimeTypeDef, FileState>;
                pub type MimeTypeReader<R> = ElementReader<R, MimeTypeState>;

                impl MimeTypeState {
                    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> MimeTypeReader<R> {
                    pub fn new(reader: R, state: MimeTypeState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(MimeTypeState, FileState);
                impl_next_state_navigation!(MimeTypeState, FileState, []);
                
                // ModificationTimestamp Objects #########################################################################

                pub type ModificationTimestampState = ElementState<element_defs::ModificationTimestampDef, FileState>;
                pub type ModificationTimestampReader<R> = ElementReader<R, ModificationTimestampState>;

                impl ModificationTimestampState {
                    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> ModificationTimestampReader<R> {
                    pub fn new(reader: R, state: ModificationTimestampState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(ModificationTimestampState, FileState);
                impl_next_state_navigation!(ModificationTimestampState, FileState, []);
                
                // Data Objects #########################################################################

                pub type DataState = ElementState<element_defs::DataDef, FileState>;
                pub type DataReader<R> = ElementReader<R, DataState>;

                impl DataState {
                    pub fn new(bytes_left: usize, parent_state: FileState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> DataReader<R> {
                    pub fn new(reader: R, state: DataState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(DataState, FileState);
                impl_next_state_navigation!(DataState, FileState, []);
                
                // File Objects #########################################################################

                pub type FileState = ElementState<element_defs::FileDef, FilesState>;
                pub type FileReader<R> = ElementReader<R, FileState>;

                impl FileState {
                    pub fn new(bytes_left: usize, parent_state: FilesState) -> Self {
                        Self {
                            bytes_left,
                            parent_state,
                            _phantom: PhantomData::<_>,
                        }
                    }
                }

                impl<R: BufRead> FileReader<R> {
                    pub fn new(reader: R, state: FileState) -> Self {
                        Self { reader, state }
                    }
                }

                impl_skip_state_navigation!(FileState, FilesState);
                impl_next_state_navigation!(FileState, FileNextStates, [(Void, VoidState), (Data, DataState), (FileName, FileNameState), (ModificationTimestamp, ModificationTimestampState), (MimeType, MimeTypeState)]);
                
                    #[derive(Debug, Clone, PartialEq)]
                    #[enum_dispatch]
                    pub enum FileNextStates {
                        Void(VoidState),Data(DataState),FileName(FileNameState),ModificationTimestamp(ModificationTimestampState),MimeType(MimeTypeState),
                        Parent(FilesState),
                    }

                    #[derive(Debug, PartialEq)]
                    #[enum_dispatch]
                    pub enum FileNextReaders<R> {
                        Void(VoidReader<R>),Data(DataReader<R>),FileName(FileNameReader<R>),ModificationTimestamp(ModificationTimestampReader<R>),MimeType(MimeTypeReader<R>),
                        Parent(FilesReader<R>),
                    }

                    impl_from_substates_for_states!(FileNextStates, States, [Void, Data, FileName, ModificationTimestamp, MimeType, Parent]);
                    impl_from_subreaders_for_readers!(FileNextReaders, Readers, [Void, Data, FileName, ModificationTimestamp, MimeType, Parent]);

                    impl_into_reader!(FileNextStates, FileNextReaders, [Void, Data, FileName, ModificationTimestamp, MimeType, Parent]);
                    impl_from_readers_for_states!(FileNextReaders, FileNextStates, [Void, Data, FileName, ModificationTimestamp, MimeType, Parent]);
                    