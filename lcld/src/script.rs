use std::path::PathBuf;

use binfmt::fmt::Binfmt;

use crate::link::{InputId, RegionId};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ParsedScript {
    pub command: Vec<ScriptTopCommand>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SymbolDef {
    Extern(Vec<String>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ScriptTopCommand {
    Entry(String),
    Input(Vec<InputId>),
    Group(Vec<InputId>),
    Output(PathBuf),
    // Note: These are processed eagerly
    SearchPath(Vec<PathBuf>),
    Startup(InputId),
    OutputFormat(&'static dyn Binfmt),
    // Note: This isn't actually used, except to set the default `OutputFormat`
    Target(&'static dyn Binfmt),
    RegionAlias(String, RegionId),
}
