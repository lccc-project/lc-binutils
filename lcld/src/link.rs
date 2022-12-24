use std::path::PathBuf;

use crate::input::InputFileType;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputFile {
    pub path: PathBuf,
    pub ty: InputFileType,
    pub as_needed: bool,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputId(u32);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SectionId(u32);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SectionOffset {
    Absolute(u128),
    Begin(u64),
    End(i64),
    AfterInput { input_spec_number: u32, pos: i64 },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolDef {
    Object(InputId),
    ArchiveMember(InputId, String),
    ScriptDefined(SectionId, SectionOffset),
    Undefined,
}
