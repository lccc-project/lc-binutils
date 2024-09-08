use std::{io, path::PathBuf};

use binfmt::{
    ar::Archive,
    fmt::{BinaryFile, Section},
};
use indexmap::IndexMap;

use crate::{input::InputFileType, script::ParsedScript};

pub enum LinkInput {
    Unopened(PathBuf),
    Object(BinaryFile<'static>),
    Archive(Archive),
    ParsedScript(ParsedScript),
    GroupStartMarker,
    Group(InputId),
}

pub struct InputFile {
    pub input: LinkInput,
    pub ty: InputFileType,
    pub as_needed: bool,
}

impl InputFile {
    pub fn open(&mut self) -> io::Result<()> {
        if let LinkInput::Unopened(path) = &self.input {
            let mut file = std::fs::File::open(path)?;
            match self.ty {
                InputFileType::Archive => self.input = LinkInput::Archive(Archive::read(file)?),
                InputFileType::LinkerScript => todo!("Parse Linker Script"),
                InputFileType::Object(fmt) => {
                    self.input = LinkInput::Object(fmt.read_file(&mut file)?.ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Failed to open: {} (detected as {fmt:?})", path.display()),
                        )
                    })?);
                }
                InputFileType::LtoInput(lto) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unrecognized LTO Format {lto:?}"),
                ))?,
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputId(u32);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SectionId(u32);

impl SectionId {
    /// Output /DISCARD/ section from linker scripts.
    /// Used to black hole an input section programmatically
    pub const DISCARD: Self = Self(!0);
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RegionId(u32);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GroupId(u32);

impl GroupId {
    pub const NO_GROUP: Self = Self(!0);
}

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

#[allow(dead_code)]
pub struct LinkState {
    inputs: IndexMap<InputId, InputFile>,
    symbol_defs: IndexMap<String, SymbolDef>,
    section_ids: IndexMap<String, SectionId>,
    sections: IndexMap<SectionId, Section>,
    next_input_id: u32,
    next_section_id: u32,
    group_head_stack: Vec<InputId>,
}

impl LinkState {
    pub fn new() -> LinkState {
        Self {
            inputs: IndexMap::new(),
            symbol_defs: IndexMap::new(),
            section_ids: IndexMap::new(),
            sections: IndexMap::new(),
            next_input_id: 0,
            next_section_id: 0,
            group_head_stack: Vec::new(),
        }
    }

    pub fn begin_group(&mut self) {
        let id = InputId(self.next_input_id);
        self.next_input_id += 1;

        self.inputs.insert(
            id,
            InputFile {
                input: LinkInput::GroupStartMarker,
                ty: InputFileType::LinkerScript,
                as_needed: false,
            },
        );

        self.group_head_stack.push(id);
    }

    pub fn end_group(&mut self) {
        let id = InputId(self.next_input_id);
        self.next_input_id += 1;

        self.inputs.insert(
            id,
            InputFile {
                input: LinkInput::Group(
                    self.group_head_stack
                        .pop()
                        .expect("begin_group and end_group calls must be balanced"),
                ),
                ty: InputFileType::LinkerScript,
                as_needed: false,
            },
        );
    }

    pub fn add_input(&mut self, file: InputFile) -> io::Result<()> {
        let id = self.next_input_id;
        self.next_input_id += 1;

        self.inputs.entry(InputId(id)).or_insert(file).open()?;

        Ok(())
    }
}
