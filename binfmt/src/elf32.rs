pub type Elf32Format<Howto> = crate::elf::ElfFormat<crate::elf::Elf32, Howto>;

pub use crate::elf::{consts, Elf32};

#[cfg(feature = "w65")]
pub mod w65;
